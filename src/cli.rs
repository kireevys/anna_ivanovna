use crate::distribute::{distribute, Income};
use crate::finance::Money;
use crate::planning::{IncomeSource, Plan};
use crate::storage::{distribute_to_yaml, plan_from_yaml};
use chrono::Local;
use clap::{Parser, Subcommand};
use homedir::my_home;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::{fs, io};

const PLAN: &str = "plan.yaml";
const BASE: &str = "buh";
const STORAGE: &str = "storage";
const INCOMES: &str = "incomes";

#[derive(Parser)]
#[clap(name = "Anna Ivanovna", version = env!("CARGO_PKG_VERSION"), author = "github.com/kireevys")]
struct Cli {
    /// Подкоманда для работы с финансами
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Добавить источник дохода
    #[clap(alias = "income")]
    AddIncome { amount: Decimal },

    /// Отобразить план
    #[clap(alias = "plan")]
    ShowPlan,
}

fn user_input() -> String {
    let mut source_num = String::new();
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut source_num)
        .expect("Не удалось прочитать строку");
    source_num.trim().to_string()
}

fn enumerated_sources(plan: &Plan) -> HashMap<String, &IncomeSource> {
    let mut hm = HashMap::new();
    for (n, i) in plan.sources.iter().enumerate() {
        hm.insert((n + 1).to_string(), i);
    }
    hm
}

fn choose_source(plan: &Plan) -> &IncomeSource {
    if plan.sources.len() == 1 {
        return &plan.sources[0];
    }
    let sources = enumerated_sources(plan);
    let mut sorted_vec: Vec<_> = sources.iter().collect();
    sorted_vec.sort_by_key(|&(k, _)| k);
    println!("В Бюджете указано несколько источников дохода:");
    for (n, i) in &sorted_vec {
        println!("{n} : {i}");
    }
    print!("Введите номер источника $ ");
    sources.get(&user_input()).expect("Неверный источник")
}

/// Запуск cli для работы с выбранным планом
///
/// # Arguments
///
/// * `plan`: Ссылка на Бюджет
///
/// # Panics
/// При неожиданном пользовательском вводе
///
/// returns: ()
pub fn run() {
    let cli = Cli::parse();
    let base = my_home()
        .expect("Не удалось определить Домашнюю Директорию")
        .map(|home| home.join(BASE).join(STORAGE))
        .expect("Не удалось подключить Хранилище");
    fs::create_dir_all(base.clone()).unwrap_or_else(|_| panic!("Невозможно создать {BASE}"));
    let result_path: String = format!("{}.yaml", Local::now().format("%Y-%m-%d"));
    let result_path = base.join(INCOMES).join(result_path);
    let plan_p = base.join(PLAN);
    assert!(plan_p.exists(), "Не найден файл: {}", plan_p.display());
    let plan = plan_from_yaml(plan_p.as_path());

    println!("Используется файл плана {}", plan_p.display());
    match cli.command {
        Commands::AddIncome { amount } => {
            let source = choose_source(&plan);
            let income = Income::new_today(source.clone(), Money::new_rub(amount));

            match distribute(&plan, &income) {
                Ok(d) => {
                    let mut file = File::create(&result_path).expect("cannot file");
                    file.write_all(distribute_to_yaml(&d).as_bytes())
                        .expect("cannot write to file");
                    println!("Записано в {result_path:?}");
                    println!("{d}");
                }
                Err(e) => println!("{e:?}"),
            }
        }
        Commands::ShowPlan => {
            // TODO:
            println!("{plan:#?}");
        }
    }
}
