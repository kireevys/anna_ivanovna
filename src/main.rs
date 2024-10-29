use chrono::Utc;
use rust_decimal_macros::dec;
use uuid::Uuid;

use anna_ivanovna;
use anna_ivanovna::distribute::{distribute, Income};
use anna_ivanovna::finance::{Currency, Money, Percentage};
use anna_ivanovna::planning::planning::{Expense, ExpenseValue, IncomeSource, Plan};

fn main() {
    let zp = IncomeSource::new(
        "Зарплата".to_string(),
        Money::new(dec!(100000), Currency::RUB),
    );

    let rent = Expense::new(
        "Аренда".to_string(),
        ExpenseValue::MONEY {
            value: Money::new_rub(dec!(25000)),
        },
    );
    let food = Expense::new(
        "Еда".to_string(),
        ExpenseValue::MONEY {
            value: Money::new(dec!(17345), Currency::RUB),
        },
    );
    let health = Expense::new(
        "Здоровье".to_string(),
        ExpenseValue::MONEY {
            value: Money::new(dec!(10000), Currency::RUB),
        },
    );
    let mortgage = Expense::new(
        "Ипотека".to_string(),
        ExpenseValue::MONEY {
            value: Money::new(dec!(10000), Currency::RUB),
        },
    );
    let home_service = Expense::new(
        "Коммуналка".to_string(),
        ExpenseValue::MONEY {
            value: Money::new(dec!(6000), Currency::RUB),
        },
    );
    let bag_month = Expense::new(
        "Подушка на месяц".to_string(),
        ExpenseValue::MONEY {
            value: Money::new(dec!(5000), Currency::RUB),
        },
    );

    let auto = Expense::new(
        "Авто".to_string(),
        ExpenseValue::RATE {
            value: Percentage::from(dec!(26.6599)),
        },
    );

    let plan = Plan::try_build(
        Uuid::new_v4(),
        vec![zp.clone()],
        vec![rent, food, health, mortgage, home_service, bag_month, auto],
    )
        .unwrap();

    let income = Income::new(
        zp.clone(),
        Money::new_rub(dec!(50000)),
        Utc::now().date_naive(),
    );
    print!("{}", distribute(plan, income).unwrap());
}
