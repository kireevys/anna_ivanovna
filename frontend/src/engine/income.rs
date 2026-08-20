use std::str::FromStr;

use rust_decimal::Decimal;

use ai_core::finance::{Currency, Money, Percentage};

pub struct TaxBreakdown {
    pub gross: Money,
    pub net: Money,
    pub tax: Money,
}

pub fn parse_rate(raw: &str) -> Option<Percentage> {
    Percentage::from_str(raw).ok()
}

pub fn withhold(gross: Money, rate: &Percentage) -> Option<TaxBreakdown> {
    if !rate.is_valid_withholding_rate() {
        return None;
    }
    let tax = rate.apply_to(gross.value);
    Some(TaxBreakdown {
        gross,
        net: Money::new(gross.value - tax, gross.currency),
        tax: Money::new(tax, gross.currency),
    })
}

pub fn tax_from_gross(
    gross: &str,
    rate: &Percentage,
    currency: Currency,
) -> Option<TaxBreakdown> {
    let gross = Decimal::from_str(gross).ok()?;
    withhold(Money::new(gross, currency), rate)
}

pub fn restore_gross(net: Money, rate: &Percentage) -> Option<TaxBreakdown> {
    let gross = rate.gross_from_net(net.value)?;
    Some(TaxBreakdown {
        gross: Money::new(gross, net.currency),
        net,
        tax: Money::new(gross - net.value, net.currency),
    })
}

pub fn tax_from_net(
    net: &str,
    rate: &Percentage,
    currency: Currency,
) -> Option<TaxBreakdown> {
    let net = Decimal::from_str(net).ok()?;
    restore_gross(Money::new(net, currency), rate)
}
