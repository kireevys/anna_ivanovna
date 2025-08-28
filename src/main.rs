
type Error = anna_ivanovna::infra::config::Error;
fn get_buh_home() -> Result<std::path::PathBuf, Error> { anna_ivanovna::infra::config::get_buh_home() }

fn logging_init(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    anna_ivanovna::infra::logging::init(dir, "anna_ivanovna.log")
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
    let fs_repo = match anna_ivanovna::storage::FileSystem::init(buh_home.join("storage")) {
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
        if let Err(e) = anna_ivanovna::interfaces::tui::run(&fs_repo) {
            eprintln!("Ошибка TUI: {e}");
            std::process::exit(1);
        }
    }
}
