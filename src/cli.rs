use crate::distribute::{distribute, Income};
use crate::finance::Money;
use crate::planning::{IncomeSource, Plan};
use crate::storage::{distribute_to_yaml, plan_from_yaml};
use chrono::Local;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, io};

const PLAN: &str = "plan.yaml";
const STORAGE: &str = "storage";
const INCOMES: &str = "incomes";
const HOMEVAR: &str = "HOMEVAR";

#[derive(Debug)]
pub enum Error {
    NoConfig,
    NoPlan,
    CantWriteResult,
    InvalidInput,
}

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

fn user_input() -> Result<usize, Error> {
    let mut source_num = String::new();
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut source_num)
        .map_err(|_| Error::InvalidInput)?;
    source_num
        .trim()
        .parse::<usize>()
        .map_err(|_| Error::InvalidInput)
}

fn choose_source(plan: &Plan) -> Result<&IncomeSource, Error> {
    if plan.sources.len() == 1 {
        return plan.sources.first().ok_or(Error::NoPlan);
    }
    println!("В Бюджете указано несколько источников дохода:");
    for (n, i) in plan.sources.iter().enumerate() {
        println!("{n} : {i}");
    }
    print!("Введите номер источника $ ");
    let input = user_input()?;
    plan.sources.get(input).ok_or(Error::InvalidInput)
}

/// Запуск cli для работы с выбранным планом
///
/// # Arguments
///
/// * `plan`: Ссылка на Бюджет
///
/// # Errors
/// При неожиданном пользовательском вводе
///
/// returns: ()
pub fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    if dotenv().is_err() {
        eprintln!(
            "⚠️ .env файл не найден, используем переменные окружения {:?}",
            env::var(HOMEVAR)
        );
    }
    let home = env::var(HOMEVAR)
        .map(|p| Path::new(p.as_str()).to_path_buf())
        .map_err(|_e| Error::NoConfig)?;
    let storage = home.join(STORAGE);
    storage.try_exists().map_err(|_| Error::NoPlan)?;
    let result_path: String = format!("{}.yaml", Local::now().format("%Y-%m-%d"));
    let result_path = storage.join(INCOMES).join(result_path);
    let plan_p = storage.join(PLAN);
    plan_p.as_path().try_exists().map_err(|_| Error::NoPlan)?;
    println!("Используется файл плана {plan_p:?}");
    let plan = plan_from_yaml(plan_p.as_path()).map_err(|_| Error::NoPlan)?;

    match cli.command {
        Commands::AddIncome { amount } => {
            let source = choose_source(&plan)?;
            let income = Income::new_today(source.clone(), Money::new_rub(amount));

            match distribute(&plan, &income) {
                Ok(d) => {
                    let mut file =
                        File::create(&result_path).map_err(|_| Error::CantWriteResult)?;
                    file.write_all(distribute_to_yaml(&d).as_bytes())
                        .map_err(|_| Error::CantWriteResult)?;
                    println!("Записано в {result_path:?}");
                    println!("{d}");
                }
                Err(e) => println!("{e:?}"),
            }
            Ok(())
        }
        Commands::ShowPlan => {
            // TODO: Красивый План
            println!("{plan:#?}");
            Ok(())
        }
    }
}
