mod config;

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use ai_app::api::CoreApi;
use anna_ivanovna_lib::{interfaces::web::create_router, storage::sqlite::SqliteRepo};
use config::TauriConfigProvider;
use tauri::{Manager, RunEvent, async_runtime::spawn};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::watch;

const BACKEND_HOST: &str = "127.0.0.1";
const BACKEND_PORT: u16 = 31415;

struct AppState {
    shutdown_rx: std::sync::Mutex<watch::Receiver<bool>>,
    backend_started: AtomicBool,
}

#[tauri::command]
fn is_configured(app: tauri::AppHandle) -> Result<bool, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("failed to resolve app config dir: {e}"))?;

    let provider = TauriConfigProvider::new(config_dir);

    if provider.has_settings() {
        return Ok(true);
    }

    // Migration: check BUH_HOME env or ~/.buh existence
    if let Some(legacy_home) = resolve_legacy_buh_home() {
        let defaults = TauriConfigProvider::load_defaults()
            .map_err(|e| format!("failed to load defaults: {e}"))?;
        provider
            .save_initial(&legacy_home, defaults.database)
            .map_err(|e| format!("failed to migrate config: {e}"))?;
        tracing::info!("Migrated legacy buh_home: {}", legacy_home.display());
        return Ok(true);
    }

    Ok(false)
}

fn resolve_legacy_buh_home() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("BUH_HOME") {
        let path = PathBuf::from(val);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let default = home.join(".buh");
        if default.exists() {
            return Some(default);
        }
    }

    None
}

#[tauri::command]
fn get_default_buh_home() -> Result<String, String> {
    let path = TauriConfigProvider::default_buh_home()
        .map_err(|e| format!("failed to get default: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn pick_data_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        handle
            .dialog()
            .file()
            .set_title("Выберите папку для данных Anna Ivanovna")
            .blocking_pick_folder()
    })
    .await
    .map_err(|e| format!("dialog error: {e}"))?;

    match result {
        Some(path) => {
            let p = path.into_path().map_err(|e| format!("invalid path: {e}"))?;
            Ok(Some(p.to_string_lossy().to_string()))
        }
        None => Ok(None),
    }
}

#[tauri::command]
async fn complete_setup(app: tauri::AppHandle, buh_home: String) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("failed to resolve app config dir: {e}"))?;

    let provider = TauriConfigProvider::new(config_dir);

    let defaults = TauriConfigProvider::load_defaults()
        .map_err(|e| format!("failed to load defaults: {e}"))?;

    let buh_home_path = PathBuf::from(&buh_home);

    provider
        .save_initial(&buh_home_path, defaults.database)
        .map_err(|e| format!("failed to save config: {e}"))?;

    let app_config = provider
        .load()
        .map_err(|e| format!("failed to load config: {e}"))?;

    try_start_backend(&app, app_config)?;

    Ok(())
}

#[tauri::command]
async fn start_app_backend(app: tauri::AppHandle) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("failed to resolve app config dir: {e}"))?;

    let provider = TauriConfigProvider::new(config_dir);

    let app_config = provider
        .load()
        .map_err(|e| format!("failed to load config: {e}"))?;

    try_start_backend(&app, app_config)?;

    Ok(())
}

fn try_start_backend(
    app: &tauri::AppHandle,
    config: ai_app::config::Config,
) -> Result<(), String> {
    let state = app.state::<AppState>();

    if state.backend_started.swap(true, Ordering::SeqCst) {
        tracing::warn!("Backend already started, ignoring duplicate request");
        return Ok(());
    }

    let shutdown_rx = state
        .shutdown_rx
        .lock()
        .map_err(|e| format!("lock error: {e}"))?
        .clone();

    spawn(start_backend(shutdown_rx, config));

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            shutdown_rx: std::sync::Mutex::new(shutdown_rx),
            backend_started: AtomicBool::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            is_configured,
            get_default_buh_home,
            pick_data_folder,
            complete_setup,
            start_app_backend,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build tauri application");

    app.run(move |_app_handle, event| {
        if let RunEvent::Exit = event {
            tracing::info!("Tauri exit, sending shutdown signal");
            let _ = shutdown_tx.send(true);
        }
    });
}

async fn start_backend(
    shutdown_rx: watch::Receiver<bool>,
    config: ai_app::config::Config,
) {
    if let Err(e) = run_backend(shutdown_rx, config).await {
        tracing::error!("Backend failed: {e}");
        std::process::exit(1);
    }
}

async fn run_backend(
    mut shutdown_rx: watch::Receiver<bool>,
    config: ai_app::config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo =
        SqliteRepo::init(std::path::Path::new(config.database.connection_string()))
            .await
            .map_err(|e| format!("SQLite init failed: {e}"))?;

    let api = CoreApi::new(Arc::new(repo));
    let app = create_router(api);

    let addr = format!("{BACKEND_HOST}:{BACKEND_PORT}");
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        format!(
            "Не удалось запустить сервер на {addr}: {e}. \
             Порт {BACKEND_PORT} занят — закройте приложение, которое его использует, \
             и попробуйте снова."
        )
    })?;

    tracing::info!("Axum backend listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.wait_for(|&v| v).await;
            tracing::info!("Shutting down Axum backend");
        })
        .await?;

    Ok(())
}
