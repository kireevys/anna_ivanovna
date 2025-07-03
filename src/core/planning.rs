use crate::core::finance::{Money, Percentage};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeSource {
    pub name: String,
    pub expected: Money,
}

impl IncomeSource {
    pub fn expected(&self) -> &Money {
        &self.expected
    }
}
impl PartialEq for IncomeSource {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl IncomeSource {
    #[must_use]
    pub fn new(name: String, expected: Money) -> Self {
        Self { name, expected }
    }
}

impl Display for IncomeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.expected)
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyPlan,
    TooBigExpenses,
    InvalidPlan,
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum ExpenseValue {
    RATE { value: Percentage },
    MONEY { value: Money },
}

impl Default for ExpenseValue {
    fn default() -> Self {
        ExpenseValue::MONEY {
            value: Money::default(),
        }
    }
}

impl FromStr for ExpenseValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('%') {
            let percentage =
                Percentage::from_str(s).map_err(|e| format!("Failed to parse percentage: {e}"))?;
            return Ok(ExpenseValue::RATE { value: percentage });
        }

        if s.contains("₽") {
            let money = Money::from_str(s).map_err(|e| format!("Failed to parse money: {e}"))?;
            return Ok(ExpenseValue::MONEY { value: money });
        }

        Err("Invalid format".to_string())
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Expense {
    pub name: String,
    pub value: ExpenseValue,
    pub category: Option<String>,
}

impl Expense {
    pub fn new(name: String, value: ExpenseValue, category: Option<String>) -> Self {
        Self {
            name,
            value,
            category,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub sources: Vec<IncomeSource>,
    budget: HashMap<Expense, Percentage>,
    pub rest: Percentage,
}

impl Debug for Plan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let expenses_count = self.budget.len();
        f.debug_struct("Plan")
            .field("sources_count", &self.sources.len())
            .field("expenses_count", &expenses_count)
            .field("rest", &self.rest)
            .finish()
    }
}

#[cfg(test)]
impl Clone for Plan {
    fn clone(&self) -> Self {
        Self {
            sources: self.sources.clone(),
            budget: self.budget.clone(),
            rest: self.rest.clone(),
        }
    }
}

impl<'a> IntoIterator for &'a Plan {
    type Item = (&'a Expense, &'a Percentage);
    type IntoIter = std::collections::hash_map::Iter<'a, Expense, Percentage>;

    fn into_iter(self) -> Self::IntoIter {
        self.budget.iter()
    }
}

impl Deref for Plan {
    type Target = HashMap<Expense, Percentage>;

    fn deref(&self) -> &Self::Target {
        &self.budget
    }
}

impl Plan {
    pub fn has_source(&self, source: &IncomeSource) -> bool {
        self.sources.contains(source)
    }

    /// Группирует расходы по категориям
    pub fn categories(&self) -> impl Iterator<Item = (String, Vec<&Expense>)> {
        let mut sorted_expenses: Vec<_> = self.budget.keys().collect();
        sorted_expenses.sort_by_key(|e| &e.name);

        let mut category_map: BTreeMap<String, Vec<&Expense>> = BTreeMap::new();

        for expense in sorted_expenses {
            let category_name = expense
                .category
                .as_deref()
                .unwrap_or("Без категории")
                .to_string();
            category_map.entry(category_name).or_default().push(expense);
        }

        category_map.into_iter()
    }
}

impl Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "План бюджета:")?;

        // Источники дохода
        writeln!(f, "├─ 💸 Источники дохода:")?;
        let sources_len = self.sources.len();
        for (i, source) in self.sources.iter().enumerate() {
            let prefix = if i + 1 == sources_len {
                "└──"
            } else {
                "├──"
            };
            writeln!(f, "│   {prefix} {source}")?;
        }

        let total_income = self.sources.iter().map(|s| s.expected).sum::<Money>();
        let rest_amount = Money::new_rub(self.rest.apply_to(total_income.value));
        writeln!(f, "│")?;
        writeln!(f, "├─ 🏦 Остаток: {:<25} [{}]", rest_amount, self.rest)?;
        writeln!(f, "│")?;
        writeln!(f, "└─ Запланированные расходы:")?;

        let categories: Vec<_> = self.categories().collect();
        let cat_len = categories.len();
        for (ci, (category_name, expenses)) in categories.into_iter().enumerate() {
            let cat_prefix = if ci + 1 == cat_len {
                "    └──"
            } else {
                "    ├──"
            };
            let cat_emoji = if category_name == "Без категории" {
                "📦"
            } else {
                "📂"
            };
            writeln!(f, "{cat_prefix} {cat_emoji} {category_name}")?;
            let exp_len = expenses.len();
            let mut cat_total_amount = Money::new_rub(Decimal::ZERO);
            let mut cat_total_percent = Percentage::ZERO;
            for (ei, expense) in expenses.iter().enumerate() {
                let exp_prefix = if ei + 1 == exp_len {
                    "        └──"
                } else {
                    "        ├──"
                };
                if let Some(percentage) = self.budget.get(expense) {
                    let estimated_amount = Money::new_rub(percentage.apply_to(total_income.value));
                    cat_total_amount += estimated_amount;
                    cat_total_percent += percentage.clone();
                    writeln!(
                        f,
                        "{exp_prefix} {:<25} {} [{}]",
                        expense.name, estimated_amount, percentage
                    )?;
                }
            }
            writeln!(
                f,
                "         💰 {cat_total_amount:<25} [{cat_total_percent}]"
            )?;
        }
        Ok(())
    }
}

