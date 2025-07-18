use anna_ivanovna::storage::FileSystem;
use std::fs::{OpenOptions, create_dir_all};
use std::path::Path;
use thiserror::Error;
use tracing::info;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

const STORAGE: &str = "storage";
const DEFAULT_HOME: &str = ".buh";
const ENV_BUH_HOME: &str = "BUH_HOME";
#[derive(Debug, Error)]
enum Error {
    #[error("config not found")]
    NoConfig,
}

fn get_buh_home() -> Result<std::path::PathBuf, Error> {
    if let Ok(val) = std::env::var(ENV_BUH_HOME) {
        info!("Использую BUH_HOME из переменной окружения: {val}");
        Ok(std::path::PathBuf::from(val))
    } else {
        let default = dirs::home_dir()
            .map(|h| h.join(DEFAULT_HOME))
            .ok_or(Error::NoConfig)?;
        info!(
            "BUH_HOME не задан, использую директорию по умолчанию: {}",
            default.display()
        );
        Ok(default)
    }
}

fn logging_init(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Создаём директорию, если нужно
    let logdir = dir.join("logs");
    let logfile = logdir.join("anna_ivanovna.log");
    println!("{}", logfile.display());
    if let Err(e) = create_dir_all(&logdir) {
        eprintln!("Не удалось создать директорию для логов: {e}");
        std::process::exit(1);
    }
    let file = OpenOptions::new().create(true).append(true).open(logfile)?;
    tracing_subscriber::fmt()
        .with_writer(BoxMakeWriter::new(file))
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
    Ok(())
}

fn main() {
    // Определяем директорию для логов
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

    // Автоматическая инициализация, если хранилище не найдено
    let fs_repo = match FileSystem::init(buh_home.join(STORAGE)) {
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    // Проверяем, есть ли аргументы командной строки
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Есть аргументы - запускаем CLI
        if let Err(e) = anna_ivanovna::cli::run(&fs_repo) {
            eprintln!("Ошибка CLI: {e}");
            std::process::exit(1);
        }
    } else {
        // Нет аргументов - запускаем TUI
        if let Err(e) = anna_ivanovna::tui::run(&fs_repo) {
            eprintln!("Ошибка TUI: {e}");
            std::process::exit(1);
        }
    }
}
