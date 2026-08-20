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
#[case(dec!(13), dec!(100000))]
#[case(dec!(6), dec!(54321.99))]
#[case(dec!(0.5), dec!(1))]
fn gross_from_net_round_trips(#[case] rate: Decimal, #[case] net: Decimal) {
    let rate = Percentage::from(rate);
    let gross = rate
        .gross_from_net(net)
        .expect("ставка внутри допустимого диапазона");

    let restored_net = gross - rate.apply_to(gross);
    assert!(
        (restored_net - net).abs() < dec!(0.0000001),
        "удержание {rate} из {gross} дало {restored_net}, ожидалось {net}"
    );
}

#[rstest]
#[case(dec!(100))]
#[case(dec!(100.01))]
#[case(dec!(150))]
#[case(dec!(-0.01))]
#[case(dec!(-13))]
fn gross_from_net_rejects_rate_outside_range(#[case] rate: Decimal) {
    assert_eq!(Percentage::from(rate).gross_from_net(dec!(80)), None);
}

#[rstest]
#[case(dec!(0), true)]
#[case(dec!(13), true)]
#[case(dec!(99.99), true)]
#[case(dec!(100), false)]
#[case(dec!(-0.01), false)]
fn withholding_rate_range(#[case] rate: Decimal, #[case] expected: bool) {
    assert_eq!(
        Percentage::from(rate).is_valid_withholding_rate(),
        expected,
        "ставка {rate}%"
    );
}
