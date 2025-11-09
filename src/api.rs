use std::ops::Deref;

use thiserror::Error;
use tracing::{debug, instrument};

use crate::core::{
    distribute::{Budget, Income, distribute as core_dist},
    editor::Plan,
    planning::DistributionWeights,
};
#[derive(Debug, Error)]
pub enum Error {
    #[error("distribution error")]
    CantDistribute,
    #[error("cant save budget")]
    CantSaveBudget,
}

pub type BudgetId = String;
pub type PlanId = String;

#[derive(Debug, Clone)]
pub struct StorageBudget {
    pub id: BudgetId,
    pub budget: Budget,
}

impl From<(BudgetId, Budget)> for StorageBudget {
    fn from(value: (BudgetId, Budget)) -> Self {
        Self {
            id: value.0,
            budget: value.1,
        }
    }
}

pub trait CoreRepo {
    fn location(&self) -> &str;
    fn get_plan(&self) -> Option<Plan>;
    fn save_plan(&self, plan: Plan, id: PlanId) -> Result<PlanId, Error>;
    fn save_budget(&self, budget: Budget) -> Result<BudgetId, Error>;
    fn budget_ids<'r>(
        &'r self,
        from: Option<Cursor>,
        limit: usize,
    ) -> Box<dyn Iterator<Item = BudgetId> + 'r>;
    fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget>;
}

#[instrument(skip(provider))]
pub fn get_plan<R: CoreRepo>(provider: &R) -> Option<Plan> {
    provider.get_plan()
}

#[instrument(skip(plan, income))]
pub fn distribute(plan: &DistributionWeights, income: &Income) -> Result<Budget, Error> {
    core_dist(plan, income).map_err(|_| Error::CantDistribute)
}

#[instrument(skip(plan, repo))]
pub fn save_plan<R: CoreRepo>(plan: Plan, id: PlanId, repo: &R) -> Result<PlanId, Error> {
    repo.save_plan(plan, id).map_err(|_| Error::CantSaveBudget)
}

#[instrument(skip(budget, repo))]
pub fn save_budget<R: CoreRepo>(budget: Budget, repo: &R) -> Result<BudgetId, Error> {
    repo.save_budget(budget).map_err(|_| Error::CantSaveBudget)
}

pub type Cursor = String;
#[derive(Debug)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Cursor>,
}

