CREATE TABLE IF NOT EXISTS plans (
id TEXT PRIMARY KEY,
content TEXT NOT NULL,
created_at TEXT NOT NULL DEFAULT (datetime ('now'))
) ;

CREATE TABLE IF NOT EXISTS budgets (
id TEXT PRIMARY KEY,
source TEXT NOT NULL,
income_date TEXT NOT NULL,
content TEXT NOT NULL,
created_at TEXT NOT NULL DEFAULT (datetime ('now'))
) ;
