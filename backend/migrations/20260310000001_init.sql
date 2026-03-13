CREATE TABLE IF NOT EXISTS plans (
id TEXT PRIMARY KEY,
user_id TEXT NOT NULL DEFAULT 'default',
name TEXT,
status TEXT NOT NULL DEFAULT 'active',
version INTEGER NOT NULL DEFAULT 1,
content TEXT NOT NULL,
created_at TEXT NOT NULL DEFAULT (datetime ('now')),
updated_at TEXT NOT NULL DEFAULT (datetime ('now')),
deleted_at TEXT
) ;

CREATE TABLE IF NOT EXISTS plan_events (
id INTEGER PRIMARY KEY AUTOINCREMENT,
plan_id TEXT NOT NULL REFERENCES plans (id),
version INTEGER NOT NULL,
action TEXT NOT NULL,
content TEXT,
created_at TEXT NOT NULL DEFAULT (datetime ('now'))
) ;

CREATE TABLE IF NOT EXISTS budgets (
id TEXT PRIMARY KEY,
source TEXT NOT NULL,
income_date TEXT NOT NULL,
content TEXT NOT NULL,
created_at TEXT NOT NULL DEFAULT (datetime ('now'))
) ;
