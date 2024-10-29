pub mod distributor {
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::ops::Add;

    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use crate::finance::{Currency, Money, Percentage};

    #[derive(Debug, PartialEq, Clone)]
    pub struct Plan {
        uuid: Uuid,
        incomes: Vec<IncomeSource>,
        survivals: Vec<Absolute>,
        targets: Vec<Target>,
    }

    impl Plan {
        pub fn try_build(
            incomes: Vec<IncomeSource>,
            survivals: Vec<Absolute>,
            targets: Vec<Target>,
        ) -> Result<Self, DistributeError> {
            if incomes.len() == 0 {
                return Err(DistributeError::NoIncomes);
            }

            Ok(Self {
                uuid: Uuid::new_v4(),
                incomes,
                survivals,
                targets,
            })
        }
        fn total(&self) -> Decimal {
            let mut sum = dec!(0);
            for income in self.incomes.iter() {
                sum += income.expected.value;
            }
            sum
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

        fn distribute(&self, expense: &Expense, income: &Income) -> Money {
            let p = Percentage::how(
                expense.money.value,
                self.expected_income(income.currency()).value,
            );
            Money::new(p.apply_to(income.money.value), income.currency())
        }

        pub fn new_income<'a>(
            &'a self,
            income: &'a Income,
        ) -> Result<Distribution, DistributeError> {
            match &self.incomes.iter().find(|p| **p == income.source) {
                Some(_) => {
                    let mut d = Distribution::from_income(&self, income);
                    for survival in self.survivals.iter() {
                        let expense = Expense::new(survival.name.clone(), survival.money);
                        let money = self.distribute(&expense, income);
                        //
                        d.add(expense, money).expect("Cannot add Expense");
                    }

                    for target in self.targets.iter() {
                        let expense = Expense::new(
                            target.name.clone(),
                            Money::new(target.rate.apply_to(self.total()), Currency::RUB),
                        );
                        let money = self.distribute(&expense, income);
                        //
                        d.add(expense, money).expect("Cannot add Expense");
                    }
                    Ok(d)
                }
                None => Err(DistributeError::UnknownIncomeSource),
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
    pub struct Absolute {
        name: String,
        money: Money,
    }

    impl Absolute {
        pub fn new(name: String, money: Money) -> Absolute {
            Self { name, money }
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
        plan: &'a Plan,
        income: &'a Income,
        rest: Money,
        expenditures: HashMap<Expense, Money>,
    }

    impl<'a> Distribution<'a> {
        pub fn rest(&self) -> &Money {
            &self.rest
        }
        pub fn new(
            plan: &'a Plan,
            income: &'a Income,
            rest: Money,
            expenditures: HashMap<Expense, Money>,
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
                expenditures: HashMap::with_capacity(plan.survivals.len() + plan.targets.len()),
            }
        }

        pub fn add(&mut self, expense: Expense, money: Money) -> Result<(), DistributeError> {
            self.expenditures
                .entry(expense)
                .and_modify(|e| *e += money)
                .or_insert(money);
            self.rest -= money;
            Ok(())
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
    use distributor::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::finance::{Currency, Money};
    use crate::finance::Percentage;

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
        let plan = Plan::try_build(vec![default], vec![], vec![]).unwrap();
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
        let plan = Plan::try_build(vec![default], vec![], vec![]).unwrap();

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
        let plan = Plan::try_build(vec![binding], vec![], vec![]).unwrap();

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
        let expenditure = Absolute::new(
            "Коммуналка".to_string(),
            Money::new(dec!(1.5), Currency::RUB),
        );
        let plan = Plan::try_build(vec![binding], vec![expenditure.clone()], vec![]).unwrap();
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(1.5), Currency::RUB),
        );
        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(49.25), Currency::RUB),
                HashMap::from([(expenditure, Money::new(dec!(0.75), Currency::RUB))]),
            ))
        )
    }

    #[test]
    fn base_distribute() {
        let zp = IncomeSource::new(
            "Зарплата".to_string(),
            Money::new(dec!(100_000), Currency::RUB),
        );
        let home_service = Absolute::new(
            "Коммуналка".to_string(),
            Money::new(dec!(6000), Currency::RUB),
        );
        let auto = Target::new(
            "Авто".to_string(),
            Money::new(dec!(1_000_000), Currency::RUB),
            Percentage::from_int(10),
        );
        let plan = Plan::try_build(vec![zp.clone()], vec![home_service], vec![auto]).unwrap();
        let income = Income::new(
            zp.clone(),
            Utc::now(),
            Money::new(dec!(100_000), Currency::RUB),
        );

        let result = plan.new_income(&income).unwrap();

        let e_auto = Expense::new("Авто".to_string(), Money::new(dec!(10_000), Currency::RUB));
        let e_home_service = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(6000), Currency::RUB),
        );
        let expected = Distribution::new(
            &plan,
            &income,
            Money::new(dec!(84000.00), Currency::RUB),
            HashMap::from([
                (e_home_service, Money::new(dec!(6000), Currency::RUB)),
                // (&mortgage, Money::new(dec!(15000), Currency::RUB)),
                // (&bag_month, Money::new(dec!(5900), Currency::RUB)),
                // (&food, Money::new(dec!(17340), Currency::RUB)),
                // (&health, Money::new(dec!(2000), Currency::RUB)),
                (e_auto, Money::new(dec!(10_000), Currency::RUB)),
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
        let expenditure = Absolute::new(
            "Коммуналка".to_string(),
            Money::new(dec!(50), Currency::RUB),
        );
        let plan = Plan::try_build(vec![source], vec![expenditure.clone()], vec![]).unwrap();
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(50), Currency::RUB),
        );

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(25), Currency::RUB),
                HashMap::from([(expenditure, Money::new(dec!(25), Currency::RUB))]),
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
        let expenditure = Absolute::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![source], vec![expenditure.clone()], vec![]).unwrap();
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );

        assert_eq!(
            plan.new_income(&income),
            Ok(Distribution::new(
                &plan,
                &income,
                Money::new(dec!(0), Currency::RUB),
                HashMap::from([(expenditure, Money::new(dec!(50), Currency::RUB))]),
            ))
        );
    }

    #[test]
    fn add_expenditure_to_distribution() {
        let source = default_source();
        let income = Income::new(
            source.clone(),
            Default::default(),
            Money::new(dec!(50.0), Currency::RUB),
        );
        let expenditure = Absolute::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![source], vec![expenditure.clone()], vec![]).unwrap();
        let mut distribution = Distribution::from_income(&plan, &income);
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        distribution
            .add(expenditure.clone(), Money::new(dec!(50), Currency::RUB))
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
        let expenditure = Absolute::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        let plan = Plan::try_build(vec![source], vec![expenditure.clone()], vec![]).unwrap();
        let mut distribution = Distribution::from_income(&plan, &income);
        let expenditure = Expense::new(
            "Коммуналка".to_string(),
            Money::new(dec!(100), Currency::RUB),
        );
        distribution
            .add(expenditure.clone(), Money::new(dec!(50), Currency::RUB))
            .unwrap();
        distribution
            .add(expenditure.clone(), Money::new(dec!(50), Currency::RUB))
            .unwrap();

        assert_eq!(
            distribution.get(&expenditure),
            Some(Money::new(dec!(100), Currency::RUB)).as_ref()
        );
        assert_eq!(*distribution.rest(), Money::new(dec!(-50), Currency::RUB));
    }
}
