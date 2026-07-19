use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State, Url};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_store::StoreExt;

use crate::archive::{
    clear_archive_in, compose_with_state, import_archive_into, load_state_from_dir,
    persist_meta_to_dir, status_from_state, AppState, ArchiveMeta, ArchiveStatus,
};
use crate::catalog::{app_data_dir, archive_path, STORE_ARCHIVE_KEY, STORE_FILENAME};
use crate::compose::ComposeResult;
use crate::error::ComposeError;
use crate::workbook::write_template_archive;

#[tauri::command]
pub fn archive_status(state: State<'_, Mutex<AppState>>) -> Result<ArchiveStatus, ComposeError> {
    let state = state
        .lock()
        .map_err(|e| ComposeError::Catalog(e.to_string()))?;
    Ok(status_from_state(&state))
}

#[tauri::command]
pub fn compose_query(
    state: State<'_, Mutex<AppState>>,
    query: String,
) -> Result<ComposeResult, ComposeError> {
    let state = state
        .lock()
        .map_err(|e| ComposeError::Catalog(e.to_string()))?;
    compose_with_state(&state, &query)
}

#[tauri::command]
pub async fn import_archive(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<ArchiveStatus, ComposeError> {
    // Must be async so blocking_pick_file runs off the main/GTK thread.
    let picked = app
        .dialog()
        .file()
        .add_filter("Excel workbook", &["xlsx"])
        .set_title("Import prompt archive")
        .blocking_pick_file();

    let Some(file_path) = picked else {
        return Err(ComposeError::invalid(
            "import_cancelled",
            "import cancelled — no file selected",
        ));
    };

    let source = file_path_to_pathbuf(file_path)?;
    import_from_source(&app, &state, source)
}

/// E2E-only path import. Requires `PROMPT_COMPOSER_E2E=1` in the process environment.
#[tauri::command]
pub fn import_archive_from_path(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
    path: String,
) -> Result<ArchiveStatus, ComposeError> {
    if env::var("PROMPT_COMPOSER_E2E").ok().as_deref() != Some("1") {
        return Err(ComposeError::invalid(
            "e2e_disabled",
            "import_archive_from_path requires PROMPT_COMPOSER_E2E=1",
        ));
    }
    import_from_source(&app, &state, PathBuf::from(path))
}

/// E2E-only: reload the frontend via native WebView navigate (avoids JS location.href crashes).
#[tauri::command]
pub fn e2e_reload_frontend(app: AppHandle) -> Result<(), ComposeError> {
    if env::var("PROMPT_COMPOSER_E2E").ok().as_deref() != Some("1") {
        return Err(ComposeError::invalid(
            "e2e_disabled",
            "e2e_reload_frontend requires PROMPT_COMPOSER_E2E=1",
        ));
    }
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| ComposeError::invalid("no_window", "main window not found"))?;
    let url = Url::parse("tauri://localhost")
        .map_err(|e| ComposeError::invalid("bad_url", e.to_string()))?;
    win.navigate(url)
        .map_err(|e| ComposeError::Catalog(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn clear_archive(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<ArchiveStatus, ComposeError> {
    let data_dir = app_data_dir(&app)?;
    let mut state = state
        .lock()
        .map_err(|e| ComposeError::Catalog(e.to_string()))?;
    let status = clear_archive_in(&mut state, &data_dir)?;
    // Also clear Tauri store sidecar used by UI metadata display
    clear_store_meta(&app)?;
    Ok(status)
}

/// Save a blank archive template (correct sheets/columns + e.g. hint rows).
#[tauri::command]
pub async fn export_archive_template(app: AppHandle) -> Result<String, ComposeError> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Excel workbook", &["xlsx"])
        .set_file_name("prompt_archive_template.xlsx")
        .set_title("Save archive template")
        .blocking_save_file();

    let Some(file_path) = picked else {
        return Err(ComposeError::invalid(
            "export_cancelled",
            "export cancelled — no path selected",
        ));
    };

    let path = file_path_to_pathbuf(file_path)?;
    write_template_archive(&path)?;
    Ok(path.display().to_string())
}

pub fn load_app_state(app: &AppHandle) -> Result<AppState, ComposeError> {
    let data_dir = app_data_dir(app)?;
    let mut state = load_state_from_dir(&data_dir)?;

    // Prefer store metadata when present (production path)
    if let Ok(Some(meta)) = load_store_meta(app) {
        if state.catalog.is_some() {
            state.meta = Some(meta);
        }
    } else if state.catalog.is_some() && state.meta.is_none() {
        // Fall back: archive file exists without meta
        let path = archive_path(app)?;
        state.meta = Some(ArchiveMeta {
            original_name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("prompt_archive.xlsx")
                .to_string(),
            imported_at: 0,
        });
    }

    if state.catalog.is_some() {
        log::info!(
            "loaded prompt archive from {}",
            archive_path(app)?.display()
        );
    }
    Ok(state)
}

fn import_from_source(
    app: &AppHandle,
    state: &State<'_, Mutex<AppState>>,
    source: PathBuf,
) -> Result<ArchiveStatus, ComposeError> {
    let data_dir = app_data_dir(app)?;
    let mut state = state
        .lock()
        .map_err(|e| ComposeError::Catalog(e.to_string()))?;
    let status = import_archive_into(&mut state, &source, &data_dir)?;
    if let Some(meta) = &state.meta {
        write_store_meta(app, meta)?;
        let _ = persist_meta_to_dir(&data_dir, Some(meta));
    }
    Ok(status)
}

fn file_path_to_pathbuf(file: FilePath) -> Result<PathBuf, ComposeError> {
    match file {
        FilePath::Path(p) => Ok(p),
        FilePath::Url(url) => url.to_file_path().map_err(|_| {
            ComposeError::Catalog(format!("could not convert file URL to path: {url}"))
        }),
    }
}

fn write_store_meta(app: &AppHandle, meta: &ArchiveMeta) -> Result<(), ComposeError> {
    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| ComposeError::Catalog(format!("store open: {e}")))?;
    store.set(
        STORE_ARCHIVE_KEY.to_string(),
        serde_json::json!({
            "originalName": meta.original_name,
            "importedAt": meta.imported_at,
        }),
    );
    store
        .save()
        .map_err(|e| ComposeError::Catalog(format!("store save: {e}")))?;
    Ok(())
}

fn clear_store_meta(app: &AppHandle) -> Result<(), ComposeError> {
    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| ComposeError::Catalog(format!("store open: {e}")))?;
    store.delete(STORE_ARCHIVE_KEY);
    store
        .save()
        .map_err(|e| ComposeError::Catalog(format!("store save: {e}")))?;
    Ok(())
}

fn load_store_meta(app: &AppHandle) -> Result<Option<ArchiveMeta>, ComposeError> {
    let store = match app.store(STORE_FILENAME) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let Some(value) = store.get(STORE_ARCHIVE_KEY) else {
        return Ok(None);
    };
    match serde_json::from_value::<ArchiveMeta>(value) {
        Ok(meta) => Ok(Some(meta)),
        Err(_) => Ok(None),
    }
}
