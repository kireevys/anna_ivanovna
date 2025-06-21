use crate::distribute::Budget;
use crate::finance::Money;
use crate::planning::{
    Draft, Error as PlanningError, Expense as DomainExpense, ExpenseValue, IncomeSource, Plan,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

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
        eprintln!("Невозможно прочитать файл: {e}");
        Error::CantReadPlan
    })?;
    let root: Root = serde_yaml::from_str(&yaml_data).map_err(|e| {
        eprintln!("Невозможно спарсить файл: {e}");
        Error::CantParsePlan
    })?;
    yaml_to_domain(root.plan).map_err(|e| {
        eprintln!("Невозможно преобразовать файл: {e:?}");
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
        eprintln!("Невозможно прочитать файл: {e}");
        Error::CantReadDistribute
    })?;

    // Парсим JSON и переформатируем его для сравнения
    let value: serde_json::Value = serde_json::from_str(&json_data).map_err(|e| {
        eprintln!("Невозможно спарсить JSON файл: {e}");
        Error::CantParseDistribute
    })?;

    // Возвращаем отформатированную строку
    serde_json::from_value(value).map_err(|e| {
        eprintln!("Невозможно сериализовать JSON: {e}");
        Error::CantParseDistribute
    })
}

#[cfg(test)]
mod tests {
    use crate::distribute::{Income, distribute};
    use crate::finance::Money;
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
