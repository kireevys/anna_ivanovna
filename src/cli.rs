use crate::distribute::{distribute, Income};
use crate::finance::Money;
use crate::planning::{IncomeSource, Plan};
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::io;
use std::io::Write;

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
pub fn run(plan: &Plan) {
    let cli = Cli::parse();

    match cli.command {
        Commands::AddIncome { amount } => {
            let source = choose_source(plan);
            let income = Income::new_today(source.clone(), Money::new_rub(amount));

            match distribute(plan, &income) {
                Ok(d) => println!("{d}"),
                Err(e) => println!("{e:?}"),
            }
        }
        Commands::ShowPlan => {
            println!("{plan:#?}");
        }
    }
}
