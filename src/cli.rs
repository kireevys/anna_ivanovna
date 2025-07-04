use crate::api::{distribute_budget, get_plan, save_budget};
use crate::core::distribute::Income;
use crate::core::finance::Money;
use crate::core::planning::{IncomeSource, Plan};
use crate::storage::FileSystem;
use crate::tui::run_tui;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

const STORAGE: &str = "storage";
const DEFAULT_HOME: &str = ".buh";
const ENV_BUH_HOME: &str = "BUH_HOME";

#[derive(Debug)]
pub enum Error {
    NoConfig,
    NoPlan,
    CantWriteResult,
    InvalidInput,
    CantPrepareStorage,
    CantDistribute,
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
    Plan,

    /// Показать бюджет по id
    #[clap(alias = "show")]
    ShowBudget {
        /// id бюджета
        id: String,
    },

    /// Подготовить структуру папок и файлов для работы
    #[clap(alias = "prepare")]
    PrepareStorage { path: PathBuf },

    /// Вывести справку по командам
    #[clap(alias = "readme")]
    Manual,

    /// Запустить TUI-интерфейс
    Tui,
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

fn get_buh_home() -> Result<std::path::PathBuf, Error> {
    if let Ok(val) = std::env::var(ENV_BUH_HOME) {
        println!("🏠 [anna_ivanovna] Использую BUH_HOME из переменной окружения: {val}");
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
    let fs_repo = match FileSystem::init(buh_home.join(STORAGE)) {
        Ok(fs) => {
            info!(fs=?fs);
            fs
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let cli = Cli::parse();
    let plan = get_plan(&fs_repo).ok_or(Error::NoPlan)?;
    let start = std::time::Instant::now();
    match cli.command {
        Commands::AddIncome { amount } => {
            let source = choose_source(&plan)?;
            let income = Income::new_today(source.clone(), Money::new_rub(amount));
            let budget = distribute_budget(&plan, &income).map_err(|_| Error::CantDistribute)?;

            println!("{budget}");
            let id = save_budget(budget, &fs_repo).map_err(|_| Error::CantWriteResult)?;

            println!("💾 {id} сохранен");
        }
        Commands::Plan => {
            println!("{plan}");
        }
        Commands::ShowBudget { id } => match crate::api::budget_by_id(&fs_repo, &id) {
            Some(budget) => println!("{budget}"),
            None => eprintln!("Ошибка: не удалось загрузить или распарсить бюджет с id {id}"),
        },
        Commands::PrepareStorage { path } => {
            if let Err(e) = FileSystem::init(path) {
                eprintln!("{e}");
            }
        }
        Commands::Manual => {
            println!(
                "Anna Ivanovna - CLI для управления бюджетом. Используйте --help для справки."
            );
        }
        Commands::Tui => {
            if let Err(e) = run_tui(&fs_repo) {
                eprintln!("Ошибка TUI: {e}");
            }
            let elapsed = start.elapsed();
            println!("⏱️ Время сеанса: {elapsed:.2?}");
            return Ok(());
        }
    }
    let elapsed = start.elapsed();
    println!("⏱️ Выполенено за: {elapsed:.2?}");
    Ok(())
}
