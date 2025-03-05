use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::finance::{Money, Percentage};
use crate::planning::{Expense, IncomeSource, Plan};
use chrono::{NaiveDate, Utc};
use serde;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_yaml::Mapping;

#[derive(PartialEq, Debug)]
pub enum Error {
    EmptyPlan,
    UnknownSource,
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Income {
    #[serde(serialize_with = "serialize_income_source_yaml")]
    #[serde(flatten)]
    source: IncomeSource,
    #[serde(serialize_with = "serialize_money_yaml")]
    #[serde(flatten)]
    money: Money,
    date: NaiveDate,
}

fn serialize_income_source_yaml<S>(source: &IncomeSource, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("IncomeSource", 2)?;
    state.serialize_field("source", &source.name)?;
    state.end()
}

impl Display for Income {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} от {}", &self.money, &self.date)
    }
}

impl Income {
    #[must_use]
    pub fn new(source: IncomeSource, money: Money, date: NaiveDate) -> Self {
        Self {
            source,
            money,
            date,
        }
    }
    #[must_use]
    pub fn new_today(source: IncomeSource, money: Money) -> Self {
        Self {
            source,
            money,
            date: Utc::now().date_naive(),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Distribute {
    #[serde(flatten)]
    income: Income,
    #[serde(serialize_with = "serialize_rest_yaml")]
    #[serde(flatten)]
    rest: Money,
    #[serde(serialize_with = "serialize_expenditures_yaml")]
    expenditures: HashMap<Expense, Money>,
}
fn serialize_rest_yaml<S>(money: &Money, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("Money", 2)?;
    state.serialize_field("rest", &money.to_string())?;
    state.end()
}

fn serialize_money_yaml<S>(money: &Money, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("Money", 2)?;
    state.serialize_field("money", &money.to_string())?;
    state.end()
}

fn _serialize_expenditures_json<S>(
    map: &HashMap<Expense, Money>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vec: Vec<_> = map
        .iter()
        .map(|(key, value)| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "name".to_string(),
                serde_json::Value::String(key.name.clone()),
            );
            // let value = format!("{}{}", value.currency.clone(), value.value.clone());
            obj.insert(
                "money".to_string(),
                serde_json::Value::String(value.to_string()),
            );
            serde_json::Value::Object(obj)
        })
        .collect();

    vec.serialize(serializer)
}

fn serialize_expenditures_yaml<S>(
    map: &HashMap<Expense, Money>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let vec: Vec<_> = map
        .iter()
        .map(|(key, value)| {
            let mut obj = Mapping::new();
            obj.insert(
                serde_yaml::Value::String("name".to_string()),
                serde_yaml::Value::String(key.clone().name),
            );
            obj.insert(
                serde_yaml::Value::String("money".to_string()),
                serde_yaml::Value::String(value.to_string()),
            );
            obj
        })
        .collect();

    vec.serialize(serializer)
}

impl Distribute {
    fn new(income: Income, expenditures: HashMap<Expense, Money>) -> Self {
        Self {
            rest: income.money,
            income,
            expenditures,
        }
    }

    fn calculate(&mut self, expense: Expense, rate: &Percentage) {
        let money = Money::new_rub(rate.apply_to(self.income.money.value));
        self.expenditures.insert(expense, money);
        self.rest -= money;
    }
}

impl Display for Distribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        let mut sorted: Vec<(_, _)> = self.clone().expenditures.into_iter().collect();
        sorted.sort_by_key(|(key, _)| key.name.clone());
        for (e, v) in sorted {
            let row = format!("- {:20} - {:}\n", e.name, v);
            result.push_str(row.as_str());
        }
        write!(
            f,
            "\nРаспределение дохода {}По источнику {}\n{result}- {:20} - {:}",
            &self.income, &self.income.source, "Остаток", self.rest
        )
    }
}

/// Функция занимается распределением Дохода согласно Плана
///
/// # Arguments
///
/// * `plan`: Запланированный бюджет
/// * `income`: Полученный доход
///
/// # Errors
/// `UnknownSource` - план не содержит Источника полученного Дохода
///
/// returns: Result<Distribute, `DistributeError`>
///
pub fn distribute(plan: &Plan, income: &Income) -> Result<Distribute, Error> {
    if !&plan.has_source(&income.source) {
        return Err(Error::UnknownSource);
    }
    let result = HashMap::with_capacity(plan.len());

    let mut d = Distribute::new(income.clone(), result.clone());

    plan.into_iter()
        .for_each(|(e, r)| d.calculate(e.clone(), r));
    Ok(d)
}

#[cfg(test)]
mod test_distribute {
    use std::collections::HashMap;

    use chrono::Utc;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;

    use crate::distribute::{distribute, Distribute, Error, Income};
    use crate::finance::{Money, Percentage};
    use crate::planning::{Draft, Expense, ExpenseValue, IncomeSource, Plan};

    fn rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn income_from_unknown_source() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let source_1 = IncomeSource::new("Unknown".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.5) },
        );
        let draft = Draft::build(&[source], &[expense]);
        let plan = Plan::try_from(draft).unwrap();
        let income = Income::new(source_1, rub(1.0), Utc::now().date_naive());
        assert_eq!(distribute(&plan, &income), Err(Error::UnknownSource));
    }

    #[test]
    fn expense_is_money_and_less_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.5) },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let plan = Plan::try_from(draft).unwrap();
        let income = Income::new_today(source, rub(1.0));
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), rub(0.5))]),
                rest: rub(0.5),
            })
        );
    }

    #[test]
    fn expense_is_money_and_more_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(1.0) },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(0.5));
        let plan = Plan::try_from(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), rub(0.5))]),
                rest: rub(0.0),
            })
        );
    }

    #[test]
    fn expense_is_full_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), rub(1.0))]),
                rest: rub(0.0),
            })
        );
    }

    #[test]
    fn expense_is_half_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(50),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), rub(0.5))]),
                rest: rub(0.5),
            })
        );
    }

    #[test]
    fn expense_is_zero_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(0),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), rub(0.0))]),
                rest: rub(1.0),
            })
        );
    }

    #[test]
    fn expense_is_one_percent_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(1),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), rub(0.01))]),
                rest: rub(0.99),
            })
        );
    }
}
