use ai_core::templates::collections;

#[test]
fn each_template_has_valid_plan() {
    use ai_core::planning::DistributionWeights;

    for collection in &collections() {
        for template in &collection.templates {
            let result = DistributionWeights::try_from(template.plan.clone());
            assert!(
                result.is_ok(),
                "Template '{}' has invalid plan: {:?}",
                template.id,
                result.err()
            );
        }
    }
}
