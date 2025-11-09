use crate::api::{self, BudgetId, CoreRepo, PlanId};
use crate::core::distribute::Budget;
use crate::core::editor::Plan;
use crate::core::finance::Money;
use crate::core::planning::{
    Error as PlanningError, Expense as DomainExpense, ExpenseValue, IncomeSource,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
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
        .map(|i| Money::from_str(i.value.as_str()).map(|v| IncomeSource::new(i.source, v)))
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
    Plan::build(sources, expenses).map_err(|_| PlanningError::InvalidPlan)
}

fn plan_to_yaml(plan: &Plan) -> Result<String, Error> {
    let plan_details = PlanDetails {
        incomes: plan
            .sources
            .values()
            .map(|s| Income {
                source: s.name.clone(),
                value: format!("{}", s.expected()),
            })
            .collect(),
        expenses: plan
            .expenses
            .values()
            .map(|e| Expense {
                name: e.name.clone(),
                value: match &e.value {
                    ExpenseValue::MONEY { value } => format!("{}", value),
                    ExpenseValue::RATE { value } => format!("{}", value),
                },
                category: e.category.clone(),
            })
            .collect(),
    };
    let root = Root {
        plan: plan_details,
    };
    serde_yaml::to_string(&root).map_err(|e| {
        error!("Невозможно сериализовать план: {e}");
        Error::CantParsePlan
    })
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
#[derive(Debug)]
pub struct FileSystem {
    root_dir: PathBuf,
    plans_path: PathBuf,
    incomes_path: PathBuf,
}

impl FileSystem {
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
        std::fs::create_dir_all(buh_dir).map_err(|e| format!("Ошибка создания директории: {e}"))?;
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
                info!("Создана директория plans для миграции: {}", fs.plans_path().display());
            }
            // Создаем incomes, если его нет
            if !fs.incomes_path().exists() {
                std::fs::create_dir_all(fs.incomes_path())
                    .map_err(|e| format!("Ошибка создания incomes: {e}"))?;
                info!("Создана директория incomes: {}", fs.incomes_path().display());
            }

            // Миграция: если plans пуст, но есть старый plan.yaml, копируем его
            let old_plan_path = fs.root_dir.join("plan.yaml");
            if old_plan_path.exists() && fs.get_latest_plan_path().is_none() {
                let migration_date = chrono::Utc::now().format("%Y-%m-%d");
                let new_plan_path = fs.plans_path().join(format!("{}.yaml", migration_date));
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
        let mut files: Vec<_> = match std::fs::read_dir(self.incomes_path()) {
            Ok(rd) => rd
                .filter_map(|e| {
                    let path = e.ok()?.path();
                    if path.extension().is_some_and(|ext| ext == "json") {
                        // Возвращаем имя файла как BudgetId, а не PathBuf
                        path.file_name()
                            .map(|os_str| os_str.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
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
    fn location(&self) -> &str {
        self.root().to_str().unwrap_or_default()
    }

    #[instrument(skip(self))]
    fn get_plan(&self) -> Option<Plan> {
        self.get_latest_plan_path()
            .and_then(|path| plan_from_yaml(&path).ok())
    }

    #[instrument(skip(self, plan))]
    fn save_plan(&self, plan: Plan, id: PlanId) -> Result<PlanId, api::Error> {
        let yaml_content = plan_to_yaml(&plan).map_err(|_| api::Error::CantSaveBudget)?;
        let plan_path = self.plans_path().join(&id);
        let mut file = File::create(&plan_path).map_err(|_| api::Error::CantSaveBudget)?;
        file.write_all(yaml_content.as_bytes())
            .map_err(|_| api::Error::CantSaveBudget)?;
        info!("План сохранен в {plan_path:?}");
        Ok(id)
    }

    #[instrument(skip(self, budget))]
    fn save_budget(&self, budget: Budget) -> Result<BudgetId, api::Error> {
        let filename: BudgetId = format!(
            "{}-{}.json",
            budget.income_date().format("%Y-%m-%d"),
            budget.income.source.name
        );
        let result_path = &self.incomes_path().join(&filename);
        let mut file = File::create(result_path).map_err(|_| api::Error::CantSaveBudget)?;

        let json_result =
            serde_json::to_string_pretty(&budget).map_err(|_| api::Error::CantSaveBudget)?;
        file.write_all(json_result.as_bytes())
            .map_err(|_| api::Error::CantSaveBudget)?;

        info!("Записано в {result_path:?}");
        Ok(filename)
    }

    #[instrument(skip(self))]
    fn budget_ids<'r>(
        &'r self,
        from: Option<api::Cursor>,
        limit: usize,
    ) -> Box<dyn Iterator<Item = BudgetId> + 'r> {
        let files: Vec<_> = self.full_storage().collect();
        let start = from
            .as_ref()
            .and_then(|cursor| files.iter().position(|p| p == cursor))
            .map_or(0, |idx| idx + 1);
        let files: Vec<_> = files.into_iter().skip(start).take(limit).collect();
        Box::new(files.into_iter())
    }

    #[instrument(skip(self))]
    fn budget_by_id(&self, id: &BudgetId) -> Option<api::StorageBudget> {
        let path = self.incomes_path().join(id);
        match distribute_from_json(&path) {
            Ok(budget) => {
                info!(id = id);
                Some(api::StorageBudget {
                    id: id.clone(),
                    budget,
                })
            }
            Err(e) => {
                error!("[anna_ivanovna] WARNING: Не удалось загрузить бюджет из {id}: {e:?}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::distribute::{Income, distribute};
    use crate::core::finance::Money;
    use crate::core::planning::DistributionWeights;
    use crate::storage::{distribute_from_json, plan_from_yaml};
    use chrono::NaiveDate;
    use std::path::Path;

    #[test]
    fn test_e2e() {
        let plan = plan_from_yaml(Path::new("src/test_storage/plan.yaml")).unwrap();
        let (_, source) = plan.sources.first_key_value().unwrap();

        let income = Income::new(
            source.clone(),
            Money::new_rub((source.expected.value / rust_decimal::Decimal::from(2)).round_dp(2)),
            NaiveDate::from_ymd_opt(2025, 6, 21).unwrap(),
        );
        let weights: DistributionWeights = plan.try_into().unwrap();
        let result = distribute(&weights, &income).unwrap();

        let expected = distribute_from_json(Path::new("src/test_storage/result.json")).unwrap();
        assert_eq!(result, expected);
    }
}
