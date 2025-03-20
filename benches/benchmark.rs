use anna_ivanovna::distribute::{distribute, Income};
use anna_ivanovna::finance::Money;
use anna_ivanovna::planning::IncomeSource;
use anna_ivanovna::storage::{distribute_to_yaml, plan_from_yaml};
use chrono::Local;
use criterion::{criterion_group, criterion_main, Criterion};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn rub(v: f64) -> Money {
    Money::new_rub(Decimal::from_f64(v).unwrap())
}

fn plan_distribute_from_file() {
    sleep(Duration::new(0, 0));
    let result_path: String = format!("{}.yaml", Local::now().format("%Y-%m-%d"));
    let base = env::current_dir()
        .expect("Failed to get current directory")
        .join("storage");

    let result_path = base.join(result_path);
    let plan = plan_from_yaml(Path::new("src/test_storage/plan.yaml")).expect("Проблема");
    let source = IncomeSource::new("Зарплата".to_string(), rub(1.0));
    let income = Income::new_today(source, rub(100.0));
    let d = distribute(&plan, &income).expect("whaaaat???");
    let mut file = File::create(result_path).expect("cannot file");
    file.write_all(distribute_to_yaml(&d).as_bytes())
        .expect("cannot write to file");
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("plan_reader", |b| {
        b.iter(plan_distribute_from_file);
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
