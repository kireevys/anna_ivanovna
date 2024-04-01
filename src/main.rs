use chrono::Utc;
use rust_decimal_macros::dec;

use crate::distributor::{Currency, Expenditure, Income, IncomeSource, Money, Plan};

mod distributor {
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::ops::{Add, AddAssign, Sub, SubAssign};

    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[derive(Debug, PartialEq)]
    pub struct Plan<'a> {
        incomes: Vec<&'a IncomeSource>,
        expenses: Vec<&'a Expenditure>,
    }

    impl<'a> Plan<'a> {
        pub fn try_build(
            incomes: Vec<&'a IncomeSource>,
            expenses: Vec<&'a Expenditure>,
        ) -> Result<Self, DistributeError> {
            if incomes.len() == 0 {
                return Err(DistributeError::NoIncomes);
            }
            Ok(Self { incomes, expenses })
        }

        fn contains_income(&'a self, income: &'a Income) -> Option<&Income> {
            match &self.incomes.iter().find(|p| **p == income.source) {
                Some(_) => Some(income),
                None => None,
            }
        }

        fn contains_expenditure(&'a self, expenditure: &'a Expenditure) -> Option<&Expenditure> {
            match &self.expenses.iter().find(|p| **p == expenditure) {
                Some(_) => Some(expenditure),
                None => None,
            }
        }

        fn expected_sum(&self, currency: Currency) -> Money {
            let mut expected = Money::new(Decimal::ZERO, currency);
            for i in &self.incomes {
                if i.expected.currency == currency {
                    expected += i.expected
                }
            }
            expected
        }

        fn percent(&self, expenditure: &Expenditure) -> Decimal {
            match &self.contains_expenditure(expenditure) {
                Some(e) => e.money.value / &self.expected_sum(expenditure.money.currency).value,
                None => panic!("{:?}", DistributeError::UnknownExpenditure),
            }
        }

        fn distribute(&'a self, expenditure: &'a Expenditure, income: &Income) -> Money {
            Money::new(
                income.money.value * self.percent(expenditure),
                income.currency(),
            )
        }

        pub fn new_income(&self, income: &'a Income) -> Result<Distribution, DistributeError> {
            match &self.contains_income(income) {
                Some(_) => {
                    let mut d = Distribution::from_income(&self, income);
                    for expenditure in self.expenses.iter() {
                        d.add(expenditure, self.distribute(expenditure, income))
                            .expect("Cannot add Expenditure");
                    }
                    Ok(d)
                }
                None => Err(DistributeError::UnknownIncomeSource),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
    pub enum Currency {
        RUB,
    }

    impl Display for Currency {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match &self {
                Currency::RUB => {
                    write!(f, "{}", '₽')
                }
            }
        }
    }

    #[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
    pub struct Money {
        value: Decimal,
        currency: Currency,
    }

    impl Display for Money {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} {}", &self.currency, &self.value)
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
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum DistributeError {
        NoIncomes,
        UnknownIncomeSource,
        UnknownExpenditure,
    }

    #[derive(Debug)]
    pub struct IncomeSource {
        id: Uuid,
        name: String,
        expected: Money,
    }

    impl PartialEq for IncomeSource {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl IncomeSource {
        pub fn new(name: String, expected: Money) -> Self {
            Self {
                id: Uuid::new_v4(),
                name,
                expected,
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Income<'a> {
        source: &'a IncomeSource,
        pub date: DateTime<Utc>,
        money: Money,
    }

    impl<'a> Income<'a> {
        pub fn new(source: &'a IncomeSource, date: DateTime<Utc>, money: Money) -> Self {
            Self {
                source,
                date,
                money,
            }
        }

        pub fn currency(&self) -> Currency {
            self.money.currency
        }
    }

    #[derive(PartialEq, Eq, Hash, Debug)]
    pub struct Expenditure {
        name: String,
        money: Money,
    }

    impl Expenditure {
        pub fn new(name: String, money: Money) -> Self {
            Self { name, money }
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct Distribution<'a> {
        plan: &'a Plan<'a>,
        income: &'a Income<'a>,
        rest: Money,
        hm_items: HashMap<&'a Expenditure, Money>,
    }

    impl<'a> Distribution<'a> {
        pub fn rest(&self) -> &Money {
            &self.rest
        }
        pub fn new(
            plan: &'a Plan,
            income: &'a Income,
            rest: Money,
            items: HashMap<&'a Expenditure, Money>,
        ) -> Self {
            Self {
                plan,
                income,
                rest,
                hm_items: items,
            }
        }

        pub fn from_income(plan: &'a Plan, income: &'a Income) -> Self {
            Self {
                plan,
                income,
                rest: income.money,
                hm_items: HashMap::with_capacity(plan.expenses.len()),
            }
        }

        pub fn add(
            &mut self,
            expenditure: &'a Expenditure,
            money: Money,
        ) -> Result<(), DistributeError> {
            match self.plan.contains_expenditure(expenditure) {
                Some(_) => {
                    self.hm_items
                        .entry(expenditure)
                        .and_modify(|e| *e += money)
                        .or_insert(money);
                    self.rest -= money;
                    Ok(())
                }
                None => Err(DistributeError::UnknownExpenditure),
            }
        }

        pub fn get(&self, expenditure: &Expenditure) -> Option<&Money> {
            self.hm_items.get(expenditure)
        }
    }

    impl Display for Distribution<'_> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let mut result = format!(
                "Распределение дохода {} от {:?}\n",
                &self.income.source.name,
                &self.income.date.date_naive()
            );
            for (k, v) in &self.hm_items {
                let row = format!("{:20} - {:}\n", k.name, v);
                result.push_str(row.as_str());
            }
            result.push_str(format!("{:20} - {:}", "Остаток", self.rest).as_str());
            write!(f, "{}", result)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use distributor::*;

    use super::*;

    fn default_money() -> Money {
        Money::new(dec!(100.0), Currency::RUB)
    }

    fn default_source() -> IncomeSource {
        IncomeSource::new("salary".to_string(), default_money())
    }

    #[test]
    fn unknown_income_source() {
        let default = default_source();
        let other_source = IncomeSource::new("other".to_string(), default_money());
        let plan = Plan::try_build(vec![&default], vec![]).unwrap();
        let income = Income::new(
            &other_source,
            Default::default(),
            Money::new(dec!(100.0), Currency::RUB),
        );
        assert_eq!(
            plan.new_income(&income),
            Err(DistributeError::UnknownIncomeSource)
        );
    }

    #[test]
    fn zero_income() {
        let default = default_source();
        let income = Income::new(
            &default,
            Default::default(),
            Money::new(Decimal::ZERO, Currency::RUB),
        );
        let plan = Plan::try_build(vec![&default], vec![]).unwrap();

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(Decimal::ZERO, Currency::RUB),
                HashMap::new(),
            ))
        );
    }

    #[test]
    fn no_expensive() {
        let binding = default_source();
        let income = Income::new(
            &binding,
            Default::default(),
            Money::new(dec!(100.00), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&binding], vec![]).unwrap();

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(100.0), Currency::RUB),
                HashMap::new(),
            ))
        );
    }

    #[test]
    fn expensive_less_than_income() {
        let binding = default_source();
        let income = Income::new(
            &binding,
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expenditure::new(
            "Коммуналка".to_string(),
            Money::new(dec!(1.5), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&binding], vec![&expenditure]).unwrap();

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(49.25), Currency::RUB),
                HashMap::from([(&expenditure, Money::new(dec!(0.75), Currency::RUB))]),
            ))
        )
    }

    #[test]
    fn expensive_eq_income() {
        let source = default_source();
        let income = Income::new(
            &source,
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expenditure::new(
            "Коммуналка".to_string(),
            Money::new(dec!(50), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![&expenditure]).unwrap();

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(25), Currency::RUB),
                HashMap::from([(&expenditure, Money::new(dec!(25), Currency::RUB))]),
            ))
        );
    }

    #[test]
    fn expensive_more_than_income() {
        let source = default_source();
        let income = Income::new(
            &source,
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expenditure::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![&expenditure]).unwrap();

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(0), Currency::RUB),
                HashMap::from([(&expenditure, Money::new(dec!(50), Currency::RUB))]),
            ))
        );
    }

    #[test]
    fn add_expenditure_to_distribution_without_plan() {
        let source = default_source();
        let income = Income::new(
            &source,
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expenditure::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let other = Expenditure::new(
            "На пряники".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![&expenditure]).unwrap();
        let mut distribution = Distribution::from_income(&plan, &income);

        assert_eq!(
            distribution.add(&other, Money::new(dec!(50), Currency::RUB)),
            Err(DistributeError::UnknownExpenditure)
        );

        assert_eq!(distribution.get(&other), None);
        assert_eq!(*distribution.rest(), Money::new(dec!(50), Currency::RUB))
    }

    #[test]
    fn add_expenditure_to_distribution() {
        let source = default_source();
        let income = Income::new(
            &source,
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expenditure::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![&expenditure]).unwrap();
        let mut distribution = Distribution::from_income(&plan, &income);

        distribution
            .add(&expenditure, Money::new(dec!(50), Currency::RUB))
            .unwrap();

        assert_eq!(
            distribution.get(&expenditure),
            Some(Money::new(dec!(50), Currency::RUB)).as_ref()
        );
        assert_eq!(*distribution.rest(), Money::new(dec!(0), Currency::RUB))
    }

    #[test]
    fn add_double_distribution() {
        let source = default_source();
        let income = Income::new(
            &source,
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expenditure::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![&expenditure]).unwrap();
        let mut distribution = Distribution::from_income(&plan, &income);

        distribution
            .add(&expenditure, Money::new(dec!(50), Currency::RUB))
            .unwrap();
        distribution
            .add(&expenditure, Money::new(dec!(50), Currency::RUB))
            .unwrap();

        assert_eq!(
            distribution.get(&expenditure),
            Some(Money::new(dec!(100), Currency::RUB)).as_ref()
        );
        assert_eq!(*distribution.rest(), Money::new(dec!(-50), Currency::RUB));
    }
}

