use crate::finance::{Money, Percentage};
use crate::planning::{Expense, IncomeSource, Plan};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    UnknownSource,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Income {
    pub source: IncomeSource,
    pub amount: Money,
    pub date: NaiveDate,
}

impl Display for Income {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} от {}", &self.amount, &self.date)
    }
}

impl Income {
    #[must_use]
    pub fn new(source: IncomeSource, money: Money, date: NaiveDate) -> Self {
        Self {
            source,
            amount: money,
            date,
        }
    }
    #[must_use]
    pub fn new_today(source: IncomeSource, money: Money) -> Self {
        Self {
            source,
            amount: money,
            date: Utc::now().date_naive(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BudgetEntry {
    pub expense: Expense,
    pub amount: Money,
}

impl BudgetEntry {
    pub fn new(expense: Expense, amount: Money) -> Self {
        Self { expense, amount }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub income: Income,
    pub rest: Money,
    pub no_category: Vec<BudgetEntry>,
    pub categories: HashMap<String, Vec<BudgetEntry>>,
}

impl Budget {
    fn new(income: Income) -> Self {
        Self {
            rest: income.amount,
            income,
            no_category: Vec::new(),
            categories: HashMap::new(),
        }
    }

    fn calculate(&mut self, expense: Expense, rate: &Percentage) {
        let money = Money::new_rub(rate.apply_to(self.income.amount.value));

        if let Some(category) = &expense.category {
            self.categories
                .entry(category.clone())
                .or_default()
                .push(BudgetEntry::new(expense, money));
        } else {
            self.no_category.push(BudgetEntry::new(expense, money));
        }

        self.rest -= money;
    }
}

impl Display for Budget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\nРаспределение дохода {}По источнику {}",
            &self.income, &self.income.source
        )?;

        if !self.no_category.is_empty() {
            let total: Money = self.no_category.iter().map(|entry| entry.amount).sum();
            writeln!(f, "- {:20} - {}", "Без категории", total)?;

            for entry in &self.no_category {
                writeln!(f, "  {:18} - {}", entry.expense.name, entry.amount)?;
            }
        }

        let mut sorted_categories: Vec<_> = self.categories.iter().collect();
        sorted_categories.sort_by_key(|(category, _)| *category);

        for (category, entries) in sorted_categories {
            let total: Money = entries.iter().map(|entry| entry.amount).sum();
            writeln!(f, "- {:20} - {}", category, total)?;

            for entry in entries {
                writeln!(f, "  {:18} - {}", entry.expense.name, entry.amount)?;
            }
        }

        writeln!(f, "- {:20} - {}", "Остаток", self.rest)
    }
}

/// Функция занимается распределением Дохода согласно Плана
///
/// # Arguments
///
/// * `plan`: Запланированный бюджет
/// * `income`: Полученный доход
///
/// # Errors
/// `UnknownSource` - план не содержит Источника полученного Дохода
///
/// returns: Result<Distribute, `DistributeError`>
///
pub fn distribute(plan: &Plan, income: &Income) -> Result<Budget, Error> {
    if !&plan.has_source(&income.source) {
        return Err(Error::UnknownSource);
    }

    let mut d = Budget::new(income.clone());

    plan.into_iter()
        .for_each(|(e, r)| d.calculate(e.clone(), r));

    for entries in d.categories.values_mut() {
        entries.sort_by_key(|entry| entry.expense.name.clone());
    }

    Ok(d)
}

#[cfg(test)]
mod test_distribute {

    use chrono::Utc;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;

    use crate::distribute::{Budget, Error, Income, distribute};
    use crate::finance::{Money, Percentage};
    use crate::planning::{Draft, Expense, ExpenseValue, IncomeSource, Plan};

    fn rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn income_from_unknown_source() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let source_1 = IncomeSource::new("Unknown".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.5) },
            None,
        );
        let draft = Draft::build(&[source], &[expense]);
        let plan = Plan::try_from(draft).unwrap();
        let income = Income::new(source_1, rub(1.0), Utc::now().date_naive());
        assert_eq!(distribute(&plan, &income), Err(Error::UnknownSource));
    }

    #[test]
    fn expense_is_money_and_less_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.5) },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let plan = Plan::try_from(draft).unwrap();
        let income = Income::new_today(source, rub(1.0));
        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(50));
        expected.rest = rub(0.5);
        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expense_is_money_and_more_than_incomes() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(1.0) },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(0.5));
        let plan = Plan::try_from(draft).unwrap();
        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(100));
        expected.rest = rub(0.0);
        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expense_is_full_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(100));
        expected.rest = rub(0.0);
        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expense_is_half_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(50),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(50));
        expected.rest = rub(0.5);
        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expense_is_zero_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(0),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(0));
        expected.rest = rub(1.0);
        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expense_is_one_percent_by_rate() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(1),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();
        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(1));
        expected.rest = rub(0.99);
        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expenses_with_and_without_categories() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));

        let expense_no_category = Expense::new(
            "Еда".to_string(),
            ExpenseValue::MONEY { value: rub(0.3) },
            None,
        );

        let expense_with_category = Expense::new(
            "Развлечения".to_string(),
            ExpenseValue::MONEY { value: rub(0.2) },
            Some("Досуг".to_string()),
        );

        let draft = Draft::build(
            &[source.clone()],
            &[expense_no_category.clone(), expense_with_category.clone()],
        );
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();

        let mut expected = Budget::new(income.clone());
        expected.calculate(expense_no_category.clone(), &Percentage::from_int(30));
        expected.calculate(expense_with_category.clone(), &Percentage::from_int(20));
        expected.rest = rub(0.5);

        assert_eq!(distribute(&plan, &income), Ok(expected));
    }

    #[test]
    fn expense_with_category_only() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));

        let expense = Expense::new(
            "Кино".to_string(),
            ExpenseValue::MONEY { value: rub(0.4) },
            Some("Развлечения".to_string()),
        );

        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let income = Income::new_today(source, rub(1.0));
        let plan = Plan::try_from(draft).unwrap();

        let mut expected = Budget::new(income.clone());
        expected.calculate(expense.clone(), &Percentage::from_int(40));
        expected.rest = rub(0.6);

        assert_eq!(distribute(&plan, &income), Ok(expected));
    }
}
