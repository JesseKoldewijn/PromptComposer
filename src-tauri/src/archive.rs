use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::catalog::{Catalog, CatalogCounts, ARCHIVE_FILENAME};
use crate::compose::{compose_from_query, ComposeResult};
use crate::error::ComposeError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveMeta {
    pub original_name: String,
    pub imported_at: u64,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub catalog: Option<Catalog>,
    pub meta: Option<ArchiveMeta>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStatus {
    pub loaded: bool,
    pub original_name: Option<String>,
    pub imported_at: Option<u64>,
    pub counts: Option<CatalogCounts>,
}

pub fn status_from_state(state: &AppState) -> ArchiveStatus {
    match (&state.catalog, &state.meta) {
        (Some(catalog), meta) => ArchiveStatus {
            loaded: true,
            original_name: meta.as_ref().map(|m| m.original_name.clone()),
            imported_at: meta.as_ref().map(|m| m.imported_at),
            counts: Some(catalog.counts()),
        },
        (None, _) => ArchiveStatus {
            loaded: false,
            original_name: None,
            imported_at: None,
            counts: None,
        },
    }
}

/// Dialog-free import: validate source, copy into `data_dir`, update state + sidecar meta JSON.
pub fn import_archive_into(
    state: &mut AppState,
    source: &Path,
    data_dir: &Path,
) -> Result<ArchiveStatus, ComposeError> {
    let catalog = Catalog::load(source)?;
    fs::create_dir_all(data_dir)
        .map_err(|e| ComposeError::Catalog(format!("failed to create data dir: {e}")))?;

    let dest = data_dir.join(ARCHIVE_FILENAME);
    fs::copy(source, &dest).map_err(|e| {
        ComposeError::Catalog(format!("failed to save archive to {}: {e}", dest.display()))
    })?;

    let original_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("prompt_archive.xlsx")
        .to_string();

    let meta = ArchiveMeta {
        original_name,
        imported_at: now_secs(),
    };
    persist_meta_to_dir(data_dir, Some(&meta))?;

    state.catalog = Some(catalog);
    state.meta = Some(meta);
    Ok(status_from_state(state))
}

pub fn clear_archive_in(
    state: &mut AppState,
    data_dir: &Path,
) -> Result<ArchiveStatus, ComposeError> {
    let dest = data_dir.join(ARCHIVE_FILENAME);
    if dest.exists() {
        fs::remove_file(&dest)
            .map_err(|e| ComposeError::Catalog(format!("failed to remove archive: {e}")))?;
    }
    persist_meta_to_dir(data_dir, None)?;
    state.catalog = None;
    state.meta = None;
    Ok(status_from_state(state))
}

pub fn compose_with_state(state: &AppState, query: &str) -> Result<ComposeResult, ComposeError> {
    let catalog = state.catalog.as_ref().ok_or_else(|| {
        ComposeError::invalid(
            "no_archive",
            "no archive loaded — upload an .xlsx prompt archive first",
        )
    })?;
    compose_from_query(catalog, query)
}

pub fn load_state_from_dir(data_dir: &Path) -> Result<AppState, ComposeError> {
    let meta = load_meta_from_dir(data_dir)?;
    let path = data_dir.join(ARCHIVE_FILENAME);
    if path.is_file() {
        match Catalog::load(&path) {
            Ok(catalog) => {
                return Ok(AppState {
                    catalog: Some(catalog),
                    meta,
                });
            }
            Err(e) => {
                log::warn!("stored archive failed to load ({e}); starting empty");
            }
        }
    }
    Ok(AppState {
        catalog: None,
        meta: None,
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn meta_path(data_dir: &Path) -> PathBuf {
    data_dir.join("archive_meta.json")
}

pub fn persist_meta_to_dir(
    data_dir: &Path,
    meta: Option<&ArchiveMeta>,
) -> Result<(), ComposeError> {
    fs::create_dir_all(data_dir)
        .map_err(|e| ComposeError::Catalog(format!("failed to create data dir: {e}")))?;
    let path = meta_path(data_dir);
    match meta {
        Some(meta) => {
            let value = json!({
                "originalName": meta.original_name,
                "importedAt": meta.imported_at,
            });
            fs::write(
                &path,
                serde_json::to_vec_pretty(&value)
                    .map_err(|e| ComposeError::Catalog(format!("meta serialize: {e}")))?,
            )
            .map_err(|e| ComposeError::Catalog(format!("meta write: {e}")))?;
        }
        None => {
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|e| ComposeError::Catalog(format!("meta remove: {e}")))?;
            }
        }
    }
    Ok(())
}

fn load_meta_from_dir(data_dir: &Path) -> Result<Option<ArchiveMeta>, ComposeError> {
    let path = meta_path(data_dir);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| ComposeError::Catalog(format!("meta read: {e}")))?;
    match serde_json::from_slice::<ArchiveMeta>(&bytes) {
        Ok(meta) => Ok(Some(meta)),
        Err(_) => Ok(None),
    }
}
