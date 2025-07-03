use crate::api::{self, BudgetId, CoreRepo};
use crate::core::distribute::Budget;
use crate::core::finance::Money;
use crate::core::planning::{
    Draft, Error as PlanningError, Expense as DomainExpense, ExpenseValue, IncomeSource, Plan,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::{error, info};
#[derive(Debug)]
pub enum Error {
    CantReadPlan,
    CantParsePlan,
    PlanNotAdaptable,
    CantReadDistribute,
    CantParseDistribute,
}

#[derive(Deserialize)]
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
    Plan::try_from(Draft::build(&sources, &expenses))
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
    let value: serde_json::Value = serde_json::from_str(&json_data).map_err(|e| {
        error!("Невозможно спарсить JSON файл {:?}: {e}", path.file_name());
        Error::CantParseDistribute
    })?;

    // Возвращаем отформатированную строку
    serde_json::from_value(value).map_err(|e| {
        error!("Невозможно сериализовать JSON: {e}");
        Error::CantParseDistribute
    })
}
#[derive(Debug)]
pub struct FileSystem {
    root_dir: PathBuf,
    plan_path: PathBuf,
    incomes_path: PathBuf,
}

impl FileSystem {
    const DEFAULT_PLAN_CONTENT: &'static str = include_str!("../example/plan.yaml");
    fn root(&self) -> &PathBuf {
        &self.root_dir
    }

    fn plan_path(&self) -> &PathBuf {
        &self.plan_path
    }

    fn incomes_path(&self) -> &PathBuf {
        &self.incomes_path
    }

    /// Подготавливает структуру хранилища (директории, файлы)
    fn prepare_storage(
        &self,
        default_plan_content: &str,
        example_plan_path: &Path,
    ) -> Result<(), String> {
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
        let plan_p = self.plan_path();
        std::fs::create_dir_all(incomes_path)
            .map_err(|e| format!("Ошибка создания incomes: {e}"))?;
        info!("Создана директория: {}", incomes_path.display());
        if !plan_p.exists() {
            if example_plan_path.exists() {
                std::fs::copy(example_plan_path, plan_p)
                    .map_err(|e| format!("Ошибка копирования example/plan.yaml: {e}"))?;
                info!(
                    "Скопирован пример плана: {} → {}",
                    example_plan_path.display(),
                    plan_p.display()
                );
                info!(
                    "Перейдите к этому файлу и отредактируйте его под себя перед использованием!"
                );
            } else {
                std::fs::write(plan_p, default_plan_content)
                    .map_err(|e| format!("Ошибка создания plan.yaml: {e}"))?;
                info!("Создан файл плана с примером: {}", plan_p.display());
                info!("Перейдите к этому файлу и заполните его перед использованием!");
            }
        }
        info!("Хранилище инициализировано: {}", buh_dir.display());
        Ok(())
    }

    /// Инициализирует хранилище, если оно не инициализировано, и возвращает FileSystem
    pub fn init<P: AsRef<Path>>(root_dir: P) -> Result<Self, String> {
        let root_dir = root_dir.as_ref().to_path_buf();
        let fs = Self {
            root_dir: root_dir.clone(),
            plan_path: root_dir.join("plan.yaml"),
            incomes_path: root_dir.join("incomes"),
        };
        // Если уже инициализировано, просто возвращаем
        if fs.root_dir.exists() && fs.plan_path().exists() && fs.incomes_path().exists() {
            return Ok(fs);
        }
        // Иначе инициализируем
        let example_plan_path = Path::new("example/plan.yaml");
        fs.prepare_storage(Self::DEFAULT_PLAN_CONTENT, example_plan_path)?;
        info!(fs = ?fs);
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
}

impl CoreRepo for FileSystem {
    fn location(&self) -> &str {
        self.root().to_str().unwrap_or_default()
    }

    fn get_plan(&self) -> Option<Plan> {
        plan_from_yaml(self.plan_path()).ok()
    }

    fn save_budget(&self, budget: Budget) -> Result<BudgetId, api::Error> {
        let filename: BudgetId = format!("{}.json", Local::now().format("%Y-%m-%d"));
        let result_path = &self.incomes_path().join(&filename);
        let mut file = File::create(result_path).map_err(|_| api::Error::CantSaveBudget)?;

        let json_result =
            serde_json::to_string_pretty(&budget).map_err(|_| api::Error::CantSaveBudget)?;
        file.write_all(json_result.as_bytes())
            .map_err(|_| api::Error::CantSaveBudget)?;

        info!("Записано в {result_path:?}");
        Ok(filename)
    }

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

    fn budget_by_id(&self, id: &BudgetId) -> Option<api::StorageBudget> {
        let path = self.incomes_path().join(id);
        match distribute_from_json(&path) {
            Ok(budget) => {
                info!(budget = ?budget, id = id);
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
    use crate::storage::{distribute_from_json, plan_from_yaml};
    use chrono::NaiveDate;
    use std::path::Path;

    #[test]
    fn test_e2e() {
        let plan = plan_from_yaml(Path::new("src/test_storage/plan.yaml")).unwrap();
        let source = plan.sources.first().unwrap();

        let income = Income::new(
            source.clone(),
            Money::new_rub((source.expected.value / rust_decimal::Decimal::from(2)).round_dp(2)),
            NaiveDate::from_ymd_opt(2025, 6, 21).unwrap(),
        );
        let result = distribute(&plan, &income).unwrap();

        let expected = distribute_from_json(Path::new("src/test_storage/result.json")).unwrap();
        assert_eq!(result, expected);
    }
}
