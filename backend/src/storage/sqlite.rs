use ai_app::storage::{
    BudgetId,
    CoreRepo,
    Cursor,
    Page,
    PlanAction,
    PlanEvent,
    PlanId,
    PlanStatus,
    StorageBudget,
    StorageError,
    StoragePlan,
    UserId,
};
use ai_core::{distribute::Budget, plan::Plan};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;
use tokio::runtime::Handle;
use tracing::{error, info, instrument};

#[derive(Debug, Clone)]
pub struct SqliteRepo {
    pool: SqlitePool,
    db_path: String,
}

impl SqliteRepo {
    pub async fn init(db_path: &Path) -> Result<Self, String> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| format!("Ошибка подключения к SQLite: {e}"))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| format!("Ошибка миграции SQLite: {e}"))?;

        info!("SQLite инициализирован: {}", db_path.display());
        Ok(Self {
            pool,
            db_path: db_path.to_string_lossy().into_owned(),
        })
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        tokio::task::block_in_place(|| Handle::current().block_on(future))
    }
}

impl CoreRepo for SqliteRepo {
    #[instrument(skip(self))]
    fn get_plan(&self, user_id: &UserId) -> Option<StoragePlan> {
        self.block_on(async {
            let row = sqlx::query(
                "SELECT id, content, version, status FROM plans \
                 WHERE user_id = ? AND status = 'active' LIMIT 1",
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()?;

            let id: String = row.get("id");
            let content: String = row.get("content");
            let version: i64 = row.get("version");
            let status_raw: String = row.get("status");

            let plan: Plan = serde_json::from_str(&content)
                .map_err(|e| error!("Ошибка десериализации плана: {e}"))
                .ok()?;

            let status: PlanStatus = match status_raw.parse() {
                Ok(s) => s,
                Err(_) => {
                    error!("Неизвестный статус плана в БД: {status_raw}");
                    return None;
                }
            };

            Some(StoragePlan {
                user_id: user_id.clone(),
                id,
                plan,
                version,
                status,
            })
        })
    }

    #[instrument(skip(self, plan))]
    fn create_plan(
        &self,
        user_id: &UserId,
        plan_id: PlanId,
        plan: Plan,
    ) -> Result<PlanId, StorageError> {
        self.block_on(async {
            let content =
                serde_json::to_string(&plan).map_err(|_| StorageError::CreatePlan)?;

            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|_| StorageError::CreatePlan)?;

            sqlx::query(
                "INSERT INTO plans (id, user_id, content, version, status) \
                 VALUES (?, ?, ?, 1, 'active')",
            )
            .bind(&plan_id)
            .bind(user_id)
            .bind(&content)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(db_err) = &e {
                    // SQLITE_CONSTRAINT_PRIMARYKEY или аналогичная ошибка
                    if db_err.code().as_deref() == Some("1555")
                        || db_err.message().contains("UNIQUE")
                    {
                        return StorageError::PlanAlreadyExists;
                    }
                }
                StorageError::CreatePlan
            })?;

            sqlx::query(
                "INSERT INTO plan_events (plan_id, version, action, content) \
                 VALUES (?, 1, 'created', ?)",
            )
            .bind(&plan_id)
            .bind(&content)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::CreatePlan)?;

            tx.commit().await.map_err(|_| StorageError::CreatePlan)?;

