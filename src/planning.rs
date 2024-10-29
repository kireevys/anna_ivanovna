pub mod planning {
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use crate::finance::{Money, Percentage};

    #[derive(Debug, Clone)]
    pub struct IncomeSource {
        id: Uuid,
        pub name: String,
        pub expected: Money,
    }

    #[derive(Debug)]
    pub enum PlanningError {}

    impl PartialEq for IncomeSource {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    #[derive(PartialEq, Debug, Clone, Eq, Hash)]
    pub enum ExpenseValue {
        RATE { value: Percentage },
        MONEY { value: Money },
    }

    impl ExpenseValue {
        pub fn to_decimal(&self, sum: Decimal) -> Decimal {
            match self {
                ExpenseValue::RATE { value } => { value.apply_to(sum) }
                ExpenseValue::MONEY { value } => { value.value }
            }
        }
    }

    #[derive(PartialEq, Debug, Clone, Eq, Hash)]
    pub struct Expense {
        uuid: Uuid,
        pub name: String,
        pub value: ExpenseValue,
    }

    impl Expense {
        pub fn new(name: String, value: ExpenseValue) -> Self {
            Self {
                uuid: Uuid::new_v4(),
                name,
                value,
            }
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

    pub struct Plan {
        pub uuid: Uuid,
        pub sources: Vec<IncomeSource>,
        pub expenses: Vec<Expense>,
    }

    impl Plan {
        pub fn add_expense(&mut self, expense: Expense) -> Result<(), PlanningError> {
            let _ = &self.expenses.push(expense);
            Ok(())
        }
        pub fn add_source(&mut self, income_source: IncomeSource) -> Result<(), PlanningError> {
            let _ = &self.sources.push(income_source);
            Ok(())
        }

        pub fn new() -> Self {
            Self {
                uuid: Uuid::new_v4(),
                sources: vec![],
                expenses: vec![],
            }
        }

        pub fn try_build(
            uuid: Uuid,
            sources: Vec<IncomeSource>,
            expenses: Vec<Expense>,
        ) -> Result<Self, PlanningError> {
            // TODO: Vec<PlanningError>?
            let mut plan = Self {
                uuid,
                sources: vec![],
                expenses: vec![],
            };
            sources
                .iter()
                .try_for_each(|s| plan.add_source(s.clone()))?;
            expenses
                .iter()
                .try_for_each(|e| plan.add_expense(e.clone()))?;
            Ok(plan)
        }

        pub fn total_incomes(&self) -> Money {
            let mut total_rub = Money::new_rub(Decimal::ZERO);
            for s in self.sources.iter() {
                total_rub += s.expected
            }
            total_rub
        }
    }
}


#[cfg(test)]
mod test_planning {
    use rust_decimal_macros::dec;

    use crate::finance::{Currency, Money, Percentage};
    use crate::planning::planning::{Expense, ExpenseValue, IncomeSource, Plan};

    #[test]
    fn new_plan() {
        let plan = Plan::new();
        assert_eq!(plan.expenses, vec![]);
        assert_eq!(plan.sources, vec![]);
    }

    #[test]
    fn add_source() {
        let mut plan = Plan::new();
        let source =
            IncomeSource::new("Gold goose".to_string(), Money::new(dec!(1), Currency::RUB));
        let _ = plan.add_source(source.clone()).unwrap();
        assert_eq!(plan.sources, vec![source]);
        assert_eq!(plan.expenses, vec![]);
    }

    #[test]
    fn add_value_expense() {
        let mut plan = Plan::new();
        let expense = Expense::new(
            "Black hole".to_string(),
            ExpenseValue::MONEY {
                value: Money::new(dec!(1), Currency::RUB),
            },
        );
        let _ = plan.add_expense(expense.clone()).unwrap();
        assert_eq!(plan.expenses, vec![expense]);
        assert_eq!(plan.sources, vec![]);
    }

    #[test]
    fn add_rate_expense() {
        let mut plan = Plan::new();
        let expense = Expense::new(
            "Black hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(10),
            },
        );
        let _ = plan.add_expense(expense.clone()).unwrap();
        assert_eq!(plan.expenses, vec![expense]);
        assert_eq!(plan.sources, vec![]);
    }
}
