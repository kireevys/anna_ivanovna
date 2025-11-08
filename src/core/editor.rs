use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    fmt,
};

use crate::core::{
    finance::{Money, Percentage},
    planning::{Error as PlanningError, Expense, ExpenseValue, IncomeSource, Plan},
};
use thiserror;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Income,
    Expense,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Income => write!(f, "income"),
            EntityType::Expense => write!(f, "expense"),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("{0} {1} Already Exists")]
    AlreadyExists(EntityType, String),
    #[error("{0} {1} Not Found")]
    NotFound(EntityType, String),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Draft {
    pub sources: BTreeMap<String, IncomeSource>,
    pub expenses: BTreeMap<String, Expense>,
}

impl TryFrom<Draft> for Plan {
    type Error = PlanningError;

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
        if draft.is_empty() {
            return Err(Self::Error::EmptyPlan);
        }

        let plan_total = draft.total_incomes();
        let mut rate_plan = HashMap::with_capacity(draft.expenses.len());
        let mut total = Percentage::ZERO;
        for e in draft.expenses.values() {
            let current = match &e.value {
                ExpenseValue::MONEY { value } => Percentage::of(value.value, plan_total.value),
                ExpenseValue::RATE { value } => value.clone(),
            };
            total += current.clone();
            if total > Percentage::ONE_HUNDRED {
                return Err(Self::Error::TooBigExpenses);
            }
            rate_plan.insert(e.clone(), current);
        }
        Ok(Self {
            sources: draft.sources.values().cloned().collect(),
            budget: rate_plan,
            rest: Percentage::ONE_HUNDRED - total,
        })
    }
}

impl Draft {
    pub fn new(
        sources: BTreeMap<String, IncomeSource>,
        expenses: BTreeMap<String, Expense>,
    ) -> Self {
        Self { sources, expenses }
    }

    fn build_map<T, F>(
        items: impl IntoIterator<Item = T>,
        key: F,
        kind: EntityType,
    ) -> Result<BTreeMap<String, T>, Error>
    where
        F: Fn(&T) -> String,
    {
        items
            .into_iter()
            .try_fold(BTreeMap::new(), |mut map, item| {
                let key_str = key(&item);
                if map.insert(key_str.clone(), item).is_some() {
                    return Err(Error::AlreadyExists(kind, key_str));
                }
                Ok(map)
            })
    }

    pub fn build(
        sources: impl IntoIterator<Item = IncomeSource>,
        expenses: impl IntoIterator<Item = Expense>,
    ) -> Result<Self, Error> {
        let sources = Self::build_map(sources, |s| s.name.clone(), EntityType::Income)?;
        let expenses = Self::build_map(expenses, |e| e.name.clone(), EntityType::Income)?;
        Ok(Self { sources, expenses })
    }
    fn total_incomes(&self) -> Money {
        self.sources.values().map(|s| *s.expected()).sum()
    }

    pub fn rest(&self) -> Rest {
        let total = self.total_incomes();
        let (mut m, mut p) = (total, Percentage::TOTAL);
        for e in self.expenses.values() {
            match &e.value {
                ExpenseValue::MONEY { value } => {
                    p -= Percentage::of(value.value, total.value);
                    m -= *value
                }
                ExpenseValue::RATE { value } => {
                    p -= value.clone();
                    m -= value.apply_to(total.value)
                }
            };
        }
        Rest::new(m, p)
    }

    fn add_source(&mut self, source: IncomeSource) -> Result<(), Error> {
        match self.sources.entry(source.name.clone()) {
            Entry::Vacant(vacant_entry) => vacant_entry.insert(source),
            Entry::Occupied(e) => Err(Error::AlreadyExists(EntityType::Income, e.key().clone()))?,
        };
        Ok(())
    }

    fn remove_source(&mut self, name: &str) -> Result<(), Error> {
        self.sources
            .remove(name)
            .ok_or_else(|| Error::NotFound(EntityType::Income, name.to_string()))?;
        Ok(())
    }