impl<T> Deref for Page<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<Cursor>) -> Self {
        Self { items, next_cursor }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[allow(dead_code)]
pub fn budget_list<R: CoreRepo>(
    repo: &R,
    from: Option<Cursor>,
    limit: usize,
) -> Page<StorageBudget> {
    let mut iter = repo.budget_ids(from, limit + 1);
    let items: Vec<StorageBudget> = iter
        .by_ref()
        .take(limit)
        .filter_map(|id| repo.budget_by_id(&id))
        .collect();
    let next_cursor = iter.next();
    Page::new(items, next_cursor)
}

#[instrument(skip(repo))]
pub fn budget_ids<R: CoreRepo>(repo: &R, from: Option<Cursor>, limit: usize) -> Page<BudgetId> {
    let mut iter = repo.budget_ids(from, limit + 1);
    let items: Vec<BudgetId> = iter.by_ref().take(limit).collect();
    let next_cursor = iter.next();
    let page = Page::new(items, next_cursor);
    debug!(?page);
    page
}

#[instrument(skip(repo))]
pub fn budget_by_id<R: CoreRepo>(repo: &R, id: &BudgetId) -> Option<StorageBudget> {
    repo.budget_by_id(id)
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::core::distribute::Budget;
    use crate::core::finance::Money;
    use crate::core::planning::IncomeSource;
    use std::collections::HashMap;

    struct InMemoryRepo {
        budgets: Vec<StorageBudget>,
        plan: Option<Plan>,
    }

    impl CoreRepo for InMemoryRepo {
        fn location(&self) -> &str {
            "MemoryRepo"
        }
        fn get_plan(&self) -> Option<Plan> {
            self.plan.clone()
        }
        fn save_plan(&self, _plan: Plan, id: PlanId) -> Result<PlanId, Error> {
            Ok(id)
        }
        fn save_budget(&self, _budget: Budget) -> Result<BudgetId, Error> {
            unimplemented!()
        }
        fn budget_ids<'a>(
            &'a self,
            from: Option<Cursor>,
            limit: usize,
        ) -> Box<dyn Iterator<Item = BudgetId> + 'a> {
            let mut items: Vec<_> = self.budgets.iter().map(|b| b.id.clone()).collect();
            items.sort();
            let start = from
                .as_ref()
                .and_then(|cursor| items.iter().position(|b| b == cursor))
                .map_or(0, |idx| idx + 1);
            Box::new(items.into_iter().skip(start).take(limit))
        }
        fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget> {
            self.budgets.iter().find(|b| &b.id == id).cloned()
        }
    }

    fn make_budget(id: &str) -> StorageBudget {
        StorageBudget {
            id: id.to_string(),
            budget: Budget {
                income: crate::core::distribute::Income {
                    source: IncomeSource::new("TestSource".to_string(), Money::new_rub(dec!(100))),
                    amount: Money::new_rub(dec!(100)),
                    date: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                },
                rest: Money::new_rub(dec!(42)),
                no_category: vec![],
                categories: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_empty_storage() {
        let repo = InMemoryRepo {
            budgets: vec![],
            plan: None,
        };
        let page = budget_list(&repo, None, 10);
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn test_one_budget() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("first_bdg")],
            plan: None,
        };
        let page = budget_list(&repo, None, 10);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, "first_bdg");
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn test_from_param() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b"), make_budget("c")],
            plan: None,
        };
        let page = budget_list(&repo, Some("a".to_string()), 10);
        let ids: Vec<_> = page.items.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["b", "c"]);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn test_limit_param() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b"), make_budget("c")],
            plan: None,
        };
        let page = budget_list(&repo, None, 2);
        let ids: Vec<_> = page.items.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
        assert_eq!(page.next_cursor, Some("c".to_string()));
    }

    #[test]
    fn test_from_and_limit() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b"), make_budget("c")],
            plan: None,
        };
        let page = budget_list(&repo, Some("a".to_string()), 1);
        let ids: Vec<_> = page.items.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["b"]);
        assert_eq!(page.next_cursor, Some("c".to_string()));
    }

    #[test]
    fn test_limit_zero() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b")],
            plan: None,
        };
        let page = budget_list(&repo, None, 0);
        assert!(page.items.is_empty());
        assert_eq!(page.next_cursor, Some("a".to_string()));
    }

    #[test]
    fn test_budget_ids_empty() {
        let repo = InMemoryRepo {
            budgets: vec![],
            plan: None,
        };
        let page = budget_ids(&repo, None, 10);
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn test_budget_ids_one() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("first_bdg")],
            plan: None,
        };
        let page = budget_ids(&repo, None, 10);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0], "first_bdg");
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn test_budget_ids_from_param() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b"), make_budget("c")],
            plan: None,
        };
        let page = budget_ids(&repo, Some("a".to_string()), 10);
        let ids: Vec<_> = page.items.iter().map(|b| b.as_str()).collect();
        assert_eq!(ids, ["b", "c"]);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn test_budget_ids_limit_param() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b"), make_budget("c")],
            plan: None,
        };
        let page = budget_ids(&repo, None, 2);
        let ids: Vec<_> = page.items.iter().map(|b| b.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
        assert_eq!(page.next_cursor, Some("c".to_string()));
    }

    #[test]
    fn test_budget_ids_from_and_limit() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b"), make_budget("c")],
            plan: None,
        };
        let page = budget_ids(&repo, Some("a".to_string()), 1);
        let ids: Vec<_> = page.items.iter().map(|b| b.as_str()).collect();
        assert_eq!(ids, ["b"]);
        assert_eq!(page.next_cursor, Some("c".to_string()));
    }

    #[test]
    fn test_budget_ids_limit_zero() {
        let repo = InMemoryRepo {
            budgets: vec![make_budget("a"), make_budget("b")],
            plan: None,
        };
        let page = budget_ids(&repo, None, 0);
        assert!(page.items.is_empty());
        assert_eq!(page.next_cursor, Some("a".to_string()));
    }
}