fn main() {
    let zp = IncomeSource::new(
        "Зарплата".to_string(),
        Money::new(dec!(100000), Currency::RUB),
    );
    let rent = Expenditure::new("Аренда".to_string(), Money::new(dec!(25000), Currency::RUB));
    let food = Expenditure::new("Еда".to_string(), Money::new(dec!(17345), Currency::RUB));
    let health = Expenditure::new(
        "Здоровье".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let mortgage = Expenditure::new(
        "Ипотека".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let home_service = Expenditure::new(
        "Коммуналка".to_string(),
        Money::new(dec!(6000), Currency::RUB),
    );
    let bag_month = Expenditure::new(
        "Подушка на месяц".to_string(),
        Money::new(dec!(5000), Currency::RUB),
    );
    let cloth = Expenditure::new("Шмотки".to_string(), Money::new(dec!(2000), Currency::RUB));
    let house_maintenance = Expenditure::new(
        "Бытовые удобства".to_string(),
        Money::new(dec!(1000), Currency::RUB),
    );

    let plan = Plan::try_build(
        vec![&zp],
        vec![
            &rent,
            &food,
            &health,
            &mortgage,
            &home_service,
            &bag_month,
            &cloth,
            &house_maintenance,
        ],
    )
        .unwrap();

    let income = Income::new(&zp, Utc::now(), Money::new(dec!(50000), Currency::RUB));
    print!("{}", plan.new_income(&income).unwrap());
}
