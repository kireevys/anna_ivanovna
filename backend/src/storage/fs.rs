use ai_app::storage::{
    BudgetId,
    CoreRepo,
    Cursor,
    Page,
    PlanEvent,
    PlanId,
    PlanStatus,
    StorageBudget,
    StorageError,
    StoragePlan,
    UserId,
};
use ai_core::{
    distribute::Budget,
    finance::Money,
    plan::Plan,
    planning::{
        Error as PlanningError,
        Expense as DomainExpense,
        ExpenseValue,
        IncomeSource,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::{error, info, instrument};
#[derive(Debug)]
pub enum Error {
    CantReadPlan,
    CantParsePlan,
    PlanNotAdaptable,
    CantReadDistribute,
    CantParseDistribute,
}

#[derive(Serialize, Deserialize)]
struct Root {
    pub plan: PlanDetails,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanDetails {
    pub incomes: Vec<Income>,
    pub expenses: Vec<Expense>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Income {
    pub source: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Expense {
    pub name: String,
    pub value: String,
    pub category: Option<String>,
}

fn yaml_to_domain(yaml: PlanDetails) -> Result<Plan, PlanningError> {
    let sources = yaml
        .incomes
        .into_iter()
        .map(|i| {
            Money::from_str(i.value.as_str()).map(|v| IncomeSource::new(i.source, v))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_e| PlanningError::InvalidPlan)?;

    let expenses = yaml
        .expenses
        .into_iter()
        .map(|e| {
            ExpenseValue::from_str(e.value.as_str())
                .map(|v| DomainExpense::new(e.name, v, e.category))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_e| PlanningError::InvalidPlan)?;
    Ok(Plan::build(&sources, &expenses))
}

/// Парсит переданный файл в Бюджет
///
/// # Arguments
///
/// * `path`: Путь к файлу
///
/// returns: Plan
///
/// # Errors
/// - `CantReadPlan` - Проблема чтения файла
/// - `CantParsePlan` - Проблема парсинга файла
/// - `PlanNotAdaptable` - Проблема конвертации в доменный объект
///
pub fn plan_from_yaml(path: &Path) -> Result<Plan, Error> {
    let yaml_data = fs::read_to_string(path).map_err(|e| {
        error!("Невозможно прочитать файл: {e}");
        Error::CantReadPlan
    })?;
    let root: Root = serde_yaml::from_str(&yaml_data).map_err(|e| {
        error!("Невозможно спарсить файл: {e}");
        Error::CantParsePlan
    })?;
    yaml_to_domain(root.plan).map_err(|e| {
        error!("Невозможно преобразовать файл: {e:?}");
        Error::PlanNotAdaptable
    })
}

/// Читает JSON файл с Бюджетом и возвращает его
///
/// # Arguments
///
/// * `path`: Путь к JSON файлу
///
/// returns: Budget
///
/// # Errors
/// - `CantReadDistribute` - Проблема чтения файла
/// - `CantParseDistribute` - Проблема парсинга файла
///
pub fn distribute_from_json(path: &Path) -> Result<Budget, Error> {
    let json_data = fs::read_to_string(path).map_err(|e| {
        error!("Невозможно прочитать файл: {e}");
        Error::CantReadDistribute
    })?;

    // Парсим JSON и переформатируем его для сравнения
    serde_json::from_str(&json_data).map_err(|e| {
        error!("Невозможно спарсить JSON файл {:?}: {e}", path.file_name());
        Error::CantParseDistribute
    })
}
#[derive(Debug, Clone)]
pub struct FileSystem {
    root_dir: PathBuf,
    plans_path: PathBuf,
    incomes_path: PathBuf,
}

impl FileSystem {
    #[allow(dead_code)]
    fn root(&self) -> &PathBuf {
        &self.root_dir
    }

    fn plans_path(&self) -> &PathBuf {
        &self.plans_path
    }

    fn incomes_path(&self) -> &PathBuf {
        &self.incomes_path
    }

    /// Подготавливает структуру хранилища (директории, файлы)
    fn prepare_storage(&self) -> Result<(), String> {
        let buh_dir = &self.root_dir;
        info!("Хранилище не найдено, инициализирую: {}", buh_dir.display());
        if buh_dir.exists() {
            return Err(format!(
                "❗️ Хранилище уже инициализировано: {}",
                buh_dir.display()
            ));
        }
        std::fs::create_dir_all(buh_dir)
            .map_err(|e| format!("Ошибка создания директории: {e}"))?;
        info!("Создана директория: {}", buh_dir.display());
        let incomes_path = self.incomes_path();
        std::fs::create_dir_all(incomes_path)
            .map_err(|e| format!("Ошибка создания incomes: {e}"))?;
        info!("Создана директория: {}", incomes_path.display());
        let plans_path = self.plans_path();
        std::fs::create_dir_all(plans_path)
            .map_err(|e| format!("Ошибка создания plans: {e}"))?;
        info!("Создана директория: {}", plans_path.display());
        info!("Хранилище инициализировано: {}", buh_dir.display());
        Ok(())
    }

    /// Инициализирует хранилище, если оно не инициализировано, и возвращает FileSystem
    #[instrument]
    pub fn init<P: AsRef<Path> + std::fmt::Debug>(root_dir: P) -> Result<Self, String> {
        let root_dir = root_dir.as_ref().to_path_buf();
        let fs = Self {
            root_dir: root_dir.clone(),
            plans_path: root_dir.join("plans"),
            incomes_path: root_dir.join("incomes"),
        };

        // Если хранилище уже существует, проверяем и создаем недостающие директории
        if fs.root_dir.exists() {
            // Создаем plans, если его нет (миграция со старой структуры)
            if !fs.plans_path().exists() {
                std::fs::create_dir_all(fs.plans_path())
                    .map_err(|e| format!("Ошибка создания plans: {e}"))?;
                info!(
                    "Создана директория plans для миграции: {}",
                    fs.plans_path().display()
                );
            }
            // Создаем incomes, если его нет
            if !fs.incomes_path().exists() {
                std::fs::create_dir_all(fs.incomes_path())
                    .map_err(|e| format!("Ошибка создания incomes: {e}"))?;
                info!(
                    "Создана директория incomes: {}",
                    fs.incomes_path().display()
                );
            }

            // Миграция: если plans пуст, но есть старый plan.yaml, копируем его
            let old_plan_path = fs.root_dir.join("plan.yaml");
            if old_plan_path.exists() && fs.get_latest_plan_path().is_none() {
                let migration_date = chrono::Utc::now().format("%Y-%m-%d");
                let new_plan_path =
                    fs.plans_path().join(format!("{}.yaml", migration_date));
                std::fs::copy(&old_plan_path, &new_plan_path)
                    .map_err(|e| format!("Ошибка миграции plan.yaml: {e}"))?;
                info!("Мигрирован plan.yaml → {}", new_plan_path.display());
            }

            return Ok(fs);
        }

        // Иначе инициализируем с нуля
        fs.prepare_storage()?;
        Ok(fs)
    }

    fn full_storage(&self) -> impl Iterator<Item = BudgetId> {
        let mut files: Vec<BudgetId> = match std::fs::read_dir(self.incomes_path()) {
            Ok(rd) => rd
                .filter_map(|f| f.ok())
                .filter(|path| path.path().extension().is_some_and(|ext| ext == "json"))
                .map(|path| path.file_name().to_string_lossy().to_string())
                .collect(),
            Err(_) => Vec::new(),
        };
        files.sort_by(|a, b| b.cmp(a));
        files.into_iter()
    }

    fn get_latest_plan_path(&self) -> Option<PathBuf> {
        let mut files: Vec<_> = match std::fs::read_dir(self.plans_path()) {
            Ok(rd) => rd
                .filter_map(|e| {
                    let path = e.ok()?.path();
                    if path.extension().is_some_and(|ext| ext == "yaml") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        files.sort_by(|a, b| {
            b.file_name()
                .and_then(|n| n.to_str())
                .cmp(&a.file_name().and_then(|n| n.to_str()))
        });
        files.into_iter().next()
    }
}

impl CoreRepo for FileSystem {
    #[instrument(skip(self))]
    fn get_plan(&self, user_id: &UserId) -> Option<StoragePlan> {
        self.get_latest_plan_path()
            .and_then(|path| plan_from_yaml(&path).ok())
            .map(|plan| StoragePlan {
                user_id: user_id.clone(),
                id: String::new(),
                plan,
                version: 0,
                status: PlanStatus::Active,
            })
    }

    fn create_plan(
        &self,
        _user_id: &UserId,
        _plan_id: PlanId,
        _plan: Plan,
    ) -> Result<PlanId, StorageError> {
        unimplemented!("FileSystem — read-only source для миграции")
    }

    fn update_plan(
        &self,
        _user_id: &UserId,
        _plan_id: &PlanId,
        _plan: Plan,
    ) -> Result<(), StorageError> {
        unimplemented!("FileSystem — read-only source для миграции")
    }

    fn delete_plan(
        &self,
        _user_id: &UserId,
        _plan_id: &PlanId,
    ) -> Result<(), StorageError> {
        unimplemented!("FileSystem — read-only source для миграции")
    }

    fn plan_events(
        &self,
        _user_id: &UserId,
        _plan_id: &PlanId,
        _from: Option<Cursor>,
        _limit: usize,
    ) -> Page<PlanEvent> {
        unimplemented!("FileSystem — read-only source для миграции")
    }

    #[instrument(skip(self, budget))]
    fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> Result<BudgetId, StorageError> {
        let result_path = &self.incomes_path().join(&budget_id);
        let mut file =
            File::create(result_path).map_err(|_| StorageError::SaveBudget)?;

        let json_result = serde_json::to_string_pretty(&budget)
            .map_err(|_| StorageError::SaveBudget)?;
        file.write_all(json_result.as_bytes())
            .map_err(|_| StorageError::SaveBudget)?;

        info!("Записано в {result_path:?}");
        Ok(budget_id)
    }

    #[instrument(skip(self))]
    fn budgets(
        &self,
        from: Option<ai_app::storage::Cursor>,
        limit: usize,
    ) -> Page<StorageBudget> {
        let files = self
            .full_storage()
            .skip_while(|id| from.as_ref().is_some_and(|cursor| cursor <= id));

        let items: Vec<StorageBudget> = files
            .take(limit)
            .filter_map(|id| self.budget_by_id(&id))
            .collect();
        let next_cursor = items.last().map(|b| b.id.clone());
        Page::new(items, next_cursor)
    }

    #[instrument(skip(self))]
    fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget> {
        let path = self.incomes_path().join(id);
        match distribute_from_json(&path) {
            Ok(budget) => {
                info!(id = id);
                Some(StorageBudget {
                    id: id.clone(),
                    budget,
                })
            }
            Err(e) => {
                error!(
                    "[anna_ivanovna] WARNING: Не удалось загрузить бюджет из {id}: {e:?}"
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::fs::{distribute_from_json, plan_from_yaml};
    use ai_core::{
        distribute::{Income, distribute},
        finance::Money,
        planning::DistributionWeights,
    };
    use chrono::NaiveDate;
    use std::path::Path;

    #[test]
    fn test_e2e() {
        let plan = plan_from_yaml(Path::new("src/test_storage/plan.yaml")).unwrap();
        let source = plan.sources.first().unwrap();

        let income = Income::new(
            source.clone(),
            Money::new_rub(
                (source.expected.value / rust_decimal::Decimal::from(2)).round_dp(2),
            ),
            NaiveDate::from_ymd_opt(2025, 6, 21).unwrap(),
        );
        let weights: DistributionWeights = plan.try_into().unwrap();
        let result = distribute(&weights, &income).unwrap();

        let expected =
            distribute_from_json(Path::new("src/test_storage/result.json")).unwrap();
        assert_eq!(result, expected);
    }
}
