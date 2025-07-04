use crate::core::distribute::Budget;
use crate::core::planning::Plan;
use ratatui::text::Span;
use ratatui::widgets::ListItem;

#[derive(Debug, Clone)]
pub enum PlanNode {
    Title(String),
    Category(String),
    Expense {
        name: String,
        amount: String,
        percent: String,
    },
    Total {
        amount: String,
        percent: String,
    },
    Other(String),
}

#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    pub value: T,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }
    pub fn with_children(value: T, children: Vec<TreeNode<T>>) -> Self {
        Self { value, children }
    }
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }
}

/// Считает максимальную ширину имени и суммы для выравнивания
fn max_widths(tree: &TreeNode<PlanNode>) -> (usize, usize) {
    let mut name_w = 0;
    let mut amount_w = 0;
    let mut stack = vec![tree];
    while let Some(node) = stack.pop() {
        match &node.value {
            PlanNode::Expense { name, amount, .. } => {
                name_w = name_w.max(name.chars().count());
                amount_w = amount_w.max(amount.chars().count());
            }
            PlanNode::Total { amount, .. } => {
                amount_w = amount_w.max(amount.chars().count());
            }
            _ => {}
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    (name_w, amount_w)
}

/// Рендерит дерево с ascii-ветками и выравниванием чисел только для расходов и итогов
fn render_plan_tree<'a>(
    tree: &'a TreeNode<PlanNode>,
    prefix: &str,
    is_last: bool,
    parent_has_sibling: &[bool],
    name_w: usize,
    amount_w: usize,
) -> Vec<ListItem<'a>> {
    let mut items = Vec::new();
    // ascii-префикс
    let mut line_prefix = String::new();
    for &has_sibling in parent_has_sibling {
        if has_sibling {
            line_prefix.push_str("│  ");
        } else {
            line_prefix.push_str("   ");
        }
    }
    if !prefix.is_empty() {
        if is_last {
            line_prefix.push_str("└─ ");
        } else {
            line_prefix.push_str("├─ ");
        }
    }
    // Форматируем строку
    let formatted = match &tree.value {
        PlanNode::Expense {
            name,
            amount,
            percent,
        } => {
            let dots_w = name_w + 5 - name.chars().count();
            let dots_w = if dots_w > 0 { dots_w } else { 0 };
            if percent.is_empty() {
                format!(
                    "{}{}{} {:>amount_w$}",
                    line_prefix,
                    name,
                    "·".repeat(dots_w),
                    amount,
                    amount_w = amount_w
                )
            } else {
                format!(
                    "{}{}{} {:>amount_w$} [{}]",
                    line_prefix,
                    name,
                    "·".repeat(dots_w),
                    amount,
                    percent,
                    amount_w = amount_w
                )
            }
        }
        PlanNode::Total { amount, percent } => {
            if percent.is_empty() {
                format!("{line_prefix}💰 {amount}")
            } else {
                format!("{line_prefix}💰 {amount} [{percent}]")
            }
        }
        PlanNode::Category(cat) | PlanNode::Other(cat) | PlanNode::Title(cat) => {
            format!("{line_prefix}{cat}")
        }
    };
    items.push(ListItem::new(Span::raw(formatted)));
    let len = tree.children.len();
    for (i, child) in tree.children.iter().enumerate() {
        let last = i + 1 == len;
        let mut new_parent_has_sibling = parent_has_sibling.to_vec();
        new_parent_has_sibling.push(!last);
        items.extend(render_plan_tree(
            child,
            "",
            last,
            &new_parent_has_sibling,
            name_w,
            amount_w,
        ));
    }
    items
}

/// Упрощённый рендер: сам считает ширины и вызывает render_plan_tree
fn render_plan_tree_auto<'a>(tree: &'a TreeNode<PlanNode>) -> Vec<ListItem<'a>> {
    let (name_w, amount_w) = max_widths(tree);
    render_plan_tree(tree, "", true, &[], name_w, amount_w)
}

impl<'a> From<&'a TreeNode<PlanNode>> for Vec<ListItem<'a>> {
    fn from(value: &'a TreeNode<PlanNode>) -> Self {
        render_plan_tree_auto(value)
    }
}

impl From<&Plan> for TreeNode<PlanNode> {
    fn from(plan: &Plan) -> Self {
        use crate::core::finance::{Money, Percentage};
        let mut root = TreeNode::new(PlanNode::Title("План бюджета".to_string()));
        // Источники дохода
        let mut sources_node = TreeNode::new(PlanNode::Other("💸 Источники дохода:".to_string()));
        for source in &plan.sources {
            sources_node.add_child(TreeNode::new(PlanNode::Other(format!("{source}"))));
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
        let mut expenses_root =
            TreeNode::new(PlanNode::Other("Запланированные расходы:".to_string()));
        for (category, expenses) in plan.categories() {
            let cat_emoji = if category == "Без категории" {
                "📦"
            } else {
                "📂"
            };
            let mut cat_node = TreeNode::new(PlanNode::Category(format!("{cat_emoji} {category}")));
            let mut cat_total_amount = Money::new_rub(rust_decimal::Decimal::ZERO);
            let mut cat_total_percent = Percentage::ZERO;
            // Сначала добавляем итоговую ноду (мешочек)
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
            // Затем расходы
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
}

impl From<&Budget> for TreeNode<PlanNode> {
    fn from(budget: &Budget) -> Self {
        use crate::core::finance::Money;
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
}
