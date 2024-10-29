use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use chrono::{NaiveDate, Utc};

use crate::finance::{Money, Percentage};
use crate::planning::planning::{Expense, ExpenseValue, IncomeSource, Plan};

#[derive(PartialEq, Debug)]
pub enum DistributeError {
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
    pub fn new(source: IncomeSource, money: Money, date: NaiveDate) -> Self {
        Self {
            source,
            money,
            date,
        }
    }

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
        write!(f, "{}", result)
    }
}

fn make_rate_plan(plan: Plan) -> HashMap<Expense, Percentage> {
    let mut rate_plan = HashMap::with_capacity(plan.expenses.len());
    let plan_total = plan.total_incomes();
    for e in plan.expenses.iter() {
        match &e.value {
            ExpenseValue::MONEY { value } => {
                rate_plan.insert(e.clone(), Percentage::how(value.value, plan_total.value))
            }
            ExpenseValue::RATE { value } => rate_plan.insert(e.clone(), value.clone()),
        };
    }
    println!("{:?}", rate_plan);
    rate_plan
}

pub fn distribute(plan: Plan, income: Income) -> Result<Distribute, DistributeError> {
    plan.sources
        .iter()
        .find(|p| **p == income.source)
        .ok_or_else(|| DistributeError::UnknownSource)?;

    let expenses = HashMap::with_capacity(plan.expenses.len());
    let rate_plan = make_rate_plan(plan);

    let mut d = Distribute::new(income.clone(), expenses.clone());

    for (e, r) in rate_plan.iter() {
        let m = Money::new_rub(r.apply_to(income.money.value));
        d.expenditures.insert(e.clone().clone(), m);
        d.rest -= m;
    }
    Ok(d)
}

#[cfg(test)]
mod test_distribute {
    use std::collections::HashMap;

    use chrono::Utc;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use uuid::Uuid;

    use crate::distribute::{distribute, Distribute, DistributeError, Income, make_rate_plan};
    use crate::finance::{Money, Percentage};
    use crate::planning::planning::{Expense, ExpenseValue, IncomeSource, Plan};

    fn _rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn income_for_empty_plan() {
        let plan = Plan::new();
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let income = Income::new(source, _rub(1.0), Utc::now().date_naive());
        assert_eq!(
            distribute(plan, income.clone()),
            Err(DistributeError::UnknownSource)
        );
    }

    #[test]
    fn income_from_unknown_source() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let source_1 = IncomeSource::new("Unknown".to_string(), _rub(1.0));
        let plan = Plan::try_build(Uuid::new_v4(), vec![source], vec![]).unwrap();
        let income = Income::new(source_1, _rub(1.0), Utc::now().date_naive());
        assert_eq!(
            distribute(plan, income.clone()),
            Err(DistributeError::UnknownSource)
        );
    }

    #[test]
    fn no_expenses() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let plan = Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![]).unwrap();
        let income = Income::new(source, _rub(1.0), Utc::now().date_naive());
        assert_eq!(
            distribute(plan, income.clone()),
            Ok(Distribute {
                income: income.clone(),
                rest: _rub(1.0),
                expenditures: HashMap::new(),
            })
        );
    }

    #[test]
    fn expense_is_money_and_less_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: _rub(0.5) },
        );
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let income = Income::new_today(source, _rub(1.0));
        assert_eq!(
            distribute(plan, income.clone()),
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
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let income = Income::new_today(source, _rub(0.5));
        assert_eq!(
            distribute(plan, income.clone()),
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
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let income = Income::new_today(source, _rub(1.0));
        assert_eq!(
            distribute(plan, income.clone()),
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
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let income = Income::new_today(source, _rub(1.0));
        assert_eq!(
            distribute(plan, income.clone()),
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
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let income = Income::new_today(source, _rub(1.0));
        assert_eq!(
            distribute(plan, income.clone()),
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
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let income = Income::new_today(source, _rub(1.0));
        assert_eq!(
            distribute(plan, income.clone()),
            Ok(Distribute {
                income: income.clone(),
                expenditures: HashMap::from([(expense.clone(), _rub(0.01))]),
                rest: _rub(0.99),
            })
        );
    }

    #[test]
    fn test_empty_rate_plan() {
        let plan = Plan::try_build(Uuid::new_v4(), vec![], vec![]).unwrap();
        let res = make_rate_plan(plan);
        let expected = HashMap::with_capacity(0);
        assert_eq!(res, expected)
    }

    #[test]
    fn build_rate_plan_from_rate_expense() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
        );
        let plan =
            Plan::try_build(Uuid::new_v4(), vec![source.clone()], vec![expense.clone()]).unwrap();
        let res = make_rate_plan(plan);
        let expected = HashMap::from([(expense.clone(), Percentage::from_int(100))]);

        assert_eq!(res, expected)
    }

    #[test]
    fn build_rate_plan_from_money_expenses() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense_1 = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: _rub(0.25) },
        );
        let expense_2 = Expense::new(
            "Yet Another Black Hole".to_string(),
            ExpenseValue::MONEY { value: _rub(0.5) },
        );
        let expense_3 = Expense::new(
            "Rate Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(25),
            },
        );
        let plan = Plan::try_build(
            Uuid::new_v4(),
            vec![source.clone()],
            vec![expense_1.clone(), expense_2.clone(), expense_3.clone()],
        )
            .unwrap();
        let res = make_rate_plan(plan);
        let expected = HashMap::from([
            (expense_1.clone(), Percentage::from_int(25)),
            (expense_2.clone(), Percentage::from_int(50)),
            (expense_3.clone(), Percentage::from_int(25)),
        ]);

        assert_eq!(res, expected)
    }
}
