use crate::finance::{Money, Percentage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeSource {
    pub name: String,
    pub expected: Money,
}

impl IncomeSource {
    pub fn expected(&self) -> &Money {
        &self.expected
    }
}
impl PartialEq for IncomeSource {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl IncomeSource {
    pub fn new(name: String, expected: Money) -> Self {
        Self { name, expected }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    TooBigExpenses,
    InvalidPlan,
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum ExpenseValue {
    RATE { value: Percentage },
    MONEY { value: Money },
}

impl Default for ExpenseValue {
    fn default() -> Self {
        ExpenseValue::MONEY {
            value: Money::default(),
        }
    }
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

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Expense {
    pub name: String,
    pub value: ExpenseValue,
    pub category: Option<String>,
}

impl Expense {
    pub fn new(name: String, value: ExpenseValue, category: Option<String>) -> Self {
        Self {
            name,
            value,
            category,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
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
                    let entry = e.category.as_deref().unwrap_or("Без категории").to_string();
                    map.entry(entry).or_default().push(*e);
                    map
                },
            )
            .into_iter()
    }
}
