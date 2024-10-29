use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Percentage(Decimal);

impl Percentage {
    pub fn from(v: Decimal) -> Self {
        // if v > dec!(100) || v < Decimal::ZERO {
        //     panic!("Percentage value must be between 0 and 100, but it {}", v);
        // }
        Self(v)
    }

    pub fn from_int(d: i64) -> Self {
        Percentage::from(Decimal::new(d, 0))
    }

    pub fn how(value: Decimal, on: Decimal) -> Self {
        Percentage::from((value / on * dec!(100)).round_dp(2))
    }

    pub fn apply_to(&self, d: Decimal) -> Decimal {
        &self.0 / dec!(100) * d
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub struct Money {
    pub value: Decimal,
    pub currency: Currency,
}

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", &self.currency, &self.value)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        let mut total_rub = Money::new_rub(Decimal::ZERO);
        for m in iter {
            match m.currency {
                Currency::RUB => total_rub += m,
                _ => panic!("Нельзя складывать разные валюты")
            }
        }
        total_rub
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        // Пока валюта одна - запрещаем складывать разные валюты
        assert_eq!(
            self.currency, other.currency,
            "Нельзя складывать разные валюты"
        );
        self.value = self.value - other.value
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        // Пока валюта одна - запрещаем складывать разные валюты
        assert_eq!(
            self.currency, other.currency,
            "Нельзя складывать разные валюты"
        );
        self.value = self.value + other.value
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        // Пока валюта одна - запрещаем складывать разные валюты
        assert_eq!(
            self.currency, other.currency,
            "Нельзя складывать разные валюты"
        );
        Self::new(self.value + other.value, self.currency)
    }
}

impl Sub for Money {
    type Output = ();

    fn sub(self, other: Self) -> Self::Output {
        // Пока валюта одна - запрещаем складывать разные валюты
        assert_eq!(
            self.currency, other.currency,
            "Нельзя складывать разные валюты"
        );
        Self::new(self.value - other.value, self.currency);
    }
}

impl Money {
    pub fn new(value: Decimal, currency: Currency) -> Self {
        Self {
            value: value.round_dp(2),
            currency,
        }
    }

    pub fn new_rub(value: Decimal) -> Self {
        Self::new(value, Currency::RUB)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Currency {
    RUB,
    USD,
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Currency::RUB => {
                write!(f, "{}", '₽')
            }
            Currency::USD => {
                write!(f, "{}", '$')
            }
        }
    }
}

#[cfg(test)]
mod percentage {
    use rust_decimal_macros::dec;

    use crate::finance::Percentage;

    #[test]
    fn how() {
        assert_eq!(Percentage::how(dec!(5), dec!(100)), Percentage::from_int(5));
        assert_eq!(
            Percentage::how(dec!(1.1), dec!(12)),
            Percentage::from(dec!(9.17))
        );
        assert_eq!(
            Percentage::how(dec!(50), dec!(100)),
            Percentage::from_int(50)
        );
        assert_eq!(Percentage::how(dec!(0), dec!(100)), Percentage::from_int(0));

        assert_eq!(
            Percentage::how(dec!(100), dec!(100)),
            Percentage::from_int(100)
        );
        assert_eq!(
            Percentage::how(dec!(25000), dec!(100000)),
            Percentage::from_int(25)
        );
    }

    #[test]
    #[should_panic]
    fn how_from_zero() {
        assert_eq!(Percentage::how(dec!(10), dec!(0)), Percentage::from_int(0));
    }

    #[test]
    fn apply_to() {
        assert_eq!(Percentage::from_int(50).apply_to(dec!(100)), dec!(50));
        assert_eq!(Percentage::from_int(12).apply_to(dec!(100)), dec!(12));
        assert_eq!(Percentage::from_int(1).apply_to(dec!(1)), dec!(0.01));
    }
}
