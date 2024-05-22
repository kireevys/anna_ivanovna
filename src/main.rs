use chrono::Utc;
use rust_decimal_macros::dec;

use anna_ivanovna;
use anna_ivanovna::distributor::{Currency, Expense, Income, IncomeSource, Money, Plan};

fn main() {
    let zp = IncomeSource::new(
        "Зарплата".to_string(),
        Money::new(dec!(100000), Currency::RUB),
    );
    let rent =
        Expense::new("Аренда".to_string(), Money::new(dec!(25000), Currency::RUB));
    let food = Expense::new("Еда".to_string(), Money::new(dec!(17345), Currency::RUB));
    let health = Expense::new(
        "Здоровье".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let mortgage = Expense::new(
        "Ипотека".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let home_service = Expense::new(
        "Коммуналка".to_string(),
        Money::new(dec!(6000), Currency::RUB),
    );
    let bag_month = Expense::new(
        "Подушка на месяц".to_string(),
        Money::new(dec!(5000), Currency::RUB),
    );
    let cloth =
        Expense::new("Шмотки".to_string(), Money::new(dec!(2000), Currency::RUB));
    let house_maintenance = Expense::new(
        "Бытовые удобства".to_string(),
        Money::new(dec!(1000), Currency::RUB),
    );

    let plan = Plan::try_build(
        vec![&zp],
        vec![
            &rent,
            &food,
            &health,
            &mortgage,
            &home_service,
            &bag_month,
            &cloth,
            &house_maintenance,
        ],
        ,
    )
        .unwrap();

    let income = Income::new(zp.clone(), Utc::now(), Money::new(dec!(50000), Currency::RUB));
    print!("{}", plan.new_income(&income).unwrap());
}
