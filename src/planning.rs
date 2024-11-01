use rust_decimal::Decimal;
use std::collections::HashMap;
use std::ops::Deref;
use uuid::Uuid;

use crate::finance::{Money, Percentage};

#[derive(Debug, Clone)]
pub struct IncomeSource {
    id: Uuid,
    pub name: String,
    pub expected: Money,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    TooBigExpenses,
}

impl PartialEq for IncomeSource {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum ExpenseValue {
    RATE { value: Percentage },
    MONEY { value: Money },
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Expense {
    uuid: Uuid,
    pub name: String,
    pub value: ExpenseValue,
}

impl Expense {
    #[must_use]
    pub fn new(name: String, value: ExpenseValue) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
            value,
        }
    }
}

impl IncomeSource {
    #[must_use]
    pub fn new(name: String, expected: Money) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            expected,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Plan {
    sources: Vec<IncomeSource>,
    plan: HashMap<Expense, Percentage>,
    rest: Percentage,
}

impl<'a> IntoIterator for &'a Plan {
    type Item = (&'a Expense, &'a Percentage);
    type IntoIter = std::collections::hash_map::Iter<'a, Expense, Percentage>;

    fn into_iter(self) -> Self::IntoIter {
        self.plan.iter()
    }
}

impl Deref for Plan {
    type Target = HashMap<Expense, Percentage>;

    fn deref(&self) -> &Self::Target {
        &self.plan
    }
}

impl Plan {
    #[must_use]
    pub fn has_source(&self, source: &IncomeSource) -> bool {
        self.sources.iter().any(|p| *p == *source)
    }


    /// Создает План из Черновика
    ///
    /// # Arguments
    ///
    /// * `draft`: Черновик
    ///
    /// # Errors
    /// `EmptyPlan`: Нельзя создать План из пустого Черновика
    /// `TooBigExpenses`: Нужно запланировать свои Расходы так, чтобы они не превышали Доходы
    ///
    /// returns: Result<Plan, Error>
    ///
    pub fn from_draft(draft: Draft) -> Result<Self, Error> {
        if draft.expenses.is_empty() || draft.sources.is_empty() {
            return Err(Error::EmptyPlan);
        }

        let plan_total = draft.total_incomes();
        let mut rate_plan = HashMap::with_capacity(draft.expenses.len());
        let mut total = Percentage::ZERO;
        for e in draft.expenses {
            let current = match &e.value {
                ExpenseValue::MONEY { value } => Percentage::of(value.value, plan_total.value),
                ExpenseValue::RATE { value } => value.clone(),
            };
            total += current.clone();
            if total > Percentage::ONE_HUNDRED {
                return Err(Error::TooBigExpenses);
            }
            rate_plan.insert(e.clone(), current);
        }
        Ok(Self {
            sources: draft.sources.clone(),
            plan: rate_plan,
            rest: Percentage::ONE_HUNDRED - total,
        })
    }
}
#[derive(Clone)]
pub struct Draft {
    pub sources: Vec<IncomeSource>,
    pub expenses: Vec<Expense>,
}

impl Default for Draft {
    fn default() -> Self {
        Self::new()
    }
}

impl Draft {
    pub fn add_expense(&mut self, expense: Expense) {
        let () = &self.expenses.push(expense);
    }
    pub fn add_source(&mut self, income_source: IncomeSource) {
        let () = &self.sources.push(income_source);
    }
    #[must_use]
    pub fn new() -> Self {
        Self {
            sources: vec![],
            expenses: vec![],
        }
    }
    #[must_use]
    pub fn build(sources: &[IncomeSource], expenses: &[Expense]) -> Self {
        let mut draft = Self::new();
        sources.iter().for_each(|s| draft.add_source(s.clone()));
        expenses.iter().for_each(|e| draft.add_expense(e.clone()));
        draft
    }

