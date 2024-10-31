use crate::finance::Money;
use crate::planning::{Draft, Error, Expense as DomainExpense, ExpenseValue, IncomeSource, Plan};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

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
    pub id: Uuid,
    pub source: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Expense {
    pub id: Uuid,
    pub name: String,
    pub value: String,
}

fn yaml_to_domain(yaml: PlanDetails) -> Result<Plan, Error> {
    let mut sources = Vec::new();
    for i in yaml.incomes {
        let value = Money::from_str(i.value.as_str()).expect("failed to parse Money");
        let source = IncomeSource::build(i.id, i.source, value);
        sources.push(source);
    }
    let mut expenses = Vec::new();
    for e in yaml.expenses {
        let value = ExpenseValue::from_str(e.value.as_str()).expect("could not parse expense");
        let expense = DomainExpense::build(e.id, e.name, value);
        expenses.push(expense);
    }
    let draft = Draft::build(&sources, &expenses);
    Plan::from_draft(draft)
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
