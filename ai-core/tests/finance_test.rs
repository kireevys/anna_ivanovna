use rstest::rstest;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use ai_core::finance::Percentage;

#[rstest]
#[case(dec!(0), dec!(100))]
#[case(dec!(13), dec!(87))]
#[case(dec!(20), dec!(80))]
#[case(dec!(99), dec!(1))]
fn gross_from_net_restores_gross(#[case] rate: Decimal, #[case] net: Decimal) {
    assert_eq!(
        Percentage::from(rate).gross_from_net(net),
        Some(dec!(100)),
        "ставка {rate}% от 100 должна давать {net} на руки"
    );
}

#[rstest]
#[case(dec!(100))]
#[case(dec!(100.01))]
#[case(dec!(150))]
fn gross_from_net_rejects_full_withholding(#[case] rate: Decimal) {
    assert_eq!(Percentage::from(rate).gross_from_net(dec!(80)), None);
}