    #[must_use]
    pub fn total_incomes(&self) -> Money {
        self.sources
            .iter()
            .map(|s| s.expected)
            .fold(Money::new_rub(Decimal::ZERO), |acc, income| acc + income)

        // let mut total_rub = Money::new_rub(Decimal::ZERO);
        // for s in self.sources.iter() {
        //     total_rub += s.expected
        // }
        // total_rub
    }
}

#[cfg(test)]
mod test_planning {
    use std::collections::HashMap;

    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::finance::{Currency, Money, Percentage};

    use super::*;

    #[test]
    fn new_plan() {
        let draft = Draft::new();
        assert_eq!(draft.expenses, vec![]);
        assert_eq!(draft.sources, vec![]);
    }

    #[test]
    fn add_source() {
        let mut draft = Draft::new();
        let source =
            IncomeSource::new("Gold goose".to_string(), Money::new(dec!(1), Currency::RUB));
        draft.add_source(source.clone());
        assert_eq!(draft.sources, vec![source]);
        assert_eq!(draft.expenses, vec![]);
    }

    #[test]
    fn add_value_expense() {
        let mut draft = Draft::new();
        let expense = Expense::new(
            "Black hole".to_string(),
            ExpenseValue::MONEY {
                value: Money::new(dec!(1), Currency::RUB),
            },
        );
        draft.add_expense(expense.clone());
        assert_eq!(draft.expenses, vec![expense]);
        assert_eq!(draft.sources, vec![]);
    }

    #[test]
    fn add_rate_expense() {
        let mut draft = Draft::new();
        let expense = Expense::new(
            "Black hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(10),
            },
        );
        draft.add_expense(expense.clone());
        assert_eq!(draft.expenses, vec![expense]);
        assert_eq!(draft.sources, vec![]);
    }

    fn _rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn test_empty_draft() {
        let draft = Draft::new();
        assert_eq!(Plan::from_draft(draft), Err(Error::EmptyPlan));
    }

    #[test]
    fn no_expenses() {
        let draft = Draft::build(
            &[IncomeSource::new("Gold goose".to_string(), _rub(1.0))],
            &[],
        );
        assert_eq!(Plan::from_draft(draft), Err(Error::EmptyPlan));
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
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let res = Plan::from_draft(draft).unwrap();
        let expected = HashMap::from([(expense.clone(), Percentage::from_int(100))]);

        assert_eq!(
            res,
            Plan {
                sources: vec![source.clone()],
                plan: expected,
                rest: Percentage::from_int(0),
            }
        );
    }

    #[test]
    fn build_plan_with_overhead_percent() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(101),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        assert_eq!(Plan::from_draft(draft), Err(Error::TooBigExpenses));
    }

    #[test]
    fn build_rate_with_overhead_total_percent() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense_1 = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
        );
        let expense_2 = Expense::new(
            "Little bit".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(dec!(0.01)),
            },
        );
        let draft = Draft::build(
            &[source.clone()],
            &[expense_1.clone(), expense_2.clone()],
        );
        assert_eq!(Plan::from_draft(draft), Err(Error::TooBigExpenses));
    }

    #[test]
    fn build_rate_when_has_rest() {
        let source = IncomeSource::new("Gold goose".to_string(), _rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(50),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);

        let expected = HashMap::from([(expense.clone(), Percentage::from_int(50))]);
        assert_eq!(
            Plan::from_draft(draft).unwrap(),
            Plan {
                sources: vec![source.clone()],
                plan: expected,
                rest: Percentage::HALF,
            }
        );
    }

    #[test]
    fn build_full_plan() {
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
                value: Percentage::QUARTER,
            },
        );
        let draft = Draft::build(
            &[source.clone()],
            &[expense_1.clone(), expense_2.clone(), expense_3.clone()],
        );
        let res = Plan::from_draft(draft).unwrap();
        let expected = HashMap::from([
            (expense_1.clone(), Percentage::QUARTER),
            (expense_2.clone(), Percentage::HALF),
            (expense_3.clone(), Percentage::QUARTER),
        ]);

        assert_eq!(
            res,
            Plan {
                sources: vec![source.clone()],
                plan: expected,
                rest: Percentage::ZERO,
            }
        );
    }
}
