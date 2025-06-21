use crate::distribute::{Income, distribute};
use crate::finance::Money;
use crate::planning::{IncomeSource, Plan};
use crate::storage::plan_from_yaml;
use chrono::Local;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, fs, io};

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
    CantPrepareStorage,
}

#[derive(Parser)]
#[clap(
    name = "Anna Ivanovna",
    version = env!("CARGO_PKG_VERSION"),
    author = "github.com/kireevys",
    about = "Планировщик бюджета - автоматическое распределение доходов по статьям расходов",
    long_about = "Anna Ivanovna помогает автоматически распределять ваши доходы по заранее составленному плану бюджета.

Создайте план один раз, и программа будет автоматически рассчитывать, сколько денег тратить на каждую категорию при получении дохода.

Примеры:
  anna_ivanovna prepare-storage    # Подготовить папки для работы
  anna_ivanovna show-plan          # Показать текущий план
  anna_ivanovna add-income 50000   # Добавить доход 50000₽"
)]
struct Cli {
    /// Подкоманда для работы с финансами
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Добавить доход и распределить его согласно плану
    #[clap(alias = "income")]
    AddIncome {
        /// Сумма дохода в рублях
        amount: Decimal,
    },

    /// Отобразить текущий план бюджета
    #[clap(alias = "show-plan")]
    Plan,

    /// Подготовить структуру папок и файлов для работы
    #[clap(alias = "prepare")]
    PrepareStorage,

    /// Вывести справку по командам
    #[clap(alias = "readme")]
    Manual,
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

fn process_income(plan: &Plan, amount: Decimal, incomes_path: &Path) -> Result<(), Error> {
    let source = choose_source(plan)?;
    let income = Income::new_today(source.clone(), Money::new_rub(amount));

    match distribute(plan, &income) {
        Ok(d) => {
            let result_path =
                incomes_path.join(format!("{}.json", Local::now().format("%Y-%m-%d"),));
            let mut file = File::create(&result_path).map_err(|_| Error::CantWriteResult)?;

            let json_result =
                serde_json::to_string_pretty(&d).map_err(|_| Error::CantWriteResult)?;
            file.write_all(json_result.as_bytes())
                .map_err(|_| Error::CantWriteResult)?;

            println!("Записано в {result_path:?}");
            println!("{d}");
        }
        Err(e) => println!("{e:?}"),
    }
    Ok(())
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
    let incomes_path = storage.join(INCOMES);
    let plan_p = storage.join(PLAN);
    plan_p.as_path().try_exists().map_err(|_| Error::NoPlan)?;

    let plan = plan_from_yaml(plan_p.as_path()).map_err(|_| Error::NoPlan)?;

    match cli.command {
        Commands::AddIncome { amount } => {
            println!("Используется файл плана {plan_p:?}");
            process_income(&plan, amount, &incomes_path)?;
        }
        Commands::Plan => {
            println!("{plan}");
        }
        Commands::PrepareStorage => {
            if incomes_path.exists() {
                println!("Хранилище уже подготовлено {incomes_path:?}");
            } else {
                fs::create_dir_all(incomes_path.clone()).map_err(|e| {
                    eprintln!("{e}");
                    Error::CantPrepareStorage
                })?;
                println!("Создана директория {incomes_path:?}");
            };

            if plan_p.exists() {
                println!("Файл плана уже существует {plan_p:?}");
            } else {
                fs::write(plan_p.clone(), "").map_err(|e| {
                    eprintln!("{e}");
                    Error::CantPrepareStorage
                })?;
                println!("Создан файл плана {plan_p:?}");
            }
        }
        Commands::Manual => {
            println!(
                "https://github.com/kireevys/anna_ivanovna/blob/master/README.md#2-первоначальная-настройка"
            );
        }
    }
    Ok(())
}
