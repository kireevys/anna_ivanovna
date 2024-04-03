use chrono::Utc;
use rust_decimal_macros::dec;

use crate::distributor::{Currency, Expenditure, Income, IncomeSource, Money, Plan};

mod distributor {
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::ops::{Add, AddAssign, Sub, SubAssign};

    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[derive(Debug, PartialEq)]
    pub struct Percentage(Decimal);

    impl Percentage {
        pub fn from(v: Decimal) -> Self {
            if v > dec!(100) || v < Decimal::ZERO {
                panic!("Percentage value must be between 0 and 100, but it {}", v);
            }
            Self { 0: v }
        }

        pub fn from_int(d: i64) -> Self {
            Percentage::from(Decimal::new(d, 0))
        }

        pub fn how(value: Decimal, on: Decimal) -> Self {
            Self {
                0: value / dec!(100) * on,
            }
        }

        pub fn apply_to(&self, d: Decimal) -> Decimal {
            &self.0 / dec!(100) * d
        }
    }

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

        fn expected_income(&self, currency: Currency) -> Money {
            let mut expected = Money::new(Decimal::ZERO, currency);
            for i in &self.incomes {
                if i.expected.currency == currency {
                    expected += i.expected
                }
            }
            expected
        }

        fn distribute(&'a self, expenditure: &'a Expenditure, income: &Income) -> Money {
            let percentage = Percentage::how(
                self.expected_income(expenditure.money.currency).value,
                expenditure.money.value,
            );
            println!("{:?}", percentage);
            Money::new(percentage.apply_to(income.money.value), income.currency())
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
    pub enum ExpenditureType {
        EXPENSES(Expenditure),
        TARGET,
    }

    pub struct Target {
        name: String,
        goal: Money,
        rate: Percentage,
    }

    impl Target {
        pub fn new(name: String, goal: Money, rate: Percentage) -> Self {
            Self { name, goal, rate }
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
        pub fn new_expenses(name: String, money: Money) -> Self {
            Self { name, money }
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct Distribution<'a> {
        plan: &'a Plan<'a>,
        income: &'a Income<'a>,
        rest: Money,
        expenditures: HashMap<&'a Expenditure, Money>,
    }

    impl<'a> Distribution<'a> {
        pub fn rest(&self) -> &Money {
            &self.rest
        }
        pub fn new(
            plan: &'a Plan,
            income: &'a Income,
            rest: Money,
            expenditures: HashMap<&'a Expenditure, Money>,
        ) -> Self {
            Self {
                plan,
                income,
                rest,
                expenditures,
            }
        }

        pub fn from_income(plan: &'a Plan, income: &'a Income) -> Self {
            Self {
                plan,
                income,
                rest: income.money,
                expenditures: HashMap::with_capacity(plan.expenses.len()),
            }
        }

        pub fn add(
            &mut self,
            expenditure: &'a Expenditure,
            money: Money,
        ) -> Result<(), DistributeError> {
            match self.plan.contains_expenditure(expenditure) {
                Some(_) => {
                    self.expenditures
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
            self.expenditures.get(expenditure)
        }
    }

    impl Display for Distribution<'_> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let mut result = format!(
                "Распределение дохода {} от {:?}\n",
                &self.income.source.name,
                &self.income.date.date_naive()
            );
            for (k, v) in &self.expenditures {
                let row = format!("{:20} - {:}\n", k.name, v);
                result.push_str(row.as_str());
            }
            result.push_str(format!("{:20} - {:}", "Остаток", self.rest).as_str());
            write!(f, "{}", result)
        }
    }
}

#[cfg(test)]
mod core {
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
        let expenditure = Expenditure::new_expenses(
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
        let expenditure = Expenditure::new_expenses(
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
        let expenditure = Expenditure::new_expenses(
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
        let expenditure = Expenditure::new_expenses(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let other = Expenditure::new_expenses(
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
        let expenditure = Expenditure::new_expenses(
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
        let expenditure = Expenditure::new_expenses(
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

    // #[test]
    // fn try_target() {
    //     let source = default_source();
    //     let income = Income::new(
    //         &source,
    //         Default::default(),
    //         Money::new(dec!(50.0), Currency::RUB),
    //     );
    //     let plan = Plan::try_build(vec![&source], vec![]).unwrap();
    //     let target = Target::new(
    //         "Tank".to_string(),
    //         Money::new(dec!(3000000), Currency::RUB),
    //         Percentage::from_int(10),
    //     );
    // }
    //
    // #[test]
    // fn target() {
    //     let source = default_source();
    //     let income = Income::new(
    //         &source,
    //         Default::default(),
    //         Money::new(dec!(50.0), Currency::RUB),
    //     );
    //     let plan = Plan::try_build(vec![&source], vec![]).unwrap();
    //     let target = Target::new(
    //         "Tank".to_string(),
    //         Money::new(dec!(3000000), Currency::RUB),
    //         Percentage::from_int(10),
    //     );
    //     // plan.add_target(&target);
    //
    //     let distribution = plan.new_income(&income).unwrap();
    //
    //     assert_eq!(*distribution.rest(), Money::new(dec!(45), Currency::RUB))
    // }

    #[cfg(test)]
    mod percentage {
        use rust_decimal_macros::dec;

        use crate::distributor::Percentage;

        #[test]
        #[should_panic]
        fn should_be_more_than_0() {
            Percentage::from_int(-1);
        }

        #[test]
        #[should_panic]
        fn should_be_less_than_100() {
            Percentage::from_int(101);
        }

        #[test]
        fn how() {
            assert_eq!(Percentage::how(dec!(5), dec!(100)), Percentage::from_int(5));
            assert_eq!(
                Percentage::how(dec!(1.1), dec!(12)),
                Percentage::from(dec!(0.132))
            );
            assert_eq!(
                Percentage::how(dec!(50), dec!(100)),
                Percentage::from_int(50)
            );
            assert_eq!(Percentage::how(dec!(0), dec!(100)), Percentage::from_int(0));
            assert_eq!(Percentage::how(dec!(10), dec!(0)), Percentage::from_int(0));
            assert_eq!(
                Percentage::how(dec!(100), dec!(100)),
                Percentage::from_int(100)
            );
        }

        #[test]
        fn apply_to() {
            assert_eq!(Percentage::from_int(50).apply_to(dec!(100)), dec!(50));
            assert_eq!(Percentage::from_int(12).apply_to(dec!(100)), dec!(12));
            assert_eq!(Percentage::from_int(1).apply_to(dec!(1)), dec!(0.01));
        }
    }
}

fn main() {
    let zp = IncomeSource::new(
        "Зарплата".to_string(),
        Money::new(dec!(100000), Currency::RUB),
    );
    let rent =
        Expenditure::new_expenses("Аренда".to_string(), Money::new(dec!(25000), Currency::RUB));
    let food = Expenditure::new_expenses("Еда".to_string(), Money::new(dec!(17345), Currency::RUB));
    let health = Expenditure::new_expenses(
        "Здоровье".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let mortgage = Expenditure::new_expenses(
        "Ипотека".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let home_service = Expenditure::new_expenses(
        "Коммуналка".to_string(),
        Money::new(dec!(6000), Currency::RUB),
    );
    let bag_month = Expenditure::new_expenses(
        "Подушка на месяц".to_string(),
        Money::new(dec!(5000), Currency::RUB),
    );
    let cloth =
        Expenditure::new_expenses("Шмотки".to_string(), Money::new(dec!(2000), Currency::RUB));
    let house_maintenance = Expenditure::new_expenses(
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