impl TryFrom<Draft> for Plan {
    type Error = Error;

    /// Создает План из Черновика
    ///
    /// # Arguments
    ///
    /// * `draft`: Черновик
    ///
    /// # Errors
    /// `EmptyPlan`: Нельзя создать План из пустого Черновика
    /// `TooBigExpenses`: Нужно запланировать свои Расходы так, чтобы они не превышали Доходы
    ///
    /// returns: Result<Plan, Error>
    ///
    fn try_from(draft: Draft) -> Result<Self, Self::Error> {
        if draft.expenses.is_empty() || draft.sources.is_empty() {
            return Err(Error::EmptyPlan);
        }

        let plan_total = draft.total_incomes();
        let mut rate_plan = HashMap::with_capacity(draft.expenses.len());
        let mut total = Percentage::ZERO;
        for e in draft.expenses {
            let current = match &e.value {
                ExpenseValue::MONEY { value } => Percentage::of(value.value, plan_total.value),
                ExpenseValue::RATE { value } => value.clone(),
            };
            total += current.clone();
            if total > Percentage::ONE_HUNDRED {
                return Err(Error::TooBigExpenses);
            }
            rate_plan.insert(e.clone(), current);
        }
        Ok(Self {
            sources: draft.sources.clone(),
            budget: rate_plan,
            rest: Percentage::ONE_HUNDRED - total,
        })
    }
}

#[derive(Clone)]
pub struct Draft {
    pub sources: Vec<IncomeSource>,
    pub expenses: Vec<Expense>,
}

impl Default for Draft {
    fn default() -> Self {
        Self::new()
    }
}

impl Draft {
    pub fn add_expense(&mut self, expense: Expense) {
        let () = &self.expenses.push(expense);
    }
    pub fn add_source(&mut self, income_source: IncomeSource) {
        let () = &self.sources.push(income_source);
    }
    #[must_use]
    pub fn new() -> Self {
        Self {
            sources: vec![],
            expenses: vec![],
        }
    }
    #[must_use]
    pub fn build(sources: &[IncomeSource], expenses: &[Expense]) -> Self {
        let mut draft = Self::new();
        sources.iter().for_each(|s| draft.add_source(s.clone()));
        expenses.iter().for_each(|e| draft.add_expense(e.clone()));
        draft
    }

    #[must_use]
    pub fn total_incomes(&self) -> Money {
        self.sources
            .iter()
            .map(|s| s.expected)
            .fold(Money::new_rub(Decimal::ZERO), |acc, income| acc + income)
    }
}

#[cfg(test)]
mod test_planning {
    use std::collections::HashMap;

    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal_macros::dec;

    use crate::core::finance::{Currency, Money, Percentage};

    use super::*;

    #[test]
    fn new_plan() {
        let draft = Draft::new();
        assert_eq!(draft.expenses, vec![]);
        assert_eq!(draft.sources, vec![]);
    }

    #[test]
    fn add_source() {
        let mut draft = Draft::new();
        let source =
            IncomeSource::new("Gold goose".to_string(), Money::new(dec!(1), Currency::RUB));
        draft.add_source(source.clone());
        assert_eq!(draft.sources, vec![source]);
        assert_eq!(draft.expenses, vec![]);
    }

