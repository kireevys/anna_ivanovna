use crate::interfaces::{
    presentation::{budget_to_tree, plan_to_tree},
    tree::to_text,
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
use tracing::{self, info};

#[derive(Parser, Debug)]
#[clap(
    name = "Anna Ivanovna",
    version = env!("CARGO_PKG_VERSION"),
    author = "github.com/kireevys",
    about = "Планировщик бюджета - автоматическое распределение доходов по статьям расходов",
    long_about = "Anna Ivanovna помогает автоматически распределять ваши доходы по заранее составленному плану бюджета.\n\nСоздайте план один раз, и программа будет автоматически рассчитывать, сколько денег тратить на каждую категорию при получении дохода.\n\nПримеры:\n  anna_ivanovna plan               # Показать текущий план\n  anna_ivanovna income 50000       # Добавить доход 50000₽\n  anna_ivanovna web 0.0.0.0 8080   # Запустить web-интерфейс"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(flatten)]
    Budget(BudgetCommand),

    /// Запустить web-интерфейс
    Web { host: String, port: u16 },

    /// Миграция данных в SQLite
    Migrate {
        #[clap(subcommand)]
        source: MigrateSource,
    },
}

#[derive(Subcommand, Debug)]
pub enum BudgetCommand {
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
    ShowBudget { id: String },
}

#[derive(Subcommand, Debug)]
pub enum MigrateSource {
    /// Из файловой системы
    Fs,
    /// Из Excel CSV
    Excel {
        #[clap(long)]
        file: PathBuf,
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

#[tracing::instrument(skip(api, cmd))]
pub fn run<R>(api: CoreApi<R>, cmd: BudgetCommand) -> Result<(), Error>
where
    R: CoreRepo + Clone + Send + Sync + 'static,
{
    info!(location = api.location());
    let plan = api.get_plan().ok_or(Error::NoPlan)?;
    let weights = plan.try_into().map_err(|_| Error::InvalidPlan)?;
    let start = std::time::Instant::now();
    match cmd {
        BudgetCommand::AddIncome { amount, dry_run } => {
            let source = choose_source(&weights)?;
            let income = Income::new_today(source.clone(), Money::new_rub(amount));
            let budget = api
                .distribute(&weights, &income)
                .map_err(|_| Error::CantDistribute)?;

            let tree = budget_to_tree(&budget);
            println!("{}", to_text(&tree));
            let id: BudgetId = CoreApi::<R>::build_budget_id();
            if dry_run {
                println!("🔍 DRY-RUN: Результат НЕ сохранён");
            } else {
                let id = api
                    .save_budget(id, budget)
                    .map_err(|_| Error::CantWriteResult)?;
                println!("💾 Бюджет сохранён с ID: {id}");
            }
        }
        BudgetCommand::Plan => {
            let tree = plan_to_tree(&weights);
            println!("{}", to_text(&tree));
        }
        BudgetCommand::ShowBudget { id } => match api.budget_by_id(&id) {
            Some(budget) => {
                let tree = budget_to_tree(&budget.budget);
                println!("{}", to_text(&tree));
            }
            None => eprintln!("❌ Ошибка: не удалось загрузить бюджет с ID {id}"),
        },
    }
    let elapsed = start.elapsed();
    println!("⏱️ Время выполнения: {elapsed:.2?}");
    Ok(())
}
