use crate::finance::{Money, Percentage};
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize)]
pub struct IncomeSource {
    pub name: String,
    #[serde(skip)]
    pub expected: Money,
}
impl PartialEq for IncomeSource {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl IncomeSource {
    #[must_use]
    pub fn new(name: String, expected: Money) -> Self {
        Self { name, expected }
    }
}

impl Display for IncomeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.expected)
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    TooBigExpenses,
    InvalidPlan,
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize)]
pub enum ExpenseValue {
    RATE { value: Percentage },
    MONEY { value: Money },
}

impl FromStr for ExpenseValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('%') {
            let percentage =
                Percentage::from_str(s).map_err(|e| format!("Failed to parse percentage: {e}"))?;
            return Ok(ExpenseValue::RATE { value: percentage });
        }

        if s.contains("₽") {
            let money = Money::from_str(s).map_err(|e| format!("Failed to parse money: {e}"))?;
            return Ok(ExpenseValue::MONEY { value: money });
        }

        Err("Invalid format".to_string())
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize)]
pub struct Expense {
    pub name: String,
    #[serde(skip)]
    pub value: ExpenseValue,
}

impl Expense {
    #[must_use]
    pub fn new(name: String, value: ExpenseValue) -> Self {
        Self { name, value }
    }
}

#[derive(PartialEq, Debug)]
pub struct Plan {
    pub sources: Vec<IncomeSource>,
    plan: HashMap<Expense, Percentage>,
    pub rest: Percentage,
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
}

impl TryFrom<Draft> for Plan {
    type Error = Error;

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
    fn try_from(draft: Draft) -> Result<Self, Self::Error> {
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

    fn rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn test_empty_draft() {
        let draft = Draft::new();
        assert_eq!(Plan::try_from(draft), Err(Error::EmptyPlan));
    }

    #[test]
    fn no_expenses() {
        let draft = Draft::build(
            &[IncomeSource::new("Gold goose".to_string(), rub(1.0))],
            &[],
        );
        assert_eq!(Plan::try_from(draft), Err(Error::EmptyPlan));
    }

    #[test]
    fn build_rate_plan_from_rate_expense() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let res = Plan::try_from(draft).unwrap();
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
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(101),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        assert_eq!(Plan::try_from(draft), Err(Error::TooBigExpenses));
    }

    #[test]
    fn build_rate_with_overhead_total_percent() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
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
        let draft = Draft::build(&[source.clone()], &[expense_1.clone(), expense_2.clone()]);
        assert_eq!(Plan::try_from(draft), Err(Error::TooBigExpenses));
    }

    #[test]
    fn build_rate_when_has_rest() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(50),
            },
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);

        let expected = HashMap::from([(expense.clone(), Percentage::from_int(50))]);
        assert_eq!(
            Plan::try_from(draft).unwrap(),
            Plan {
                sources: vec![source.clone()],
                plan: expected,
                rest: Percentage::HALF,
            }
        );
    }

    #[test]
    fn build_full_plan() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense_1 = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.25) },
        );
        let expense_2 = Expense::new(
            "Yet Another Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.5) },
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
        let res = Plan::try_from(draft).unwrap();
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
