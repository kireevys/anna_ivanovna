use chrono::Utc;
use rust_decimal_macros::dec;

use anna_ivanovna;
use anna_ivanovna::distributor::{
    Absolute, Currency, Income, IncomeSource, Money, Percentage, Plan, Target,
};

fn main() {
    let zp = IncomeSource::new(
        "Зарплата".to_string(),
        Money::new(dec!(100000), Currency::RUB),
    );
    let rent = Absolute::new("Аренда".to_string(), Money::new(dec!(25000), Currency::RUB));
    let food = Absolute::new("Еда".to_string(), Money::new(dec!(17345), Currency::RUB));
    let health = Absolute::new(
        "Здоровье".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let mortgage = Absolute::new(
        "Ипотека".to_string(),
        Money::new(dec!(10000), Currency::RUB),
    );
    let home_service = Absolute::new(
        "Коммуналка".to_string(),
        Money::new(dec!(6000), Currency::RUB),
    );
    let bag_month = Absolute::new(
        "Подушка на месяц".to_string(),
        Money::new(dec!(5000), Currency::RUB),
    );

    let auto = Target::new(
        "Авто".to_string(),
        Money::new(dec!(1_000_000), Currency::RUB),
        Percentage::from_int(10),
    );

    let plan = Plan::try_build(
        vec![zp.clone()],
        vec![
            rent,
            food,
            health,
            mortgage,
            home_service,
            bag_month,
        ],
        vec![auto],
    )
        .unwrap();

    let income = Income::new(
        zp.clone(),
        Utc::now(),
        Money::new(dec!(50000), Currency::RUB),
    );
    print!("{}", plan.new_income(&income).unwrap());
}
