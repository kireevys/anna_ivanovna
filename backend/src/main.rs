use ai_core::api::CoreApi;
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
    let fs_repo = match storage::FileSystem::init(buh_home.join("storage")) {
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let repo = Arc::new(fs_repo);
    let core = CoreApi::new(repo);

    if let Err(e) = cli::run(core) {
        eprintln!("Ошибка CLI: {e}");
        std::process::exit(1);
    }
}
