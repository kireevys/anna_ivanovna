use chrono::NaiveDate;
use csv::{Reader, StringRecord};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::core::distribute::{Budget, BudgetEntry, Income};
use crate::core::finance::Money;
use crate::core::planning::{Expense, ExpenseValue, IncomeSource};

const STATICS_KEYS: [&str; 5] = [
    "Дата входа",
    "Источник",
    "Вход",
    "Период",
    "Итого распределено",
];

fn read<P: AsRef<Path>>(csv_path: &P) -> io::Result<(Vec<String>, Reader<BufReader<File>>)> {
    let file = File::open(csv_path)
        .map_err(|e| io::Error::other(format!("Не удалось открыть файл: {e}")))?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(BufReader::new(file));
    let headers = rdr
        .headers()
        .map_err(|e| io::Error::other(format!("Ошибка чтения заголовков: {e}")))?
        .clone();
    let headers: Vec<String> = headers.iter().map(|h| h.trim().to_string()).collect();
    Ok((headers, rdr))
}
pub fn parse_excel_csv<P: AsRef<Path>>(csv_path: P) -> io::Result<impl Iterator<Item = Budget>> {
    let (headers, mut rdr) = read(&csv_path)?;

    let res: Vec<Budget> = rdr
        .records()
        // .collect::<Result<_, _>>()
        .enumerate()
        .map(|(lineno, record)| {
            let record = record.map_err(io::Error::other)?;
            parse_row(&headers, &record).map_err(|_| {
                io::Error::other(format!("Ошибка парсинга строки {lineno} {record:?}"))
            })
        })
        .collect::<io::Result<_>>()?;

    Ok(res.into_iter())

    // Ok(records.iter().enumerate().map(|(lineno, record)| {
    //     parse_row(&headers, record)
    //         .map_err(|_| io::Error::other(format!("Ошибка парсинга строки {lineno} {record:?}")))?
    // }))
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%d.%m.%Y").ok()
}

fn parse_entry(key: &str, value: &str) -> io::Result<Option<BudgetEntry>> {
    if value.is_empty() {
        return Ok(None);
    }
    let money = parse_money(value).map_err(|_| {
        io::Error::other(format!(
            "Не удалось распарсить деньги для '{key}': '{value}'"
        ))
    })?;
    let expense_value = ExpenseValue::MONEY { value: money };
    let expense = Expense::new(key.to_string(), expense_value, None);
    Ok(Some(BudgetEntry::new(expense, money)))
}

fn parse_income(statics: &HashMap<String, String>) -> io::Result<Income> {
    let date_str = statics
        .get("Дата входа")
        .ok_or_else(|| io::Error::other("Нет поля 'Дата входа'"))?;
    let date =
        parse_date(date_str).ok_or_else(|| io::Error::other("Не удалось распарсить дату"))?;
    let source = statics
        .get("Источник")
        .map(|s| s.replace(" ", "-"))
        .unwrap_or("неизвестно".to_string());
    let amount = parse_money(
        statics
            .get("Вход")
            .ok_or_else(|| io::Error::other("Не найден Вход"))?,
    )
    .map_err(|_| io::Error::other("Не удалось распарсить деньги"))?;
    let income_source = IncomeSource::new(source.clone(), amount);
    Ok(Income::new(income_source, amount, date))
}

fn parse_row(headers: &[String], record: &StringRecord) -> io::Result<Budget> {
    let mut statics = HashMap::new();
    let mut expenses = HashMap::new();
    for (i, h) in headers.iter().enumerate() {
        let v = record.get(i).unwrap_or("").trim().to_string();
        if STATICS_KEYS.contains(&h.as_str()) {
            statics.insert(h.clone(), v);
        } else if !v.is_empty() {
            expenses.insert(h.clone(), v);
        }
    }
    let income = parse_income(&statics)?;
    let mut budget = Budget::new(income);
    for (k, v) in expenses.iter() {
        if let Some(b) = parse_entry(k, v)? {
            budget.push(None, b);
        }
    }
    Ok(budget)
}

fn parse_money(value: &str) -> io::Result<Money> {
    let value = value
        .replace(" ", "")
        .replace("р.", "")
        .replace("₽", "")
        .replace("\u{a0}", "")
        .replace(",", ".");
    let parsed = value
        .parse()
        .map_err(|_| io::Error::other(format!("Не спарсили деньги {value}")))?;
    Ok(Money::new_rub(parsed))
}