    fn update_source(&mut self, old_name: &str, source: IncomeSource) -> Result<(), Error> {
        if old_name == source.name {
            // Имя не изменилось, просто обновляем
            *self
                .sources
                .get_mut(old_name)
                .ok_or_else(|| Error::NotFound(EntityType::Income, old_name.to_string()))? = source;
        } else {
            // Имя изменилось, удаляем старое и добавляем новое
            self.remove_source(old_name)?;
            self.add_source(source)?;
        }
        Ok(())
    }

    fn add_expense(&mut self, expense: Expense) -> Result<(), Error> {
        match self.expenses.entry(expense.name.clone()) {
            Entry::Vacant(vacant_entry) => vacant_entry.insert(expense),
            Entry::Occupied(e) => Err(Error::AlreadyExists(EntityType::Expense, e.key().clone()))?,
        };
        Ok(())
    }

    fn remove_expense(&mut self, name: &str) -> Result<(), Error> {
        self.expenses
            .remove(name)
            .ok_or_else(|| Error::NotFound(EntityType::Expense, name.to_string()))?;
        Ok(())
    }

    fn update_expense(&mut self, old_name: &str, expense: Expense) -> Result<(), Error> {
        if old_name == expense.name {
            // Имя не изменилось, просто обновляем
            *self
                .expenses
                .get_mut(old_name)
                .ok_or_else(|| Error::NotFound(EntityType::Expense, old_name.to_string()))? =
                expense;
        } else {
            // Имя изменилось, удаляем старое и добавляем новое
            self.remove_expense(old_name)?;
            self.add_expense(expense)?;
        }
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.expenses.is_empty() || self.sources.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    AddIncomeSource {
        source: IncomeSource,
    },
    RemoveIncomeSource {
        name: String,
    },
    UpdateIncomeSource {
        old_name: String,
        source: IncomeSource,
    },
    AddExpense {
        expense: Expense,
    },
    RemoveExpense {
        name: String,
    },
    UpdateExpense {
        old_name: String,
        expense: Expense,
    },
}

#[derive(Debug, PartialEq)]
pub enum State {
    New,
    NotChanged,
    Changed { count: usize },
}

impl Default for State {
    fn default() -> Self {
        Self::New
    }
}

#[derive(Debug, PartialEq)]
pub struct Rest {
    money: Money,
    percentage: Percentage,
}

impl Rest {
    pub fn new(money: Money, percentage: Percentage) -> Self {
        Self { money, percentage }
    }
}

#[derive(Debug, Default)]
pub struct Editor {
    state: State,
    _source: Draft,
    current: Draft,
}

impl From<Draft> for Editor {
    fn from(value: Draft) -> Self {
        Self {
            state: State::NotChanged,
            _source: value.clone(),
            current: value,
        }
    }
}

impl Editor {
    pub fn state(&self) -> &State {
        &self.state
    }

    fn apply_event(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Event::AddIncomeSource { source } => self.current.add_source(source.clone()),
            Event::RemoveIncomeSource { name } => self.current.remove_source(name),
            Event::UpdateIncomeSource { old_name, source } => {
                self.current.update_source(old_name, source.clone())
            }
            Event::AddExpense { expense } => self.current.add_expense(expense.clone()),
            Event::RemoveExpense { name } => self.current.remove_expense(name),
            Event::UpdateExpense { old_name, expense } => {
                self.current.update_expense(old_name, expense.clone())
            }
        }
    }

    pub fn handle(&mut self, event: Event) -> Result<(), Error> {
        self.apply_event(&event)?;
        self.state = match self.state {
            State::Changed { count } => State::Changed { count: count + 1 },
            _ => State::Changed { count: 1 },
        };
        Ok(())
    }

    pub fn rest(&self) -> Rest {
        self.draft().rest()
    }