            info!("План создан в SQLite: {plan_id}");
            Ok(plan_id)
        })
    }

    #[instrument(skip(self, plan))]
    fn update_plan(
        &self,
        user_id: &UserId,
        plan_id: &PlanId,
        plan: Plan,
    ) -> Result<(), StorageError> {
        self.block_on(async {
            let content =
                serde_json::to_string(&plan).map_err(|_| StorageError::UpdatePlan)?;

            let mut tx = self.pool.begin().await.map_err(|_| StorageError::UpdatePlan)?;

            let result = sqlx::query(
                "UPDATE plans SET version = version + 1, content = ?, updated_at = datetime('now') \
                 WHERE user_id = ? AND id = ? AND status = 'active'",
            )
            .bind(&content)
            .bind(user_id)
            .bind(plan_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::UpdatePlan)?;

            if result.rows_affected() == 0 {
                return Err(StorageError::UpdatePlan);
            }

            sqlx::query(
                "INSERT INTO plan_events (plan_id, version, action, content) \
                 SELECT id, version, 'updated', ? FROM plans \
                 WHERE user_id = ? AND id = ?",
            )
            .bind(&content)
            .bind(user_id)
            .bind(plan_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::UpdatePlan)?;

            tx.commit().await.map_err(|_| StorageError::UpdatePlan)?;

            info!("План обновлён: {plan_id}");
            Ok(())
        })
    }

    #[instrument(skip(self))]
    fn delete_plan(
        &self,
        user_id: &UserId,
        plan_id: &PlanId,
    ) -> Result<(), StorageError> {
        self.block_on(async {
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|_| StorageError::DeletePlan)?;

            let result = sqlx::query(
                "UPDATE plans SET status = 'deleted', version = version + 1, \
                 deleted_at = datetime('now'), updated_at = datetime('now') \
                 WHERE user_id = ? AND id = ? AND status = 'active'",
            )
            .bind(user_id)
            .bind(plan_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::DeletePlan)?;

            if result.rows_affected() == 0 {
                return Err(StorageError::DeletePlan);
            }

            sqlx::query(
                "INSERT INTO plan_events (plan_id, version, action, content) \
                 SELECT id, version, 'deleted', content FROM plans \
                 WHERE user_id = ? AND id = ?",
            )
            .bind(user_id)
            .bind(plan_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::DeletePlan)?;

            tx.commit().await.map_err(|_| StorageError::DeletePlan)?;

            info!("План удалён (soft delete): {plan_id}");
            Ok(())
        })
    }

    #[instrument(skip(self))]
    fn plan_events(
        &self,
        _user_id: &UserId,
        plan_id: &PlanId,
        from: Option<Cursor>,
        limit: usize,
    ) -> Page<PlanEvent> {
        self.block_on(async {
            let rows = match &from {
                Some(cursor) => {
                    sqlx::query(
                        "SELECT id, plan_id, version, action, content, created_at \
                         FROM plan_events WHERE plan_id = ? AND id < ? ORDER BY id DESC LIMIT ?",
                    )
                    .bind(plan_id)
                    .bind(cursor)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await
                }
                None => {
                    sqlx::query(
                        "SELECT id, plan_id, version, action, content, created_at \
                         FROM plan_events WHERE plan_id = ? ORDER BY id DESC LIMIT ?",
                    )
                    .bind(plan_id)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await
                }
            };

            let rows = match rows {
                Ok(r) => r,
                Err(e) => {
                    error!("Ошибка запроса событий плана: {e}");
                    return Page::new(vec![], None);
                }
            };

            let items: Vec<PlanEvent> = rows
                .iter()
                .filter_map(|row| {
                    let id: i64 = row.get("id");
                    let plan_id: String = row.get("plan_id");
                    let version: i64 = row.get("version");
                    let action_raw: String = row.get("action");
                    let content_str: Option<String> = row.get("content");
                    let created_at: String = row.get("created_at");

                    let action: PlanAction = match action_raw.parse() {
                        Ok(a) => a,
                        Err(_) => {
                            error!("Неизвестное действие плана в БД: {action_raw}");
                            return None;
                        }
                    };

                    let content = content_str.and_then(|c| {
                        serde_json::from_str(&c)
                            .map_err(|e| {
                                error!("Ошибка десериализации плана в событии {id}: {e}")
                            })
                            .ok()
                    });

                    Some(PlanEvent {
                        id,
                        plan_id,
                        version,
                        action,
                        content,
                        created_at,
                    })
                })
                .collect();

            let next_cursor = items.last().map(|e| e.id.to_string());
            Page::new(items, next_cursor)
        })
    }

    #[instrument(skip(self, budget))]
    fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> Result<BudgetId, StorageError> {
        self.block_on(async {
            let source = &budget.income.source.name;
            let income_date = budget.income_date().format("%Y-%m-%d").to_string();
            let content =
                serde_json::to_string(&budget).map_err(|_| StorageError::SaveBudget)?;

            sqlx::query(
                "INSERT OR REPLACE INTO budgets (id, source, income_date, content) \
                 VALUES (?, ?, ?, ?)",
            )
            .bind(&budget_id)
            .bind(source)
            .bind(&income_date)
            .bind(&content)
            .execute(&self.pool)
            .await
            .map_err(|_| StorageError::SaveBudget)?;

            info!("Бюджет сохранён в SQLite: {budget_id}");
            Ok(budget_id)
        })
    }

    #[instrument(skip(self))]
    fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget> {
        self.block_on(async {
            let row = sqlx::query("SELECT content FROM budgets WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten()?;

            let content: String = row.get("content");
            let budget: Budget = serde_json::from_str(&content)
                .map_err(|e| error!("Ошибка десериализации бюджета {id}: {e}"))
                .ok()?;

            Some(StorageBudget {
                id: id.clone(),
                budget,
            })
        })
    }

    #[instrument(skip(self))]
    fn budgets(&self, from: Option<Cursor>, limit: usize) -> Page<StorageBudget> {
        self.block_on(async {
            let rows = match &from {
                Some(cursor) => {
                    sqlx::query(
                        "SELECT id, content FROM budgets WHERE id < ? ORDER BY id DESC LIMIT ?",
                    )
                    .bind(cursor)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await
                }
                None => {
                    sqlx::query("SELECT id, content FROM budgets ORDER BY id DESC LIMIT ?")
                        .bind(limit as i64)
                        .fetch_all(&self.pool)
                        .await
                }
            };

            let rows = match rows {
                Ok(r) => r,
                Err(e) => {
                    error!("Ошибка запроса бюджетов: {e}");
                    return Page::new(vec![], None);
                }
            };

            let items: Vec<StorageBudget> = rows
                .iter()
                .filter_map(|row| {
                    let id: String = row.get("id");
                    let content: String = row.get("content");
                    let budget: Budget = serde_json::from_str(&content)
                        .map_err(|e| error!("Ошибка десериализации бюджета {id}: {e}"))
                        .ok()?;
                    Some(StorageBudget { id, budget })
                })
                .collect();

            let next_cursor = items.last().map(|b| b.id.clone());
            Page::new(items, next_cursor)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_core::{
        finance::{Money, Percentage},
        planning::{Expense, ExpenseValue, IncomeSource},
    };
    use rust_decimal_macros::dec;
    use std::{path::PathBuf, time::SystemTime};

    fn temp_db_path() -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("anna_ivanovna_test_{ts}.db"))
    }

    fn valid_plan() -> Plan {
        Plan::build(
            &[IncomeSource::new(
                "Зарплата".into(),
                Money::new_rub(dec!(100000)),
            )],
            &[
                Expense::new(
                    "Аренда".into(),
                    ExpenseValue::MONEY {
                        value: Money::new_rub(dec!(30000)),
                    },
                    Some("Жильё".into()),
                ),
                Expense::new(
                    "Накопления".into(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(20),
                    },
                    None,
                ),
            ],
        )
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_plan_writes_deleted_event() {
        let db_path = temp_db_path();
        let repo = SqliteRepo::init(&db_path).await.unwrap();

        let user_id: UserId = "default".to_string();
        let plan_id: PlanId = "plan-1".to_string();

        repo.create_plan(&user_id, plan_id.clone(), valid_plan())
            .unwrap();
        repo.delete_plan(&user_id, &plan_id).unwrap();

        let (action, content_is_null, status) = repo
            .block_on(async {
                let row = sqlx::query(
                    "SELECT action, content FROM plan_events WHERE plan_id = ? ORDER BY id DESC LIMIT 1",
                )
                .bind(&plan_id)
                .fetch_one(&repo.pool)
                .await
                .unwrap();
                let action: String = row.get("action");
                let content: Option<String> = row.get("content");

                let row = sqlx::query("SELECT status FROM plans WHERE id = ?")
                    .bind(&plan_id)
                    .fetch_one(&repo.pool)
                    .await
                    .unwrap();
                let status: String = row.get("status");

                (action, content.is_none(), status)
            });

        assert_eq!(action, "deleted");
        assert!(!content_is_null);
        assert_eq!(status, "deleted");

        let _ = std::fs::remove_file(db_path);
    }
}
