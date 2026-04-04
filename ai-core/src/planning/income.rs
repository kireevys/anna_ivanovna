use serde::{Deserialize, Deserializer, Serialize};

use crate::finance::{Money, Percentage};

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