    pub fn draft(&self) -> &Draft {
        &self.current
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::core::{finance::Money, planning::ExpenseValue};

    use super::*;

    fn editor() -> Editor {
        Default::default()
    }

    fn salary_editor() -> Editor {
        let draft = Draft::build(
            vec![IncomeSource::new(
                "salary".to_string(),
                Money::new_rub(100000.into()),
            )],
            vec![],
        )
        .unwrap();
        draft.into()
    }

    #[rstest]
    fn empty_editor() {
        let editor = editor();
        assert_eq!(editor.state(), &State::New);
    }

    #[rstest]
    fn add_source() -> Result<(), Error> {
        let mut editor = editor();

        let income = Money::new_rub(10000.into());
        let salary = IncomeSource::new("salary".to_string(), income);
        editor.handle(Event::AddIncomeSource {
            source: salary.clone(),
        })?;

        assert_eq!(editor.rest(), Rest::new(income, Percentage::TOTAL));
        assert_eq!(
            Ok(editor.draft()),
            Draft::build(vec![salary.clone()], vec![]).as_ref()
        );
        assert_eq!(editor.state(), &State::Changed { count: 1 });

        let income = Money::new_rub(100000.into());
        let bottles = IncomeSource::new("bottles".to_string(), income);
        editor.handle(Event::AddIncomeSource {
            source: bottles.clone(),
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(*salary.expected() + *bottles.expected(), Percentage::TOTAL)
        );
        assert_eq!(
            Ok(editor.draft()),
            Draft::build(vec![salary, bottles], vec![]).as_ref()
        );
        assert_eq!(editor.state(), &State::Changed { count: 2 });

        Ok(())
    }

    #[rstest]
    fn add_expense() -> Result<(), Error> {
        let mut editor = salary_editor();
        editor.handle(Event::AddExpense {
            expense: Expense::new(
                "dream".to_string(),
                ExpenseValue::MONEY {
                    value: Money::new_rub(1000.into()),
                },
                None,
            ),
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(Money::new_rub(99000.into()), "99.0%".parse().unwrap())
        );
        Ok(())
    }

    #[rstest]
    fn from_draft() {
        let income = Money::new_rub(10000.into());
        let source = IncomeSource::new("salary".to_string(), income);
        let draft = Draft::build(vec![source], vec![]).unwrap();
        let editor: Editor = draft.clone().into();

        assert_eq!(editor._source, draft);
    }

    #[rstest]
    fn remove_expense() -> Result<(), Error> {
        let mut editor = salary_editor();
        let expense = Expense::new(
            "dream".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(1000.into()),
            },
            None,
        );
        editor.handle(Event::AddExpense {
            expense: expense.clone(),
        })?;

        editor.handle(Event::RemoveExpense {
            name: "dream".to_string(),
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(Money::new_rub(100000.into()), Percentage::TOTAL)
        );
        assert!(editor.draft().expenses.is_empty());
        assert_eq!(editor.state(), &State::Changed { count: 2 });
        Ok(())
    }

    #[rstest]
    fn remove_nonexistent_expense() {
        let mut editor = salary_editor();
        let result = editor.handle(Event::RemoveExpense {
            name: "nonexistent".to_string(),
        });
        assert_eq!(
            result,
            Err(Error::NotFound(
                EntityType::Expense,
                "nonexistent".to_string()
            ))
        );
        assert_eq!(editor.state(), &State::NotChanged);
    }

    #[rstest]
    fn update_expense_same_name() -> Result<(), Error> {
        let mut editor = salary_editor();
        let expense = Expense::new(
            "dream".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(1000.into()),
            },
            None,
        );
        editor.handle(Event::AddExpense {
            expense: expense.clone(),
        })?;

        let updated_expense = Expense::new(
            "dream".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(2000.into()),
            },
            None,
        );
        editor.handle(Event::UpdateExpense {
            old_name: "dream".to_string(),
            expense: updated_expense,
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(Money::new_rub(98000.into()), "98.0%".parse().unwrap())
        );
        assert_eq!(editor.draft().expenses.len(), 1);
        assert_eq!(editor.state(), &State::Changed { count: 2 });
        Ok(())
    }

    #[rstest]
    fn update_expense_different_name() -> Result<(), Error> {
        let mut editor = salary_editor();
        let expense = Expense::new(
            "dream".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(1000.into()),
            },
            None,
        );
        editor.handle(Event::AddExpense {
            expense: expense.clone(),
        })?;

        let updated_expense = Expense::new(
            "goal".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(2000.into()),
            },
            None,
        );
        editor.handle(Event::UpdateExpense {
            old_name: "dream".to_string(),
            expense: updated_expense,
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(Money::new_rub(98000.into()), "98.0%".parse().unwrap())
        );
        assert_eq!(editor.draft().expenses.len(), 1);
        assert!(editor.draft().expenses.contains_key("goal"));
        assert!(!editor.draft().expenses.contains_key("dream"));
        assert_eq!(editor.state(), &State::Changed { count: 2 });
        Ok(())
    }

    #[rstest]
    fn update_nonexistent_expense() {
        let mut editor = salary_editor();
        let updated_expense = Expense::new(
            "goal".to_string(),
            ExpenseValue::MONEY {
                value: Money::new_rub(2000.into()),
            },
            None,
        );
        let result = editor.handle(Event::UpdateExpense {
            old_name: "nonexistent".to_string(),
            expense: updated_expense,
        });
        assert_eq!(
            result,
            Err(Error::NotFound(
                EntityType::Expense,
                "nonexistent".to_string()
            ))
        );
        assert_eq!(editor.state(), &State::NotChanged);
    }

    #[rstest]
    fn remove_income_source() -> Result<(), Error> {
        let mut editor = salary_editor();
        editor.handle(Event::RemoveIncomeSource {
            name: "salary".to_string(),
        })?;

        assert!(editor.draft().sources.is_empty());
        assert_eq!(editor.state(), &State::Changed { count: 1 });
        Ok(())
    }

    #[rstest]
    fn remove_nonexistent_income_source() {
        let mut editor = salary_editor();
        let result = editor.handle(Event::RemoveIncomeSource {
            name: "nonexistent".to_string(),
        });
        assert_eq!(
            result,
            Err(Error::NotFound(
                EntityType::Income,
                "nonexistent".to_string()
            ))
        );
        assert_eq!(editor.state(), &State::NotChanged);
    }

    #[rstest]
    fn update_income_source_same_name() -> Result<(), Error> {
        let mut editor = salary_editor();
        let updated_source = IncomeSource::new("salary".to_string(), Money::new_rub(200000.into()));
        editor.handle(Event::UpdateIncomeSource {
            old_name: "salary".to_string(),
            source: updated_source,
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(Money::new_rub(200000.into()), Percentage::TOTAL)
        );
        assert_eq!(editor.draft().sources.len(), 1);
        assert_eq!(editor.state(), &State::Changed { count: 1 });
        Ok(())
    }

    #[rstest]
    fn update_income_source_different_name() -> Result<(), Error> {
        let mut editor = salary_editor();
        let updated_source = IncomeSource::new("wage".to_string(), Money::new_rub(200000.into()));
        editor.handle(Event::UpdateIncomeSource {
            old_name: "salary".to_string(),
            source: updated_source,
        })?;

        assert_eq!(
            editor.rest(),
            Rest::new(Money::new_rub(200000.into()), Percentage::TOTAL)
        );
        assert_eq!(editor.draft().sources.len(), 1);
        assert!(editor.draft().sources.contains_key("wage"));
        assert!(!editor.draft().sources.contains_key("salary"));
        assert_eq!(editor.state(), &State::Changed { count: 1 });
        Ok(())
    }

    #[rstest]
    fn update_nonexistent_income_source() {
        let mut editor = salary_editor();
        let updated_source = IncomeSource::new("wage".to_string(), Money::new_rub(200000.into()));
        let result = editor.handle(Event::UpdateIncomeSource {
            old_name: "nonexistent".to_string(),
            source: updated_source,
        });
        assert_eq!(
            result,
            Err(Error::NotFound(
                EntityType::Income,
                "nonexistent".to_string()
            ))
        );
        assert_eq!(editor.state(), &State::NotChanged);
    }
}
