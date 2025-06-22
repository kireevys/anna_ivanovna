#[cfg(test)]
mod test_e2e {
    use crate::distribute::{Income, distribute};
    use crate::finance::Money;
    use crate::storage::{distribute_from_yaml, distribute_to_yaml, plan_from_yaml};
    use chrono::NaiveDate;
    use std::path::Path;

    #[test]
    fn test_e2e() {
        let plan = plan_from_yaml(Path::new("src/test_storage/plan.yaml")).unwrap();
        let source = plan.sources.first().unwrap();

        let income = Income::new(
            source.clone(),
            Money::new_rub((source.expected.value / rust_decimal::Decimal::from(2)).round_dp(2)),
            NaiveDate::from_ymd_opt(2025, 6, 21).unwrap(),
        );
        let result = distribute(&plan, &income).unwrap();

        // Проверяем что distribute_to_yaml работает
        let yaml_result = distribute_to_yaml(&result);
        assert!(!yaml_result.is_empty());
        println!("Generated YAML: {}", yaml_result);

        // Проверяем что distribute_from_yaml работает
        let expected = distribute_from_yaml(Path::new("src/test_storage/result.yaml")).unwrap();
        println!("Result: {:?}", result);
        println!("Expected: {:?}", expected);
        assert_eq!(result, expected);
    }
}
