use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    finance::{Money, Percentage},
    planning::Error,
};

#[derive(Debug, PartialEq)]
pub enum CreditValidationError {
    ZeroTermMonths,
    NonPositivePayment,
    NonPositiveAmount,
}

impl Display for CreditValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CreditValidationError::ZeroTermMonths => {
                write!(f, "term months must be > 0")
            }
            CreditValidationError::NonPositivePayment => {
                write!(f, "monthly payment must be > 0")
            }
            CreditValidationError::NonPositiveAmount => {
                write!(f, "total amount must be > 0")
            }
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

#[non_exhaustive]
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize)]
pub struct CreditExpense {
    pub monthly_payment: Money,
    pub total_amount: Money,
    pub interest_rate: Percentage,
    pub term_months: u32,
    pub start_date: NaiveDate,
}

impl<'de> Deserialize<'de> for CreditExpense {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            monthly_payment: Money,
            total_amount: Money,
            interest_rate: Percentage,
            term_months: u32,
            start_date: NaiveDate,
        }

        let raw = Raw::deserialize(deserializer)?;
        CreditExpense::new(
            raw.monthly_payment,
            raw.total_amount,
            raw.interest_rate,
            raw.term_months,
            raw.start_date,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl CreditExpense {
    pub fn new(
        monthly_payment: Money,
        total_amount: Money,
        interest_rate: Percentage,
        term_months: u32,
        start_date: NaiveDate,
    ) -> Result<Self, Error> {
        if term_months == 0 {
            return Err(Error::InvalidCredit(CreditValidationError::ZeroTermMonths));
        }
        if monthly_payment.value <= Decimal::ZERO {
            return Err(Error::InvalidCredit(
                CreditValidationError::NonPositivePayment,
            ));
        }
        if total_amount.value <= Decimal::ZERO {
            return Err(Error::InvalidCredit(
                CreditValidationError::NonPositiveAmount,
            ));
        }
        Ok(Self {
            monthly_payment,
            total_amount,
            interest_rate,
            term_months,
            start_date,
        })
    }

    pub fn value(&self) -> ExpenseValue {
        ExpenseValue::MONEY {
            value: self.monthly_payment,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpenseKind {
    Envelope { value: ExpenseValue },
    Credit(CreditExpense),
}

impl ExpenseKind {
    pub(crate) fn value(&self) -> ExpenseValue {
        match self {
            ExpenseKind::Envelope { value } => value.clone(),
            ExpenseKind::Credit(credit) => credit.value(),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize)]
pub struct Expense {
    pub name: String,
    pub kind: ExpenseKind,
    pub category: Option<String>,
}

impl<'de> Deserialize<'de> for Expense {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            kind: Option<ExpenseKind>,
            value: Option<ExpenseValue>,
            category: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let kind = match (raw.kind, raw.value) {
            (Some(kind), _) => kind,
            (None, Some(value)) => ExpenseKind::Envelope { value },
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "Expense must have either 'kind' or 'value' field",
                ));
            }
        };

        Ok(Self {
            name: raw.name,
            kind,
            category: raw.category,
        })
    }
}

impl Expense {
    pub fn envelope(
        name: String,
        value: ExpenseValue,
        category: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: ExpenseKind::Envelope { value },
            category,
        }
    }

    pub fn credit(
        name: String,
        credit: CreditExpense,
        category: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: ExpenseKind::Credit(credit),
            category,
        }
    }

    pub fn value(&self) -> ExpenseValue {
        self.kind.value()
    }
}
