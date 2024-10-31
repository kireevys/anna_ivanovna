use anna_ivanovna::cli;
use anna_ivanovna::storage::plan_from_yaml;
use std::env;

fn main() {
    let path = env::current_dir().expect("Failed to get current directory");
    assert!(path.exists(), "Не найден файл: {}", path.display());
    println!("Используется файл плана {}", path.display());
    let plan = plan_from_yaml(path.as_path());
    cli::run(&plan);
}
