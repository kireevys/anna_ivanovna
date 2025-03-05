use crate::distribute::Distribute;
use crate::finance::Money;
use crate::planning::{Draft, Error, Expense as DomainExpense, ExpenseValue, IncomeSource, Plan};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

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
}

fn yaml_to_domain(yaml: PlanDetails) -> Result<Plan, Error> {
    let sources = yaml
        .incomes
        .into_iter()
        .map(|i| Money::from_str(i.value.as_str()).map(|v| IncomeSource::new(i.source, v)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_e| Error::InvalidPlan)?;

    let expenses = yaml
        .expenses
        .into_iter()
        .map(|e| ExpenseValue::from_str(e.value.as_str()).map(|v| DomainExpense::new(e.name, v)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_e| Error::InvalidPlan)?;
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
/// # Panics
/// Паникует в любой непонятной ситуации
///
#[must_use]
pub fn plan_from_yaml(path: &Path) -> Plan {
    let yaml_data = fs::read_to_string(path).expect("Unable to read file {path}");
    let root: Root = serde_yaml::from_str(&yaml_data).expect("Failed to parse YAML");
    yaml_to_domain(root.plan).expect("Failed to convert YAML to domain")
}
/// # Panics
/// Паникует когда не удалось собрать yaml
///
#[must_use]
pub fn distribute_to_yaml(distribute: &Distribute) -> String {
    serde_yaml::to_string(distribute).expect("Cant build yaml")
}
#[cfg(test)]
mod tests {
    use crate::finance::Percentage;
    use crate::storage::plan_from_yaml;
    use rust_decimal_macros::dec;
    use std::path::Path;

    #[test]
    fn test_basic_parse() {
        let plan = plan_from_yaml(Path::new("src/test_storage/plan.yaml"));
        assert_eq!(plan.rest, Percentage::from(dec!(0.98)));
    }
}
