use std::{path::Path, sync::Arc};

use ai_app::api::CoreApi;
use anna_ivanovna_lib::{cli, infra, interfaces, storage};
use clap::Parser;

async fn migrate_excel<T: ai_app::storage::CoreRepo>(
    target: &T,
    file: std::path::PathBuf,
) -> Result<(), String> {
    let budgets = interfaces::excel_parser::parse_excel_csv(file)
        .map_err(|e| format!("Ошибка парсинга CSV: {e}"))?;

    let mut count = 0u32;
    for b in budgets {
        let id = ai_app::storage::build_id();
        target.save_budget(id, b).await.map_err(|e| e.to_string())?;
        count += 1;
    }
    println!("Мигрировано из Excel: {count} бюджетов");
    Ok(())
}

async fn run_web<R: ai_app::storage::CoreRepo + Clone + Send + Sync + 'static>(
    api: CoreApi<R>,
    host: &str,
    port: u16,
) {
    if let Err(err) = interfaces::web::run(api, &format!("{host}:{port}")).await {
        eprintln!("Ошибка web-сервера: {err}");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    let overrides = match &cli.command {
        cli::Commands::Web { host, port } => infra::config::ConfigOverrides {
            host: host.clone(),
            port: *port,
        },
        _ => infra::config::ConfigOverrides::default(),
    };

    let config = match infra::config::init(cli.buh_home.clone(), overrides) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Ошибка инициализации: {e}");
            std::process::exit(1);
        }
    };

    let repo = match storage::sqlite::SqliteRepo::init(Path::new(
        config.database.connection_string(),
    ))
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Ошибка инициализации SQLite: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("location {}", repo.db_path());

    match cli.command {
        cli::Commands::MigrateExcel { ref file } => {
            let description = format!("Excel ({}) → SQLite", file.display());
            let db_path = repo.db_path().to_owned();
            println!("Миграция: {description}");
            println!("БД: {db_path}");
            print!("Продолжить? [y/N] ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() != "y" {
                println!("Отменено");
                return;
            }

            if let Err(e) = migrate_excel(&repo, file.clone()).await {
                eprintln!("Ошибка миграции: {e}");
                std::process::exit(1);
            }
        }
        cli::Commands::Web { .. } => {
            run_web(
                CoreApi::new(Arc::new(repo)),
                &config.server.host,
                config.server.port,
            )
            .await;
        }
        cli::Commands::Budget(cmd) => {
            if let Err(e) = cli::run(CoreApi::new(Arc::new(repo)), cmd).await {
                eprintln!("Ошибка CLI: {e}");
                std::process::exit(1);
            }
        }
    }
}
