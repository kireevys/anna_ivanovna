use ai_core::api::{CoreApi, CoreRepo};
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

fn migrate<S: CoreRepo, T: CoreRepo>(source: &S, target: &T) -> Result<(), String> {
    if let Some(plan) = source.get_plan() {
        let plan_id = uuid::Uuid::now_v7().to_string();
        target
            .save_plan(plan_id.clone(), plan)
            .map_err(|e| e.to_string())?;
        println!("Мигрирован план: {plan_id}");
    }

    let mut all_budgets = Vec::new();
    let mut cursor = None;
    loop {
        let page = source.budgets(cursor, 50);
        if page.items.is_empty() {
            break;
        }
        all_budgets.extend(page.items);
        cursor = page.next_cursor.clone();
        if cursor.is_none() {
            break;
        }
    }

    all_budgets.sort_by(|a, b| a.budget.income_date().cmp(b.budget.income_date()));

    for sb in &all_budgets {
        let id = uuid::Uuid::now_v7().to_string();
        target
            .save_budget(id, sb.budget.clone())
            .map_err(|e| e.to_string())?;
    }
    println!("Мигрировано бюджетов: {}", all_budgets.len());
    Ok(())
}

fn migrate_excel<T: CoreRepo>(
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

async fn run_web<R: CoreRepo + Clone + Send + Sync + 'static>(
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

    match cli.command {
        cli::Commands::Migrate { ref source } => {
            let (description, db_path) = match source {
                cli::MigrateSource::Fs => {
                    let fs_path = buh_home.join("storage");
                    (
                        format!("FS ({}) → SQLite", fs_path.display()),
                        repo.location().to_owned(),
                    )
                }
                cli::MigrateSource::Excel { file } => (
                    format!("Excel ({}) → SQLite", file.display()),
                    repo.location().to_owned(),
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
                        Ok(fs) => migrate(&fs, &repo),
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
