use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use chrono::{NaiveDate, Utc};

use crate::finance::{Money, Percentage};
use crate::planning::{Expense, IncomeSource, Plan};

#[derive(PartialEq, Debug)]
pub enum Error {
    EmptyPlan,
    UnknownSource,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Income {
    source: IncomeSource,
    money: Money,
    date: NaiveDate,
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

#[derive(PartialEq, Debug, Clone)]
pub struct Distribute {
    income: Income,
    rest: Money,
    expenditures: HashMap<Expense, Money>,
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
        let mut result = format!(
            "Распределение дохода {} от {}\n",
            &self.income.money, &self.income.date
        );
        for (k, v) in &self.expenditures {
            let row = format!("{:20} - {:}\n", k.name, v);
            result.push_str(row.as_str());
        }
        result.push_str(format!("{:20} - {:}", "Остаток", self.rest).as_str());
        write!(f, "{result}")
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

    plan.into_iter().for_each(|(e, r)| d.calculate(e.clone(), r));
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

    fn _rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn income_from_unknown_source() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let source_1 = IncomeSource::new("Unknown".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: _rub(0.5) },
        );
        let draft = Draft::build(&[source], &[expense]);
        let plan = Plan::from_draft(draft).unwrap();
        let income = Income::new(source_1, _rub(1.0), Utc::now().date_naive());
        assert_eq!(
            distribute(&plan, &income),
            Err(Error::UnknownSource)
        );
    }

    #[test]
    fn expense_is_money_and_less_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: _rub(0.5) },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let plan = Plan::from_draft(draft).unwrap();
        let income = Income::new_today(source, _rub(1.0));
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(0.5))]),
                rest: _rub(0.5),
            })
        );
    }

    #[test]
    fn expense_is_money_and_more_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: _rub(1.0) },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, _rub(0.5));
        let plan = Plan::from_draft(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(0.5))]),
                rest: _rub(0.0),
            })
        );
    }

    #[test]
    fn expense_is_full_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, _rub(1.0));
        let plan = Plan::from_draft(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(1.0))]),
                rest: _rub(0.0),
            })
        );
    }

    #[test]
    fn expense_is_half_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(50),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, _rub(1.0));
        let plan = Plan::from_draft(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(0.5))]),
                rest: _rub(0.5),
            })
        );
    }

    #[test]
    fn expense_is_zero_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(0),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, _rub(1.0));
        let plan = Plan::from_draft(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(0.0))]),
                rest: _rub(1.0),
            })
        );
    }

    #[test]
    fn expense_is_one_percent_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(1),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, _rub(1.0));
        let plan = Plan::from_draft(draft).unwrap();
        assert_eq!(
            distribute(&plan, &income),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(0.01))]),
                rest: _rub(0.99),
            })
        );
    }
}
