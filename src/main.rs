use anna_ivanovna::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Ошибка: {e:?}");
        std::process::exit(1);
    }
}
