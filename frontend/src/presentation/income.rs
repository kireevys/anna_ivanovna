use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use ai_core::{
    finance::{Money, Percentage},
    planning::IncomeKind as CoreIncomeKind,
};

use crate::presentation::formatting::{FormattedMoney, FormattedPercentage};

pub const SALARY_LABEL: &str = "Зарплата";
pub const OTHER_LABEL: &str = "Другое";

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum SourceKind {
    Salary {
        gross: FormattedMoney,
        tax_rate: String,
        tax_amount: FormattedMoney,
    },
    Other,
}

impl SourceKind {
    pub fn kind_label(&self) -> &str {
        match self {
            SourceKind::Salary { .. } => SALARY_LABEL,
            SourceKind::Other => OTHER_LABEL,
        }
    }
}

impl From<&CoreIncomeKind> for SourceKind {
    fn from(kind: &CoreIncomeKind) -> Self {
        match kind {
            CoreIncomeKind::Salary { gross, tax_rate } => {
                let tax = tax_rate.apply_to(gross.value);
                SourceKind::Salary {
                    gross: FormattedMoney::from_money(*gross),
                    tax_rate: FormattedPercentage::from(tax_rate.clone()).raw_value(),
                    tax_amount: FormattedMoney::from_money(Money::new(
                        tax,
                        gross.currency,
                    )),
                }
            }
            CoreIncomeKind::Other { .. } => SourceKind::Other,
        }
    }
}

pub struct TaxFromGross {
    pub net: FormattedMoney,
    pub tax: FormattedMoney,
}

pub fn tax_from_gross(gross_str: &str, rate_str: &str) -> Option<TaxFromGross> {
    let gross = Decimal::from_str(gross_str).ok()?;
    let rate = Decimal::from_str(rate_str).ok()?;
    let tax = Percentage::from(rate).apply_to(gross);
    Some(TaxFromGross {
        net: FormattedMoney::from_money(Money::new_rub(gross - tax)),
        tax: FormattedMoney::from_money(Money::new_rub(tax)),
    })
}

pub struct TaxFromNet {
    pub gross: FormattedMoney,
    pub tax: FormattedMoney,
}

pub fn tax_from_net(net_str: &str, rate_str: &str) -> Option<TaxFromNet> {
    let net = Decimal::from_str(net_str).ok()?;
    let rate = Decimal::from_str(rate_str).ok()?;
    let hundred = Decimal::from(100);
    let gross = net * hundred / (hundred - rate);
    let tax = gross - net;
    Some(TaxFromNet {
        gross: FormattedMoney::from_money(Money::new_rub(gross)),
        tax: FormattedMoney::from_money(Money::new_rub(tax)),
    })
}
