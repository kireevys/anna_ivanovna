use crate::finance::{Money, Percentage};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomeKind {
    Salary { gross: Money, tax_rate: Percentage },
    Other { expected: Money },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IncomeSource {
    pub name: String,
    pub kind: IncomeKind,
}

impl IncomeSource {
    pub fn new(name: String, kind: IncomeKind) -> Self {
        Self { name, kind }
    }

    pub fn net(&self) -> Money {
        match &self.kind {
            IncomeKind::Salary { gross, tax_rate } => {
                Money::new(gross.value - tax_rate.apply_to(gross.value), gross.currency)
            }
            IncomeKind::Other { expected } => *expected,
        }
    }
}

impl<'de> Deserialize<'de> for IncomeSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            kind: Option<IncomeKind>,
            expected: Option<Money>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let kind = match (raw.kind, raw.expected) {
            (Some(kind), _) => kind,
            (None, Some(expected)) => IncomeKind::Other { expected },
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "IncomeSource must have either 'kind' or 'expected' field",
                ));
            }
        };

        Ok(Self {
            name: raw.name,
            kind,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    TooBigExpenses,
    InvalidPlan,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyPlan => write!(f, "empty plan"),
            Error::TooBigExpenses => write!(f, "expenses exceed income"),
            Error::InvalidPlan => write!(f, "invalid plan"),
        }
    }
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
            let percentage = Percentage::from_str(s)
                .map_err(|e| format!("Failed to parse percentage: {e}"))?;
            return Ok(ExpenseValue::RATE { value: percentage });
        }

        if s.contains("₽") {
            let money = Money::from_str(s)
                .map_err(|e| format!("Failed to parse money: {e}"))?;
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
                    let entry =
                        e.category.as_deref().unwrap_or("Без категории").to_string();
                    map.entry(entry).or_default().push(*e);
                    map
                },
            )
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::finance::{Currency, Money, Percentage};

    use super::*;

    fn make_source(name: &str, kind: IncomeKind) -> IncomeSource {
        IncomeSource::new(name.to_string(), kind)
    }

    #[test]
    fn salary_net_applies_tax() {
        let source = make_source(
            "Зарплата",
            IncomeKind::Salary {
                gross: Money::new_rub(dec!(100000)),
                tax_rate: Percentage::from_int(13),
            },
        );
        assert_eq!(source.net(), Money::new_rub(dec!(87000)));
    }

    #[test]
    fn other_net_returns_expected() {
        let source = make_source(
            "Фриланс",
            IncomeKind::Other {
                expected: Money::new_rub(dec!(50000)),
            },
        );
        assert_eq!(source.net(), Money::new_rub(dec!(50000)));
    }

    #[test]
    fn serde_salary_roundtrip() {
        let source = make_source(
            "Зарплата",
            IncomeKind::Salary {
                gross: Money::new_rub(dec!(200000)),
                tax_rate: Percentage::from_int(15),
            },
        );
        let json = serde_json::to_string(&source).unwrap();
        let deserialized: IncomeSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, source.name);
        assert_eq!(deserialized.kind, source.kind);
    }

    #[test]
    fn serde_other_roundtrip() {
        let source = make_source(
            "Фриланс",
            IncomeKind::Other {
                expected: Money::new_rub(dec!(50000)),
            },
        );
        let json = serde_json::to_string(&source).unwrap();
        let deserialized: IncomeSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, source.name);
        assert_eq!(deserialized.kind, source.kind);
    }

    #[test]
    fn serde_backward_compat_old_format() {
        let old_json =
            r#"{"name":"Зарплата","expected":{"value":"100000","currency":"RUB"}}"#;
        let source: IncomeSource = serde_json::from_str(old_json).unwrap();
        assert_eq!(source.name, "Зарплата");
        assert_eq!(
            source.kind,
            IncomeKind::Other {
                expected: Money::new_rub(dec!(100000)),
            }
        );
    }

    #[test]
    fn serde_salary_uses_tag() {
        let source = make_source(
            "Зарплата",
            IncomeKind::Salary {
                gross: Money::new(dec!(100000), Currency::RUB),
                tax_rate: Percentage::from_int(13),
            },
        );
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains(r#""type":"salary""#));
    }
}
