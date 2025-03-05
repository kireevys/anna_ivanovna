use anna_ivanovna::cli;
use anna_ivanovna::storage::plan_from_yaml;
use std::env;

fn main() {
    const RESULT: &str = "result.csv";
    const PLAN: &str = "plan.yaml";
    let base = env::current_dir()
        .expect("Failed to get current directory")
        .join("storage");
    let result_path = base.join(RESULT);
    let plan_path = base.join(PLAN);
    assert!(
        plan_path.exists(),
        "Не найден файл: {}",
        plan_path.display()
    );
    println!("Используется файл плана {}", plan_path.display());
    let plan = plan_from_yaml(plan_path.as_path());
    cli::run(&plan, &result_path);
}
