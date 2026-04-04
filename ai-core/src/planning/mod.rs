mod expense;
mod income;

use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display, Formatter},
    ops::Deref,
};

use crate::finance::Percentage;

pub use expense::{
    CreditExpense,
    CreditValidationError,
    Expense,
    ExpenseKind,
    ExpenseValue,
};
pub use income::{IncomeKind, IncomeSource};

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    TooBigExpenses,
    InvalidCredit(CreditValidationError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyPlan => write!(f, "empty plan"),
            Error::TooBigExpenses => write!(f, "expenses exceed income"),
            Error::InvalidCredit(e) => write!(f, "invalid credit: {e}"),
        }
    }
}

#[derive(PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DistributionWeights {
    pub sources: Vec<IncomeSource>,
    pub budget: HashMap<Expense, Percentage>,
    pub rest: Percentage,
}

impl Debug for DistributionWeights {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let expenses_count = self.budget.len();
        f.debug_struct("Plan")
            .field("sources_count", &self.sources.len())
            .field("expenses_count", &expenses_count)
            .field("rest", &self.rest)
            .finish()
    }
}

#[cfg(test)]
impl Clone for DistributionWeights {
    fn clone(&self) -> Self {
        Self {
            sources: self.sources.clone(),
            budget: self.budget.clone(),
            rest: self.rest.clone(),
        }
    }
}

impl<'a> IntoIterator for &'a DistributionWeights {
    type Item = (&'a Expense, &'a Percentage);
    type IntoIter = std::collections::hash_map::Iter<'a, Expense, Percentage>;

    fn into_iter(self) -> Self::IntoIter {
        self.budget.iter()
    }
}

impl Deref for DistributionWeights {
    type Target = HashMap<Expense, Percentage>;

    fn deref(&self) -> &Self::Target {
        &self.budget
    }
}

impl DistributionWeights {
    pub fn has_source(&self, source: &IncomeSource) -> bool {
        self.sources.contains(source)
    }

    /// Группирует расходы по категориям
    pub fn categories(&self) -> impl Iterator<Item = (String, Vec<&Expense>)> {
        let mut sorted_expenses: Vec<_> = self.budget.keys().collect();
        sorted_expenses.sort_by_key(|e| &e.name);

        sorted_expenses
            .iter()
            .fold(
                BTreeMap::new(),
                |mut map: std::collections::BTreeMap<_, Vec<&Expense>>, e| {
                    let entry =
                        e.category.as_deref().unwrap_or("Без категории").to_string();
                    map.entry(entry).or_default().push(*e);
                    map
                },
            )
            .into_iter()
    }
}

#[cfg(test)] mod tests;
