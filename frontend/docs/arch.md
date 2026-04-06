# Архитектура фронтенда

Документ описывает генеральную архитектурную линию крейта `frontend` и правила,
которые нарушать нельзя. Все принципы равнозначны; при конфликте побеждает тот,
что ниже по номеру.

## Принцип 1. Направление зависимостей

```
presentation (yew / TUI / …) ──▶ engine ──▶ ai_core
```

Стрелки не разворачиваются. Engine не знает про yew, HTML, DOM, терминал.
Presentation не знает про внутренности engine, кроме публичного API модели.
`ai_core` — чистый домен, не знает ни про один presentation-слой и про engine.

## Принцип 2. Разделение обязанностей

| Слой | Владеет | Не владеет |
|---|---|---|
| `ai_core` | Финансовые примитивы (`Money`, `Percentage`), доменные модели (`Plan`, `Expense`, `IncomeSource`, `Budget`), чистые вычисления (`distribute`, бизнес-валидация) | UI state, TEA, IO |
| `engine` | TEA (`Model`/`Msg`/`Cmd`/`update`), модели UI-состояния (`PlanModel`, `HistoryModel`), **view-model-ы на raw типах**, editable-формы, валидация формата полей, rebuild core-плана из формы | Форматирование, разметка, framework-specific код |
| `presentation` | Рендер (yew-компоненты, HTML-шаблоны), форматирование (`FormattedMoney`, разделители тысяч, формат даты), строки UI | Бизнес-логика, группировка/сортировка данных, трансформации моделей |

## Принцип 3. View-model живёт в engine

View-model — это структура данных, удобная для отображения: группировка,
сортировка, агрегация, проекция. Если две разные presentation (yew и TUI) будут
показывать одни и те же данные — логика едина и принадлежит engine.

Поля view-model держатся на **raw-типах** из `ai_core` (`Money`, `NaiveDate`,
`CoreIncomeKind`). Форматирование — исключительно в presentation на этапе
рендера.

```text
engine::history::HistoryEntry  { income_amount: Money, date: NaiveDate, … }
                ▼ (format inline)
yew : FormattedMoney::from_money(entry.income_amount).to_string()
TUI : format!("{:>10.2}", entry.income_amount.value)
```

## Принцип 4. Editable-формы — это engine

`editable::{IncomeSource, Expense}` — модель **состояния формы**, а не
presentation. Живёт в engine потому что:

- engine владеет `EditState` и управляет её жизненным циклом через TEA;
- rebuild и валидация формата работают с ней;
- несколько presentation-ов могут рендерить одну и ту же форму.

Поля формы могут быть `String` (буфер ввода) — это не форматирование, это
промежуточное состояние ввода. Парсинг `String → Decimal` — тоже engine,
он нужен для rebuild core-плана.

## Принцип 5. Валидация — ответственность engine

Валидация формата (поле пустое, не число, дубликат имени) и валидация
бизнес-правил (расходы > доходов, неположительные суммы) — всё в engine.
Presentation только отображает ошибки, полученные от engine. Presentation
**никогда** не принимает решение «валидно ли».

## Принцип 6. Адаптеры на границе — в engine

Конвертация между api-типами (`BudgetEntry`, `StoragePlanFrontend`) и
engine-моделями — в engine. `From<&BudgetEntry> for HistoryEntry` — engine-код.
Адаптер всегда смотрит **внутрь своего слоя**: engine принимает api, никогда
не наоборот.

## Принцип 7. Запрещённые импорты

| В слое | Запрещено импортировать |
|---|---|
| `ai_core` | `crate::api`, `crate::engine`, `crate::presentation`, `yew`, `web_sys` |
| `engine` | `crate::presentation`, `yew`, `web_sys` |
| `presentation` | — (может всё, но не в обход engine) |

Нарушение — основание для блокирующего комментария на ревью.
