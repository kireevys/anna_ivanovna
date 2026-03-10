use crate::{
    interfaces,
    interfaces::{
        presentation::{budget_to_tree, plan_to_tree},
        tree::to_text,
    },
    storage::FileSystem,
};
use ai_core::{
    api::{BudgetId, CoreApi, CoreRepo},
    distribute::Income,
    finance::Money,
    planning::{DistributionWeights, IncomeSource},
};
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::{io, io::Write, path::PathBuf};
use thiserror::Error;
use tokio::runtime::Runtime;
use tracing::{self, info};

#[derive(Parser, Debug)]
#[clap(
    name = "Anna Ivanovna",
    version = env!("CARGO_PKG_VERSION"),
    author = "github.com/kireevys",
    about = "Планировщик бюджета - автоматическое распределение доходов по статьям расходов",
    long_about = "Anna Ivanovna помогает автоматически распределять ваши доходы по заранее составленному плану бюджета.\n\nСоздайте план один раз, и программа будет автоматически рассчитывать, сколько денег тратить на каждую категорию при получении дохода.\n\nПримеры:\n  anna_ivanovna prepare-storage    # Подготовить папки для работы\n  anna_ivanovna plan               # Показать текущий план\n  anna_ivanovna income 50000       # Добавить доход 50000₽"
)]
pub struct Cli {
    /// Подкоманда для работы с финансами
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Добавить доход и распределить его согласно плану
    #[clap(alias = "income")]
    AddIncome {
        amount: Decimal,
        #[clap(long)]
        dry_run: bool,
    },

    /// Отобразить текущий план бюджета
    Plan,

    /// Показать бюджет по id
    #[clap(alias = "show")]
    ShowBudget {
        id: String,
    },

    /// Подготовить структуру папок и файлов для работы
    #[clap(alias = "prepare")]
    PrepareStorage {
        path: PathBuf,
    },

    /// Спарсить Excel-совместимый CSV и сохранить json-файлы
    ParseExcel {
        #[clap(long)]
        file: PathBuf,
    },
    Web {
        host: String,
        port: u16,
    },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("План бюджета не найден")]
    NoPlan,
    #[error("Не удалось записать результат")]
    CantWriteResult,
    #[error("Неверный ввод")]
    InvalidInput,
    #[error("Не удалось распределить бюджет")]
    CantDistribute,
    #[error("Не удалось построить план распределения бюджета")]
    InvalidPlan,
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

#[tracing::instrument(skip(plan))]
fn choose_source(plan: &DistributionWeights) -> Result<&IncomeSource, Error> {
    if plan.sources.len() == 1 {
        return plan.sources.first().ok_or(Error::NoPlan);
    }
    println!("В бюджете указано несколько источников дохода:");
    for (n, i) in plan.sources.iter().enumerate() {
        println!("  {n}: {} [{}]", i.name, i.expected);
    }
    print!("Введите номер источника: ");
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
#[tracing::instrument(skip(api))]
pub fn run<R>(api: CoreApi<R>) -> Result<(), Error>
where
    R: CoreRepo + Clone + Send + Sync + 'static,
{
    info!(location = api.location());
    let cli = Cli::parse();
    let plan = api.get_plan().ok_or(Error::NoPlan)?;
    let weghts = plan.try_into().map_err(|_| Error::InvalidPlan)?;
    let start = std::time::Instant::now();
    match cli.command {
        Commands::AddIncome { amount, dry_run } => {
            let source = choose_source(&weghts)?;
            let income = Income::new_today(source.clone(), Money::new_rub(amount));
            let budget = api
                .distribute(&weghts, &income)
                .map_err(|_| Error::CantDistribute)?;

            let tree = budget_to_tree(&budget);
            println!("{}", to_text(&tree));
            let id: BudgetId = format!(
                "{}-{}.json",
                budget.income_date().format("%Y-%m-%d"),
                budget.income.source.name
            );
            if dry_run {
                println!("🔍 DRY-RUN: Результат НЕ сохранён");
            } else {
                let id = api
                    .save_budget(id, budget)
                    .map_err(|_| Error::CantWriteResult)?;
                println!("💾 Бюджет сохранён с ID: {id}");
            }
        }
        Commands::Plan => {
            let tree = plan_to_tree(&weghts);
            println!("{}", to_text(&tree));
        }
        Commands::ShowBudget { id } => match api.budget_by_id(&id) {
            Some(budget) => {
                let tree = budget_to_tree(&budget.budget);
                println!("{}", to_text(&tree));
            }
            None => eprintln!("❌ Ошибка: не удалось загрузить бюджет с ID {id}"),
        },
        Commands::PrepareStorage { path } => {
            if let Err(e) = FileSystem::init(path) {
                eprintln!("{e}");
            }
        }
        Commands::ParseExcel { file } => {
            match crate::interfaces::excel_parser::parse_excel_csv(file) {
                Ok(budgets) => {
                    let mut count = 0;
                    for b in budgets {
                        api.save_budget(CoreApi::<R>::build_budget_id(&b), b)
                            .map_err(|_| Error::CantWriteResult)?;
                        count += 1;
                    }
                    println!("✅ Успешно обработано {count} строк")
                }
                Err(e) => eprintln!("❌ Ошибка парсинга CSV: {e}"),
            }
        }
        Commands::Web { host, port } => {
            let runtime = Runtime::new().expect("failed to create tokio runtime");
            runtime.block_on(async {
                if let Err(err) =
                    interfaces::web::run(api.clone(), &format!("{host}:{port}")).await
                {
                    eprintln!("Ошибка web-сервера: {err}");
                }
            });
            return Ok(());
        }
    }
    let elapsed = start.elapsed();
    println!("⏱️ Время выполнения: {elapsed:.2?}");
    Ok(())
}
