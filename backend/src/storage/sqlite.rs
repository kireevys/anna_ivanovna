use ai_app::storage::{
    BudgetId,
    CoreRepo,
    Cursor,
    Page,
    PlanId,
    StorageBudget,
    StorageError,
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
    #[instrument(skip(self, plan))]
    fn save_plan(&self, plan_id: PlanId, plan: Plan) -> Result<PlanId, StorageError> {
        self.block_on(async {
            let content =
                serde_json::to_string(&plan).map_err(|_| StorageError::SavePlan)?;

            sqlx::query("INSERT OR REPLACE INTO plans (id, content) VALUES (?, ?)")
                .bind(&plan_id)
                .bind(&content)
                .execute(&self.pool)
                .await
                .map_err(|_| StorageError::SavePlan)?;

            info!("План сохранён в SQLite: {plan_id}");
            Ok(plan_id)
        })
    }

    #[instrument(skip(self))]
    fn get_plan(&self) -> Option<Plan> {
        self.block_on(async {
            let row = sqlx::query("SELECT content FROM plans ORDER BY id DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten()?;

            let content: String = row.get("content");
            serde_json::from_str(&content)
                .map_err(|e| error!("Ошибка десериализации плана: {e}"))
                .ok()
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
            let content = serde_json::to_string(&budget)
                .map_err(|_| StorageError::SaveBudget)?;

            sqlx::query(
                "INSERT OR REPLACE INTO budgets (id, source, income_date, content) VALUES (?, ?, ?, ?)",
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
                    sqlx::query("SELECT id, content FROM budgets WHERE id < ? ORDER BY id DESC LIMIT ?")
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
