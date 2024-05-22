pub mod distributor {
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::ops::{Add, AddAssign, Sub, SubAssign};

    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

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

    #[derive(Debug, PartialEq, Clone)]
    pub struct Plan<'a> {
        uuid: Uuid,
        incomes: Vec<&'a IncomeSource>,
        expenses: Vec<Expense>,
        targets: Vec<Target>,
    }

    impl<'a> Plan<'a> {
        pub fn try_build(
            incomes: Vec<&'a IncomeSource>,
            expenses: Vec<Expense>,
            targets: Vec<Target>,
        ) -> Result<Self, DistributeError> {
            if incomes.len() == 0 {
                return Err(DistributeError::NoIncomes);
            }
            Ok(Self {
                uuid: Uuid::new_v4(),
                incomes,
                expenses,
                targets,
            })
        }

        pub fn new_target(
            &self,
            name: String,
            goal: Money,
            wished_percent: Percentage,
        ) -> Result<Self, DistributeError> {
            let target = Expense::new(name, Money::new(dec!(10_000), Currency::RUB));
            let mut expenses = self.expenses.clone();
            expenses.push(target);
            Self::try_build(self.incomes.clone(), expenses, vec![])
        }

        fn contains_expenditure(&'a self, expenditure: &'a Expense) -> Option<&Expense> {
            match &self.expenses.iter().find(|p| **p == *expenditure) {
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

        fn distribute(&'a self, expense: &'a Expense, income: &Income) -> Money {
            let p = Percentage::how(
                expense.money.value,
                self.expected_income(income.currency()).value,
            );
            Money::new(p.apply_to(income.money.value), income.currency())
        }

        pub fn new_income(&self, income: &'a Income) -> Result<Distribution, DistributeError> {
            match &self.incomes.iter().find(|p| ***p == income.source) {
                Some(_) => {
                    let mut d = Distribution::from_income(&self, income);
                    for expense in self.expenses.iter() {
                        let _d = self.distribute(expense, income);
                        d.add(&expense, _d).expect("Cannot add Expense");
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

    #[derive(Debug, Clone)]
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
    pub struct Income {
        source: IncomeSource,
        pub date: DateTime<Utc>,
        money: Money,
    }

    impl Income {
        pub fn new(source: IncomeSource, date: DateTime<Utc>, money: Money) -> Self {
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

    #[derive(PartialEq, Eq, Hash, Debug, Clone)]
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

    #[derive(PartialEq, Eq, Hash, Debug, Clone)]
    pub struct Expense {
        name: String,
        money: Money,
    }

    impl Expense {
        pub fn new(name: String, money: Money) -> Expense {
            Self { name, money }
        }
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct Distribution<'a> {
        plan: &'a Plan<'a>,
        income: &'a Income,
        rest: Money,
        expenditures: HashMap<&'a Expense, Money>,
    }

    impl<'a> Distribution<'a> {
        pub fn rest(&self) -> &Money {
            &self.rest
        }
        pub fn new(
            plan: &'a Plan,
            income: &'a Income,
            rest: Money,
            expenditures: HashMap<&'a Expense, Money>,
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
                expenditures: HashMap::with_capacity(plan.expenses.len() + plan.targets.len()),
            }
        }

        pub fn add(&mut self, expense: &'a Expense, money: Money) -> Result<(), DistributeError> {
            match self.plan.contains_expenditure(expense) {
                Some(_) => {
                    self.expenditures
                        .entry(expense)
                        .and_modify(|e| *e += money)
                        .or_insert(money);
                    self.rest -= money;
                    Ok(())
                }
                None => Err(DistributeError::UnknownExpenditure),
            }
        }

        pub fn get(&self, expense: &Expense) -> Option<&Money> {
            self.expenditures.get(expense)
        }
    }

    impl Display for Distribution<'_> {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let mut result = format!(
                "Распределение по плану {} дохода {} от {:?} на сумму {}\n",
                &self.plan.uuid,
                &self.income.source.name,
                &self.income.date.date_naive(),
                &self.income.money
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

    use chrono::Utc;
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
        let plan = Plan::try_build(vec![&default], vec![], vec![]).unwrap();
        let income = Income::new(
            other_source.clone(),
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
            default.clone(),
            Default::default(),
            Money::new(Decimal::ZERO, Currency::RUB),
        );
        let plan = Plan::try_build(vec![&default], vec![], vec![]).unwrap();

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
            binding.clone(),
            Default::default(),
            Money::new(dec!(100.00), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&binding], vec![], vec![]).unwrap();

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
            binding.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(1.5), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&binding], vec![expenditure.clone()], vec![]).unwrap();

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
    fn base_distribute() {
        let zp = IncomeSource::new(
            "Зарплата".to_string(),
            Money::new(dec!(100_000), Currency::RUB),
        );
        let home_service = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(6000), Currency::RUB),
        );
        let food = Expense::new("Бюджет".to_string(), Money::new(dec!(17340), Currency::RUB));
        let health = Expense::new(
            "Здоровье".to_string(),
            Money::new(dec!(2000), Currency::RUB),
        );
        let mortgage = Expense::new(
            "Ипотека".to_string(),
            Money::new(dec!(15000), Currency::RUB),
        );
        let bag_month = Expense::new(
            "Подушка на месяц".to_string(),
            Money::new(dec!(5900), Currency::RUB),
        );

        let auto = Target::new(
            "Автомобиль".to_string(),
            Money::new(dec!(1_000_000), Currency::RUB),
            Percentage::from_int(10),
        );

        let mut plan =
            Plan::try_build(vec![&zp], vec![home_service.clone()], vec![auto.clone()]).unwrap();

        // let plan = plan
        //     .new_target(
        //         "Автомобиль".to_string(),
        //         Money::new(dec!(1_000_000), Currency::RUB),
        //         Percentage::from_int(10),
        //     )
        //     .unwrap();

        let income = Income::new(
            zp.clone(),
            Utc::now(),
            Money::new(dec!(100000), Currency::RUB),
        );

        let result = plan.new_income(&income).unwrap();
        let expected = Distribution::new(
            &plan,
            &income,
            Money::new(dec!(94000.00), Currency::RUB),
            HashMap::from([
                (&home_service, Money::new(dec!(6000), Currency::RUB)),
                // (&mortgage, Money::new(dec!(15000), Currency::RUB)),
                // (&bag_month, Money::new(dec!(5900), Currency::RUB)),
                // (&food, Money::new(dec!(17340), Currency::RUB)),
                // (&health, Money::new(dec!(2000), Currency::RUB)),
                // (&auto, Money::new(dec!(10_000), Currency::RUB)),
            ]),
        );

        assert_eq!(
            result.clone(),
            expected.clone(),
            "\nResult:\n{result}\n-----\nExpected:\n{expected}",
        );
    }

    #[test]
    fn expensive_eq_income() {
        let source = default_source();
        let income = Income::new(
            source.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(50), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![expenditure.clone()], vec![]).unwrap();

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
            source.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![expenditure.clone()], vec![]).unwrap();

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
            source.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let other = Expense::new(
            "На пряники".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![expenditure], vec![]).unwrap();
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
            source.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![expenditure.clone()], vec![]).unwrap();
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
            source.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![&source], vec![expenditure.clone()], vec![]).unwrap();
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

    #[cfg(test)]
    mod percentage {
        use rust_decimal_macros::dec;

        use crate::distributor::Percentage;

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
}
