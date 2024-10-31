use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd)]
pub struct Percentage(Decimal);

impl FromStr for Percentage {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim_end_matches('%');
        trimmed.parse::<Decimal>().map(Percentage::from)
    }
}

impl AddAssign for Percentage {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Percentage {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self.0 - rhs.0)
    }
}

impl Percentage {
    /// A constants representing 100%.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use anna_ivanovna::finance::Percentage;
    /// assert_eq!(Percentage::ONE_HUNDRED, Percentage::from_int(100));
    /// assert_eq!(Percentage::TOTAL, Percentage::from_int(100));
    /// ```
    pub const ONE_HUNDRED: Percentage = Percentage(Decimal::ONE_HUNDRED);
    pub const TOTAL: Percentage = Percentage(Decimal::ONE_HUNDRED);

    /// A constant representing 50%.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use anna_ivanovna::finance::Percentage;
    /// assert_eq!(Percentage::HALF, Percentage::from_int(50));
    /// ```
    pub const HALF: Percentage = Percentage(dec!(50));

    /// A constant representing 25%.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use anna_ivanovna::finance::Percentage;
    /// assert_eq!(Percentage::QUARTER, Percentage::from_int(25));
    /// ```
    pub const QUARTER: Percentage = Percentage(dec!(25));

    /// A constant representing 1%.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use anna_ivanovna::finance::Percentage;
    /// assert_eq!(Percentage::ZERO, Percentage::from_int(0));
    /// ```
    pub const ONE: Percentage = Percentage(Decimal::ONE);

    /// A constant representing 0%.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use anna_ivanovna::finance::Percentage;
    /// assert_eq!(Percentage::ZERO, Percentage::from_int(0));
    /// ```
    pub const ZERO: Percentage = Percentage(Decimal::ZERO);

    #[must_use]
    pub fn from(v: Decimal) -> Self {
        Self(v)
    }

    #[must_use]
    pub fn from_int(d: i64) -> Self {
        Percentage::from(Decimal::new(d, 0))
    }

    /// Вычисляет, сколько процентов одно число составляет от другого.
    /// Результат, округляется до 2 знака после запятой
    ///
    /// # Arguments
    ///
    /// * part: часть (число, от которого нужно найти процент)
    ///  * whole: целое (число, относительно которого ищется процент)
    ///
    /// # Паника
    /// Эта функция вызывает панику, если `total` равно нулю, так как деление на ноль невозможно.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    /// use rust_decimal_macros::dec;
    /// use anna_ivanovna::finance::Percentage;
    ///
    /// assert_eq!(Percentage::of(Decimal::ONE, Decimal::ONE_HUNDRED), Percentage::ONE);
    /// assert_eq!(Percentage::of(Decimal::ONE, Decimal::ONE), Percentage::ONE_HUNDRED);
    /// assert_eq!(Percentage::of(Decimal::ZERO, Decimal::ONE_HUNDRED), Percentage::ZERO);
    /// assert_eq!(Percentage::of(dec!(1.1), dec!(12)), Percentage::from(dec!(9.17)));
    /// assert_eq!(Percentage::of(dec!(50), dec!(100)), Percentage::HALF);
    ///
    /// # #[should_panic]
    /// # fn percentage_of_zero() {
    /// #  Percentage::of(Decimal::ONE_HUNDRED, Decimal::ZERO);
    /// # }
    /// ```
    #[must_use]
    pub fn of(part: Decimal, whole: Decimal) -> Self {
        Percentage::from((part / whole * dec!(100)).round_dp(2))
    }

    /// Применяет процентное значение к числу типа `Decimal`.
    ///
    /// Функция делит значение `self` на 100, чтобы преобразовать его в процентное значение,
    /// и затем умножает на переданный аргумент `d`, получая конечный результат.
    ///
    /// # Параметры
    /// - `d`: Значение типа `Decimal`, к которому будет применен процент.
    ///
    /// # Возвращаемое значение
    /// - Возвращает новое значение типа `Decimal`, представляющее `d`, умноженное на процентное значение `self`.
    ///
    /// # Пример
    /// ```
    /// use rust_decimal::Decimal;
    /// use rust_decimal_macros::dec;
    /// use anna_ivanovna::finance::Percentage;
    ///
    /// assert_eq!(Percentage::HALF.apply_to(Decimal::ONE_HUNDRED), dec!(50));
    /// assert_eq!(Percentage::ONE.apply_to(Decimal::ONE), dec!(0.01));
    /// assert_eq!(Percentage::ONE_HUNDRED.apply_to(Decimal::ONE_HUNDRED), Decimal::ONE_HUNDRED);
    /// assert_eq!(Percentage::from_int(12).apply_to(dec!(100)), dec!(12));
    /// ```
    #[must_use]
    pub fn apply_to(&self, d: Decimal) -> Decimal {
        self.0 / dec!(100) * d
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

impl FromStr for Money {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim_start_matches('₽');
        trimmed.parse::<Decimal>().map(Money::new_rub)
    }
}


impl Sum for Money {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        let mut total_rub = Money::new_rub(Decimal::ZERO);
        for m in iter {
            match m.currency {
                Currency::RUB => total_rub += m,
                Currency::USD => panic!("Нельзя складывать разные валюты"),
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
        self.value = self.value - other.value;
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        // Пока валюта одна - запрещаем складывать разные валюты
        assert_eq!(
            self.currency, other.currency,
            "Нельзя складывать разные валюты"
        );
        self.value = self.value + other.value;
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
        let _ = Self::new(self.value - other.value, self.currency);
    }
}

impl Money {
    #[must_use]
    pub fn new(value: Decimal, currency: Currency) -> Self {
        Self {
            value: value.round_dp(2),
            currency,
        }
    }
    #[must_use]
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
                write!(f, "₽")
            }
            Currency::USD => {
                write!(f, "$")
            }
        }
    }
}
