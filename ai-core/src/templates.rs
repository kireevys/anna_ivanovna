use rust_decimal_macros::dec;
use serde::Serialize;

use crate::{
    finance::{Money, Percentage},
    plan::Plan,
    planning::{Expense, ExpenseValue, IncomeKind, IncomeSource},
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Tag {
    Recommended,
    Stability,
    Debt,
    Future,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum CollectionContent {
    Book {
        book_url: &'static str,
        audio_url: &'static str,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Collection {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub content: CollectionContent,
    pub templates: Vec<PlanTemplate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub subtitle: &'static str,
    pub situation: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
    pub tag: Tag,
    pub plan: Plan,
}

pub fn collections() -> Vec<Collection> {
    let default_income = IncomeSource::new(
        "Зарплата".to_string(),
        IncomeKind::Salary {
            gross: Money::new_rub(dec!(1_000)),
            tax_rate: Percentage::from_int(13),
        },
    );

    let bansir = PlanTemplate {
        id: "bansir",
        name: "Бансир",
        subtitle: "колесничий · Глава 1",
        situation: "Зарабатываю, но к концу месяца ничего не остаётся",
        tagline: "10/90",
        description: "Бансир — колесничий, мастер своего дела. Хорошо зарабатывал, но тратил всё. Пришёл к Аркаду с вопросом: почему я работаю всю жизнь, а кошелёк пуст? Ответ прост — начни откладывать хотя бы десятую часть.",
        tag: Tag::Recommended,
        plan: Plan::build(
            std::slice::from_ref(&default_income),
            &[
                Expense::envelope(
                    "Заплати себе первому".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(10),
                    },
                    Some("Капитал".to_string()),
                ),
                Expense::envelope(
                    "На жизнь".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(90),
                    },
                    None,
                ),
            ],
        ),
    };

    let nomasir = PlanTemplate {
        id: "nomasir",
        name: "Номасир",
        subtitle: "сын Аркада · Глава 5",
        situation: "Стабильный доход, хочу управлять деньгами осознанно",
        tagline: "20/50/30",
        description: "Сын Аркада. Отец дал ему мешок золота и табличку с пятью законами. Номасир сначала потерял всё на глупых вложениях, но потом научился — структура и дисциплина важнее азарта.",
        tag: Tag::Stability,
        plan: Plan::build(
            std::slice::from_ref(&default_income),
            &[
                Expense::envelope(
                    "Заплати себе первому".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(20),
                    },
                    Some("Капитал".to_string()),
                ),
                Expense::envelope(
                    "Необходимое".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(50),
                    },
                    Some("На жизнь".to_string()),
                ),
                Expense::envelope(
                    "Для удовольствия".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(30),
                    },
                    None,
                ),
            ],
        ),
    };

    let dabasir = PlanTemplate {
        id: "dabasir",
        name: "Дабасир",
        subtitle: "торговец верблюдами · Глава 8",
        situation: "Есть кредиты или долги, хочу выбраться",
        tagline: "10/70/20",
        description: "Бывший раб, который влез в долги. Решил: даже с долгами — сначала заплати себе. 10% откладывай, 20% отдавай кредиторам, на 70% живи. Кредиторы согласились — лучше получать часть, чем ничего.",
        tag: Tag::Debt,
        plan: Plan::build(
            std::slice::from_ref(&default_income),
            &[
                Expense::envelope(
                    "Заплати себе первому".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(10),
                    },
                    Some("Капитал".to_string()),
                ),
                Expense::envelope(
                    "Необходимое".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(70),
                    },
                    Some("На жизнь".to_string()),
                ),
                Expense::envelope(
                    "Погашение долгов".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(20),
                    },
                    Some("Погашение долгов".to_string()),
                ),
            ],
        ),
    };

    let arkad = PlanTemplate {
        id: "arkad",
        name: "Аркад",
        subtitle: "мудрец · Глава 3",
        situation: "Расходы под контролем, хочу приумножать и строить будущее",
        tagline: "10/50/20/20",
        description: "Самый богатый человек в Вавилоне. Начинал бедным писцом. Семь правил, которые он вывел за жизнь: плати себе первому, контролируй расходы, приумножай, защищай от потерь.",
        tag: Tag::Future,
        plan: Plan::build(
            std::slice::from_ref(&default_income),
            &[
                Expense::envelope(
                    "Заплати себе первому".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(10),
                    },
                    Some("Капитал".to_string()),
                ),
                Expense::envelope(
                    "Необходимое".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(50),
                    },
                    Some("На жизнь".to_string()),
                ),
                Expense::envelope(
                    "Большая цель".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(20),
                    },
                    Some("Будущее".to_string()),
                ),
                Expense::envelope(
                    "Приумножай".to_string(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(20),
                    },
                    Some("Инвестиции".to_string()),
                ),
            ],
        ),
    };

    vec![Collection {
        id: "richest-man-in-babylon",
        name: "Самый богатый человек в Вавилоне",
        description: "Принципы из книги Джорджа Клейсона (1926), которые работают до сих пор.\nВыберите персонажа, чья ситуация ближе всего к вашей.",
        content: CollectionContent::Book {
            book_url: "https://www.litres.ru/book/dzhorzh-semuel-kleyson/samyy-bogatyy-chelovek-v-vavilone-68620378/chitat-onlayn",
            audio_url: "https://youtu.be/y2Ri81liSmk?si=F6dMM42EeZwZqtNV&t=25",
        },
        templates: vec![bansir, nomasir, dabasir, arkad],
    }]
}
