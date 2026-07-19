pub mod archive;
pub mod archive_schema;
pub mod catalog;
pub mod commands;
pub mod compose;
pub mod error;
pub mod fixtures_data;
pub mod parse;
pub mod workbook;

use std::env;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{Manager, Url};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build());

    // Embedded WebDriver must only run under e2e (TAURI_WEBDRIVER_PORT set).
    // init() otherwise defaults to :4445 and hooks every webview during normal `tauri dev`.
    let webdriver_active = env::var_os("TAURI_WEBDRIVER_PORT").is_some();

    if cfg!(debug_assertions) && !webdriver_active {
        builder = builder.plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        );
    }

    if webdriver_active {
        builder = builder
            .plugin(tauri_plugin_wdio::init())
            .plugin(tauri_plugin_wdio_webdriver::init());
    }

    builder
        .setup(|app| {
            let state = commands::load_app_state(app.handle())?;
            app.manage(Mutex::new(state));

            // WebDriver attaches while WebKit is still on about:blank. Under E2E,
            // force the asset URL via the native navigate API (JS location.href
            // intermittently tears down the embedded WebDriver server on Linux).
            if env::var("PROMPT_COMPOSER_E2E").ok().as_deref() == Some("1") {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(300));
                    let app = handle.clone();
                    let _ = handle.run_on_main_thread(move || {
                        if let Some(win) = app.get_webview_window("main") {
                            if let Ok(url) = Url::parse("tauri://localhost") {
                                let _ = win.navigate(url);
                            }
                        }
                    });
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::archive_status,
            commands::import_archive,
            commands::import_archive_from_path,
            commands::clear_archive,
            commands::compose_query,
            commands::export_archive_template,
            commands::e2e_reload_frontend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
