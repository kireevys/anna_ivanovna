use crate::distribute::{Income, distribute};
use crate::finance::Money;
use crate::planning::{IncomeSource, Plan};
use crate::storage::plan_from_yaml;
use chrono::Local;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fs, io};

const PLAN: &str = "plan.yaml";
const STORAGE: &str = "storage";
const INCOMES: &str = "incomes";
const DEFAULT_HOME: &str = ".buh";
const ENV_BUH_HOME: &str = "BUH_HOME";

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
  anna_ivanovna plan               # Показать текущий план
  anna_ivanovna income 50000       # Добавить доход 50000₽"
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

fn get_buh_home() -> Result<std::path::PathBuf, Error> {
    if let Ok(val) = std::env::var(ENV_BUH_HOME) {
        println!(
            "🏠 [anna_ivanovna] Использую BUH_HOME из переменной окружения: {}",
            val
        );
        Ok(std::path::PathBuf::from(val))
    } else {
        let default = dirs::home_dir()
            .map(|h| h.join(DEFAULT_HOME))
            .ok_or(Error::NoConfig)?;
        println!(
            "🏠 [anna_ivanovna] BUH_HOME не задан, использую директорию по умолчанию: {}",
            default.display()
        );
        Ok(default)
    }
}

pub fn auto_prepare_storage() -> Result<(), String> {
    let buh_dir = std::env::var(ENV_BUH_HOME)
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(DEFAULT_HOME))
                .ok_or("Не удалось определить домашнюю директорию".to_string())
        })?;

    println!(
        "📦 [anna_ivanovna] Хранилище не найдено, инициализирую: {}",
        buh_dir.display()
    );

    if buh_dir.exists() {
        return Err(format!(
            "❗️ Хранилище уже инициализировано: {}",
            buh_dir.display()
        ));
    }

    // Создаём директорию
    fs::create_dir_all(&buh_dir).map_err(|e| format!("Ошибка создания директории: {e}"))?;
    println!(
        "📁 [anna_ivanovna] Создана директория: {}",
        buh_dir.display()
    );

    // Вызываем prepare-логику (создание поддиректорий и файлов)
    let storage = buh_dir.join("storage");
    let incomes_path = storage.join("incomes");
    let plan_p = storage.join("plan.yaml");

    fs::create_dir_all(&incomes_path).map_err(|e| format!("Ошибка создания incomes: {e}"))?;
    println!(
        "📁 [anna_ivanovna] Создана директория: {}",
        incomes_path.display()
    );
    if !plan_p.exists() {
        // Копируем example/plan.yaml, если он есть
        let example_plan = std::path::Path::new("example/plan.yaml");
        if example_plan.exists() {
            fs::copy(example_plan, &plan_p)
                .map_err(|e| format!("Ошибка копирования example/plan.yaml: {e}"))?;
            println!(
                "📄 [anna_ivanovna] Скопирован пример плана: {} → {}",
                example_plan.display(),
                plan_p.display()
            );
            println!(
                "✏️  [anna_ivanovna] Перейдите к этому файлу и отредактируйте его под себя перед использованием!"
            );
        } else {
            fs::write(&plan_p, "").map_err(|e| format!("Ошибка создания plan.yaml: {e}"))?;
            println!(
                "📄 [anna_ivanovna] Создан пустой файл плана: {}",
                plan_p.display()
            );
            println!(
                "✏️  [anna_ivanovna] Перейдите к этому файлу и заполните его перед использованием!"
            );
        }
    }

    println!(
        "✅ [anna_ivanovna] Хранилище инициализировано: {}",
        buh_dir.display()
    );
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
    // Автоматическая инициализация, если хранилище не найдено
    let buh_home = get_buh_home()?;
    if !buh_home.exists() {
        if let Err(e) = auto_prepare_storage() {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    let cli = Cli::parse();
    let home = buh_home;
    let storage = home.join(STORAGE);
    storage.try_exists().map_err(|_| Error::NoPlan)?;
    let incomes_path = storage.join(INCOMES);
    let plan_p = storage.join(PLAN);
    plan_p.as_path().try_exists().map_err(|_| Error::NoPlan)?;

    let plan = plan_from_yaml(plan_p.as_path()).map_err(|_| Error::NoPlan)?;

    match cli.command {
        Commands::AddIncome { amount } => {
            println!(
                "[anna_ivanovna] Используется файл плана: {}",
                plan_p.display()
            );
            process_income(&plan, amount, &incomes_path)?;
        }
        Commands::Plan => {
            println!("{plan}");
        }
        Commands::PrepareStorage => {
            if incomes_path.exists() {
                println!(
                    "[anna_ivanovna] Хранилище уже подготовлено: {}",
                    incomes_path.display()
                );
            } else {
                fs::create_dir_all(incomes_path.clone()).map_err(|e| {
                    eprintln!("{e}");
                    Error::CantPrepareStorage
                })?;
                println!(
                    "[anna_ivanovna] Создана директория: {}",
                    incomes_path.display()
                );
            };

            if plan_p.exists() {
                println!(
                    "[anna_ivanovna] Файл плана уже существует: {}",
                    plan_p.display()
                );
            } else {
                fs::write(plan_p.clone(), "").map_err(|e| {
                    eprintln!("{e}");
                    Error::CantPrepareStorage
                })?;
                println!("[anna_ivanovna] Создан файл плана: {}", plan_p.display());
            }
        }
        Commands::Manual => {
            println!(
                "Anna Ivanovna - CLI для управления бюджетом. Используйте --help для справки."
            );
        }
    }
    Ok(())
}
