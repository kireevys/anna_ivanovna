use std::str::FromStr;

use rust_decimal::Decimal;

use ai_core::finance::{Money, Percentage};

pub struct TaxBreakdown {
    pub gross: Money,
    pub net: Money,
    pub tax: Money,
}

pub fn parse_rate(raw: &str) -> Option<Percentage> {
    Decimal::from_str(raw).ok().map(Percentage::from)
}

pub fn tax_from_gross(gross: &str, rate: &Percentage) -> Option<TaxBreakdown> {
    let gross = Decimal::from_str(gross).ok()?;
    let tax = rate.apply_to(gross);
    Some(TaxBreakdown {
        gross: Money::new_rub(gross),
        net: Money::new_rub(gross - tax),
        tax: Money::new_rub(tax),
    })
}

pub fn tax_from_net(net: &str, rate: &Percentage) -> Option<TaxBreakdown> {
    let net = Decimal::from_str(net).ok()?;
    let gross = rate.gross_from_net(net)?;
    Some(TaxBreakdown {
        gross: Money::new_rub(gross),
        net: Money::new_rub(net),
        tax: Money::new_rub(gross - net),
    })
}
