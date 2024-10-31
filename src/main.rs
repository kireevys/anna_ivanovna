use anna_ivanovna::cli;
use anna_ivanovna::storage::plan_from_yaml;
use std::path::Path;
use std::{env, panic};

fn main() {
    let path = match env::current_dir() {
        Ok(path) => path.join(Path::new("storage/plan.yaml")),
        Err(e) => panic!("Error retrieving current directory: {e}"),
    };
    assert!(path.exists(), "Не найден файл: {}", path.display());
    println!("Используется файл плана {}", path.display());
    let plan = plan_from_yaml(path.as_path());
    cli::run(&plan);
}
