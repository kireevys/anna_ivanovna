use ai_app::api::CoreApi;
use clap::Parser;
use std::sync::Arc;
mod cli;
mod infra;
mod interfaces;
mod storage;

type Error = infra::config::Error;
fn get_buh_home() -> Result<std::path::PathBuf, Error> {
    infra::config::get_buh_home()
}

fn logging_init(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    infra::logging::init(dir, "anna_ivanovna.log")
}

fn migrate_excel<T: ai_app::storage::CoreRepo>(
    target: &T,
    file: std::path::PathBuf,
) -> Result<(), String> {
    let budgets = interfaces::excel_parser::parse_excel_csv(file)
        .map_err(|e| format!("Ошибка парсинга CSV: {e}"))?;

    let mut count = 0u32;
    for b in budgets {
        let id = CoreApi::<T>::build_budget_id();
        target.save_budget(id, b).map_err(|e| e.to_string())?;
        count += 1;
    }
    println!("Мигрировано из Excel: {count} бюджетов");
    Ok(())
}

async fn init_sqlite(
    buh_home: &std::path::Path,
) -> Result<storage::sqlite::SqliteRepo, String> {
    let db_path = buh_home.join("anna_ivanovna.db");
    storage::sqlite::SqliteRepo::init(&db_path).await
}

async fn run_web<R: ai_app::storage::CoreRepo + Clone + Send + Sync + 'static>(
    api: CoreApi<R>,
    host: String,
    port: u16,
) {
    if let Err(err) = interfaces::web::run(api, &format!("{host}:{port}")).await {
        eprintln!("Ошибка web-сервера: {err}");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    let buh_home = match get_buh_home() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Не удалось определить домашнюю директорию: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = logging_init(&buh_home) {
        eprintln!("Ошибка инициализации логирования: {e}");
        std::process::exit(1);
    }

    let cli = cli::Cli::parse();

    let repo = match init_sqlite(&buh_home).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Ошибка инициализации SQLite: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("location {}", repo.db_path());

    match cli.command {
        cli::Commands::Migrate { ref source } => {
            let (description, db_path) = match source {
                cli::MigrateSource::Fs => {
                    let fs_path = buh_home.join("storage");
                    (
                        format!("FS ({}) → SQLite", fs_path.display()),
                        repo.db_path().to_owned(),
                    )
                }
                cli::MigrateSource::Excel { file } => (
                    format!("Excel ({}) → SQLite", file.display()),
                    repo.db_path().to_owned(),
                ),
            };
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

            let result = match source {
                cli::MigrateSource::Fs => {
                    let fs = storage::fs::FileSystem::init(buh_home.join("storage"))
                        .map_err(|e| format!("Ошибка FS: {e}"));
                    match fs {
                        Ok(fs) => ai_app::migration::migrate(&fs, &repo),
                        Err(e) => Err(e),
                    }
                }
                cli::MigrateSource::Excel { file } => {
                    migrate_excel(&repo, file.clone())
                }
            };
            if let Err(e) = result {
                eprintln!("Ошибка миграции: {e}");
                std::process::exit(1);
            }
        }
        cli::Commands::Web { host, port } => {
            run_web(CoreApi::new(Arc::new(repo)), host, port).await;
        }
        cli::Commands::Budget(cmd) => {
            if let Err(e) = cli::run(CoreApi::new(Arc::new(repo)), cmd) {
                eprintln!("Ошибка CLI: {e}");
                std::process::exit(1);
            }
        }
    }
}
