use crate::core::distribute::Budget;
use crate::core::finance::Money;
use crate::core::finance::Percentage;
use crate::core::planning::DistributionWeights;
use crate::interfaces::tree::{PlanNode, TreeNode};
pub(crate) fn plan_to_tree(plan: &DistributionWeights) -> TreeNode<PlanNode> {
    let mut root = TreeNode::new(PlanNode::Title("План бюджета".to_string()));
    // Источники дохода
    let mut sources_node = TreeNode::new(PlanNode::Other("💸 Источники дохода:".to_string()));
    for source in &plan.sources {
        sources_node.add_child(TreeNode::new(PlanNode::Other(format!(
            "{} [{}]",
            source.name, source.expected
        ))));
    }
    root.add_child(sources_node);
    // Остаток
    let total_income = plan.sources.iter().map(|s| s.expected).sum::<Money>();
    let rest_amount = Money::new_rub(plan.rest.apply_to(total_income.value));
    root.add_child(TreeNode::new(PlanNode::Other(format!(
        "🏦 Остаток: {rest_amount} [{}]",
        plan.rest
    ))));
    // Категории и расходы
    let mut expenses_root = TreeNode::new(PlanNode::Other("Запланированные расходы:".to_string()));
    for (category, expenses) in plan.categories() {
        let cat_emoji = if category == "Без категории" {
            "📦"
        } else {
            "📂"
        };
        let mut cat_node = TreeNode::new(PlanNode::Category(format!("{cat_emoji} {category}")));
        let mut cat_total_amount = Money::new_rub(rust_decimal::Decimal::ZERO);
        let mut cat_total_percent = Percentage::ZERO;
        // Итог категории
        for expense in &expenses {
            if let Some(percentage) = plan.get(expense) {
                let estimated_amount = Money::new_rub(percentage.apply_to(total_income.value));
                cat_total_amount += estimated_amount;
                cat_total_percent += percentage.clone();
            }
        }
        cat_node.add_child(TreeNode::new(PlanNode::Total {
            amount: format!("{cat_total_amount}"),
            percent: format!("{cat_total_percent}"),
        }));
        // Элементы расходов
        for expense in &expenses {
            if let Some(percentage) = plan.get(expense) {
                let estimated_amount = Money::new_rub(percentage.apply_to(total_income.value));
                cat_node.add_child(TreeNode::new(PlanNode::Expense {
                    name: expense.name.clone(),
                    amount: format!("{estimated_amount}"),
                    percent: format!("{percentage}"),
                }));
            }
        }
        expenses_root.add_child(cat_node);
    }
    root.add_child(expenses_root);
    root
}

pub(crate) fn budget_to_tree(budget: &Budget) -> TreeNode<PlanNode> {
    let mut root = TreeNode::new(PlanNode::Title("Распределение дохода".to_string()));
    // Источник дохода
    root.add_child(TreeNode::new(PlanNode::Other(format!(
        "💸 Источник: {} ({} от {})",
        budget.income.source.name, budget.income.amount, budget.income.date
    ))));
    // Остаток
    root.add_child(TreeNode::new(PlanNode::Other(format!(
        "🏦 Остаток: {}",
        budget.rest
    ))));
    // Без категории
    if !budget.no_category.is_empty() {
        let mut no_cat_node = TreeNode::new(PlanNode::Category("📦 Без категории".to_string()));
        let mut total = Money::new_rub(rust_decimal::Decimal::ZERO);
        for entry in &budget.no_category {
            total += entry.amount;
        }
        no_cat_node.add_child(TreeNode::new(PlanNode::Total {
            amount: format!("{total}"),
            percent: String::new(),
        }));
        let mut sorted_entries = budget.no_category.clone();
        sorted_entries.sort_by_key(|e| e.expense.name.clone());
        for entry in &sorted_entries {
            no_cat_node.add_child(TreeNode::new(PlanNode::Expense {
                name: entry.expense.name.clone(),
                amount: format!("{}", entry.amount),
                percent: String::new(),
            }));
        }
        root.add_child(no_cat_node);
    }
    // Категории
    let mut sorted_categories: Vec<_> = budget.categories.iter().collect();
    sorted_categories.sort_by_key(|(cat, _)| *cat);
    for (category, entries) in sorted_categories {
        let mut cat_node = TreeNode::new(PlanNode::Category(format!("📂 {category}")));
        let mut total = Money::new_rub(rust_decimal::Decimal::ZERO);
        for entry in entries {
            total += entry.amount;
        }
        cat_node.add_child(TreeNode::new(PlanNode::Total {
            amount: format!("{total}"),
            percent: String::new(),
        }));
        let mut sorted_entries = entries.clone();
        sorted_entries.sort_by_key(|e| e.expense.name.clone());
        for entry in &sorted_entries {
            cat_node.add_child(TreeNode::new(PlanNode::Expense {
                name: entry.expense.name.clone(),
                amount: format!("{}", entry.amount),
                percent: String::new(),
            }));
        }
        root.add_child(cat_node);
    }
    root
}