    #[test]
    fn add_value_expense() {
        let mut draft = Draft::new();
        let expense = Expense::new(
            "Black hole".to_string(),
            ExpenseValue::MONEY {
                value: Money::new(dec!(1), Currency::RUB),
            },
            None,
        );
        draft.add_expense(expense.clone());
        assert_eq!(draft.expenses, vec![expense]);
        assert_eq!(draft.sources, vec![]);
    }

    #[test]
    fn add_rate_expense() {
        let mut draft = Draft::new();
        let expense = Expense::new(
            "Black hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(10),
            },
            None,
        );
        draft.add_expense(expense.clone());
        assert_eq!(draft.expenses, vec![expense]);
        assert_eq!(draft.sources, vec![]);
    }

    fn rub(v: f64) -> Money {
        Money::new_rub(Decimal::from_f64(v).unwrap())
    }

    #[test]
    fn test_empty_draft() {
        let draft = Draft::new();
        assert_eq!(Plan::try_from(draft), Err(Error::EmptyPlan));
    }

    #[test]
    fn no_expenses() {
        let draft = Draft::build(
            &[IncomeSource::new("Gold goose".to_string(), rub(1.0))],
            &[],
        );
        assert_eq!(Plan::try_from(draft), Err(Error::EmptyPlan));
    }

    #[test]
    fn build_rate_plan_from_rate_expense() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        let res = Plan::try_from(draft).unwrap();
        let expected = HashMap::from([(expense.clone(), Percentage::from_int(100))]);

        assert_eq!(
            res,
            Plan {
                sources: vec![source.clone()],
                budget: expected,
                rest: Percentage::from_int(0),
            }
        );
    }

    #[test]
    fn build_plan_with_overhead_percent() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(101),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);
        assert_eq!(Plan::try_from(draft), Err(Error::TooBigExpenses));
    }

    #[test]
    fn build_rate_with_overhead_total_percent() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense_1 = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(100),
            },
            None,
        );
        let expense_2 = Expense::new(
            "Little bit".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(dec!(0.01)),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense_1.clone(), expense_2.clone()]);
        assert_eq!(Plan::try_from(draft), Err(Error::TooBigExpenses));
    }

    #[test]
    fn build_rate_when_has_rest() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(50),
            },
            None,
        );
        let draft = Draft::build(&[source.clone()], &[expense.clone()]);

        let expected = HashMap::from([(expense.clone(), Percentage::from_int(50))]);
        assert_eq!(
            Plan::try_from(draft).unwrap(),
            Plan {
                sources: vec![source.clone()],
                budget: expected,
                rest: Percentage::HALF,
            }
        );
    }

    #[test]
    fn build_full_plan() {
        let source = IncomeSource::new("Gold goose".to_string(), rub(1.0));
        let expense_1 = Expense::new(
            "Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.25) },
            None,
        );
        let expense_2 = Expense::new(
            "Yet Another Black Hole".to_string(),
            ExpenseValue::MONEY { value: rub(0.5) },
            None,
        );
        let expense_3 = Expense::new(
            "Rate Black Hole".to_string(),
            ExpenseValue::RATE {
                value: Percentage::QUARTER,
            },
            None,
        );
        let draft = Draft::build(
            &[source.clone()],
            &[expense_1.clone(), expense_2.clone(), expense_3.clone()],
        );
        let res = Plan::try_from(draft).unwrap();
        let expected = HashMap::from([
            (expense_1.clone(), Percentage::QUARTER),
            (expense_2.clone(), Percentage::HALF),
            (expense_3.clone(), Percentage::QUARTER),
        ]);

        assert_eq!(
            res,
            Plan {
                sources: vec![source.clone()],
                budget: expected,
                rest: Percentage::ZERO,
            }
        );
    }

    #[test]
    fn test_plan_display() {
        let source = IncomeSource::new("Зарплата".to_string(), rub(100000.0));
        let expense_1 = Expense::new(
            "Аренда".to_string(),
            ExpenseValue::MONEY {
                value: rub(25000.0),
            },
            Some("Жизнеобеспечение".to_string()),
        );
        let expense_2 = Expense::new(
            "Продукты".to_string(),
            ExpenseValue::MONEY {
                value: rub(20000.0),
            },
            Some("Питание".to_string()),
        );
        let expense_3 = Expense::new(
            "Развлечения".to_string(),
            ExpenseValue::RATE {
                value: Percentage::from_int(15),
            },
            None,
        );

        let draft = Draft::build(
            &[source.clone()],
            &[expense_1.clone(), expense_2.clone(), expense_3.clone()],
        );
        let plan = Plan::try_from(draft).unwrap();

        println!("{plan}");
    }
}
