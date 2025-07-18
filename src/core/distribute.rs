use crate::core::finance::{Money, Percentage};
use crate::core::planning::{Expense, IncomeSource, Plan};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

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

    pub fn name(&self) -> &str {
        &self.expense.name
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub income: Income,
    pub rest: Money,
    pub no_category: Vec<BudgetEntry>,
    pub categories: HashMap<String, Vec<BudgetEntry>>,
}

impl Budget {
    pub fn income_date(&self) -> &NaiveDate {
        &self.income.date
    }

    pub fn rest(&self) -> &Money {
        &self.rest
    }
}

impl Debug for Budget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let expenses_count =
            self.no_category.len() + self.categories.values().map(|v| v.len()).sum::<usize>();
        f.debug_struct("Budget")
            .field("date", &self.income.date)
            .field("amount", &self.income.amount)
            .field("rest", &self.rest)
            .field("categories_count", &self.categories.len())
            .field("expenses_count", &expenses_count)
            .finish()
    }
}

impl Budget {
    pub fn new(income: Income) -> Self {
        Self {
            rest: income.amount,
            income,
            no_category: Vec::new(),
            categories: HashMap::new(),
        }
    }

    pub fn push(&mut self, category: Option<String>, entry: BudgetEntry) {
        self.rest -= entry.amount;
        if let Some(category) = category {
            self.categories.entry(category).or_default().push(entry);
        } else {
            self.no_category.push(entry);
        }
    }

    fn calculate(&mut self, expense: Expense, rate: &Percentage) {
        let money = Money::new_rub(rate.apply_to(self.income.amount.value));
        self.push(expense.category.clone(), BudgetEntry::new(expense, money));
    }
}

impl Display for Budget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Распределение дохода:")?;
        writeln!(f, "├── 💸 Источник: {}", &self.income.source)?;
        writeln!(f, "│   Сумма: {}", &self.income.amount)?;
        writeln!(f, "│   Дата: {}", &self.income.date)?;
        writeln!(f, "│")?;

        let mut sorted_categories: Vec<_> = self.categories.iter().collect();
        sorted_categories.sort_by_key(|(category, _)| *category);
        let has_no_category = !self.no_category.is_empty();
        let cat_len = sorted_categories.len();
        let branch_count = cat_len + if has_no_category { 1 } else { 0 };
        let mut branch_idx = 0;
        // Сначала выводим "Без категории"
        if has_no_category {
            branch_idx += 1;
            let no_cat_prefix = if branch_idx == branch_count && cat_len == 0 {
                "└──"
            } else {
                "├──"
            };
            writeln!(f, "{no_cat_prefix} 📦 Без категории")?;
            let mut no_cat_entries = self.no_category.clone();
            no_cat_entries.sort_by_key(|entry| entry.expense.name.clone());
            let exp_len = no_cat_entries.len();
            for (ei, entry) in no_cat_entries.iter().enumerate() {
                let exp_prefix = if ei + 1 == exp_len && cat_len == 0 {
                    "    └──"
                } else {
                    "    ├──"
                };
                writeln!(
                    f,
                    "{exp_prefix} {:<23} - {}",
                    entry.expense.name, entry.amount
                )?;
            }
        }

        // Затем категории
        for (category, entries) in sorted_categories.into_iter() {
            branch_idx += 1;
            let cat_prefix = if branch_idx == branch_count {
                "└──"
            } else {
                "├──"
            };
            write!(f, "{cat_prefix} 📂 {category:<25}")?;
            let cat_total = entries.iter().map(|entry| entry.amount).sum::<Money>();
            writeln!(f, "- {cat_total}")?;

            let exp_len = entries.len();
            for (ei, entry) in entries.iter().enumerate() {
                let exp_prefix = if ei + 1 == exp_len {
                    "    └──"
                } else {
                    "    ├──"
                };
                writeln!(f, "{exp_prefix} {}", entry.expense.name)?;
            }
        }
        writeln!(f, "└── 🏦 Остаток{:17} -[{}]", "", self.rest)
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

    use crate::core::distribute::{Budget, Error, Income, distribute};
    use crate::core::finance::{Money, Percentage};
    use crate::core::planning::{Draft, Expense, ExpenseValue, IncomeSource, Plan};

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
