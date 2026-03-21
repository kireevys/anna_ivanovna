use std::sync::Arc;

use ai_app::api::CoreApi;
use anna_ivanovna_lib::{
    infra::config::get_buh_home,
    interfaces::web::create_router,
    storage::sqlite::SqliteRepo,
};
use tauri::{RunEvent, async_runtime::spawn};
use tokio::sync::watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            spawn(start_backend(shutdown_rx));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build tauri application");

    app.run(move |_app_handle, event| {
        if let RunEvent::Exit = event {
            tracing::info!("Tauri exit, sending shutdown signal");
            let _ = shutdown_tx.send(true);
        }
    });
}

async fn start_backend(shutdown_rx: watch::Receiver<bool>) {
    if let Err(e) = run_backend(shutdown_rx).await {
        tracing::error!("Backend failed: {e}");
        std::process::exit(1);
    }
}

async fn run_backend(
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let buh_home =
        get_buh_home().map_err(|e| format!("Failed to resolve buh_home: {e}"))?;

    let db_path = buh_home.join("anna_ivanovna.db");
    let repo = SqliteRepo::init(&db_path)
        .await
        .map_err(|e| format!("SQLite init failed: {e}"))?;

    let api = CoreApi::new(Arc::new(repo));
    let app = create_router(api);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:31415").await?;

    tracing::info!("Axum backend listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.wait_for(|&v| v).await;
            tracing::info!("Shutting down Axum backend");
        })
        .await?;

    Ok(())
}
