use chrono::NaiveDate;
use rstest::rstest;
use rust_decimal_macros::dec;

use crate::finance::{Currency, Money, Percentage};

use crate::planning::{
    CreditExpense,
    CreditValidationError,
    Error,
    Expense,
    ExpenseValue,
    IncomeKind,
    IncomeSource,
};

fn make_source(name: &str, kind: IncomeKind) -> IncomeSource {
    IncomeSource::new(name.to_string(), kind)
}

#[test]
fn salary_net_applies_tax() {
    let source = make_source(
        "Зарплата",
        IncomeKind::Salary {
            gross: Money::new_rub(dec!(100000)),
            tax_rate: Percentage::from_int(13),
        },
    );
    assert_eq!(source.net(), Money::new_rub(dec!(87000)));
}

#[test]
fn other_net_returns_expected() {
    let source = make_source(
        "Фриланс",
        IncomeKind::Other {
            expected: Money::new_rub(dec!(50000)),
        },
    );
    assert_eq!(source.net(), Money::new_rub(dec!(50000)));
}

#[test]
fn serde_salary_roundtrip() {
    let source = make_source(
        "Зарплата",
        IncomeKind::Salary {
            gross: Money::new_rub(dec!(200000)),
            tax_rate: Percentage::from_int(15),
        },
    );
    let json = serde_json::to_string(&source).unwrap();
    let deserialized: IncomeSource = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, source.name);
    assert_eq!(deserialized.kind, source.kind);
}

#[test]
fn serde_other_roundtrip() {
    let source = make_source(
        "Фриланс",
        IncomeKind::Other {
            expected: Money::new_rub(dec!(50000)),
        },
    );
    let json = serde_json::to_string(&source).unwrap();
    let deserialized: IncomeSource = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, source.name);
    assert_eq!(deserialized.kind, source.kind);
}

#[test]
fn serde_backward_compat_old_format() {
    let old_json =
        r#"{"name":"Зарплата","expected":{"value":"100000","currency":"RUB"}}"#;
    let source: IncomeSource = serde_json::from_str(old_json).unwrap();
    assert_eq!(source.name, "Зарплата");
    assert_eq!(
        source.kind,
        IncomeKind::Other {
            expected: Money::new_rub(dec!(100000)),
        }
    );
}

#[test]
fn serde_salary_uses_tag() {
    let source = make_source(
        "Зарплата",
        IncomeKind::Salary {
            gross: Money::new(dec!(100000), Currency::RUB),
            tax_rate: Percentage::from_int(13),
        },
    );
    let json = serde_json::to_string(&source).unwrap();
    assert!(json.contains(r#""type":"salary""#));
}

fn default_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
}

#[test]
fn credit_expense_valid_creation() {
    let credit = CreditExpense::new(
        Money::new_rub(dec!(15000)),
        Money::new_rub(dec!(500000)),
        Percentage::from_int(12),
        36,
        default_date(),
    )
    .unwrap();
    insta::assert_json_snapshot!(credit);
}

#[rstest]
#[case::zero_term(
    Money::new_rub(dec!(15000)), Money::new_rub(dec!(500000)), 0,
    CreditValidationError::ZeroTermMonths
)]
#[case::zero_payment(
    Money::new_rub(dec!(0)), Money::new_rub(dec!(500000)), 36,
    CreditValidationError::NonPositivePayment
)]
#[case::zero_total(
    Money::new_rub(dec!(15000)), Money::new_rub(dec!(0)), 36,
    CreditValidationError::NonPositiveAmount
)]
fn credit_expense_rejects_invalid(
    #[case] payment: Money,
    #[case] total: Money,
    #[case] term: u32,
    #[case] expected: CreditValidationError,
) {
    let result = CreditExpense::new(
        payment,
        total,
        Percentage::from_int(12),
        term,
        default_date(),
    );
    assert_eq!(result, Err(Error::InvalidCredit(expected)));
}

#[test]
fn credit_expense_value_returns_monthly_payment() {
    let monthly = Money::new_rub(dec!(15000));
    let credit = CreditExpense::new(
        monthly,
        Money::new_rub(dec!(500000)),
        Percentage::from_int(12),
        36,
        default_date(),
    )
    .unwrap();
    assert_eq!(credit.value(), ExpenseValue::MONEY { value: monthly });
}

#[test]
fn expense_value_for_envelope_returns_correct_value() {
    let value = ExpenseValue::MONEY {
        value: Money::new_rub(dec!(25000)),
    };
    let expense = Expense::envelope("Аренда".to_string(), value.clone(), None);
    assert_eq!(expense.value(), value);
}

#[test]
fn expense_value_for_credit_returns_monthly_payment() {
    let monthly = Money::new_rub(dec!(15000));
    let credit = CreditExpense::new(
        monthly,
        Money::new_rub(dec!(500000)),
        Percentage::from_int(12),
        36,
        default_date(),
    )
    .unwrap();
    let expense = Expense::credit("Ипотека".to_string(), credit, None);
    assert_eq!(expense.value(), ExpenseValue::MONEY { value: monthly });
}

#[test]
fn serde_envelope_expense_roundtrip() {
    let expense = Expense::envelope(
        "Продукты".to_string(),
        ExpenseValue::MONEY {
            value: Money::new_rub(dec!(25000)),
        },
        Some("Быт".to_string()),
    );
    let json = serde_json::to_value(&expense).unwrap();
    insta::assert_json_snapshot!(json);
    let deserialized: Expense = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, expense);
}

#[test]
fn serde_credit_expense_roundtrip() {
    let credit = CreditExpense::new(
        Money::new_rub(dec!(15000)),
        Money::new_rub(dec!(500000)),
        Percentage::from_int(12),
        36,
        default_date(),
    )
    .unwrap();
    let expense =
        Expense::credit("Ипотека".to_string(), credit, Some("Кредиты".to_string()));
    let json = serde_json::to_value(&expense).unwrap();
    insta::assert_json_snapshot!(json);
    let deserialized: Expense = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, expense);
}

#[test]
fn serde_expense_backward_compat_no_kind() {
    let old_json =
        r#"{"name":"Продукты","value":{"RATE":{"value":"30"}},"category":"Быт"}"#;
    let expense: Expense = serde_json::from_str(old_json).unwrap();
    insta::assert_json_snapshot!(serde_json::to_value(&expense).unwrap());
    let roundtrip: Expense =
        serde_json::from_value(serde_json::to_value(&expense).unwrap()).unwrap();
    assert_eq!(roundtrip, expense);
}
