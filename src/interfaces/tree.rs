use ratatui::text::Span;
use ratatui::widgets::ListItem;

#[derive(Debug, Clone)]
pub(crate) enum PlanNode {
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
pub(crate) struct TreeNode<T> {
    pub value: T,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }
    #[allow(dead_code)]
    pub fn with_children(value: T, children: Vec<TreeNode<T>>) -> Self {
        Self { value, children }
    }
    pub(crate) fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }
}

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

fn render_plan_tree<'a>(
    tree: &'a TreeNode<PlanNode>,
    prefix: &str,
    is_last: bool,
    parent_has_sibling: &[bool],
    name_w: usize,
    amount_w: usize,
) -> Vec<ListItem<'a>> {
    let mut items = Vec::new();
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
    let formatted = match &tree.value {
        PlanNode::Expense { name, amount, percent } => {
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

fn render_plan_tree_auto<'a>(tree: &'a TreeNode<PlanNode>) -> Vec<ListItem<'a>> {
    let (name_w, amount_w) = max_widths(tree);
    render_plan_tree(tree, "", true, &[], name_w, amount_w)
}

impl<'a> From<&'a TreeNode<PlanNode>> for Vec<ListItem<'a>> {
    fn from(value: &'a TreeNode<PlanNode>) -> Self {
        render_plan_tree_auto(value)
    }
}

fn render_plan_tree_text_impl(
    tree: &TreeNode<PlanNode>,
    parent_has_sibling: &[bool],
    is_last: bool,
    name_w: usize,
    amount_w: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line_prefix = String::new();
    for &has_sibling in parent_has_sibling {
        if has_sibling {
            line_prefix.push_str("│  ");
        } else {
            line_prefix.push_str("   ");
        }
    }
    if !parent_has_sibling.is_empty() {
        if is_last {
            line_prefix.push_str("└─ ");
        } else {
            line_prefix.push_str("├─ ");
        }
    }
    let formatted = match &tree.value {
        PlanNode::Expense { name, amount, percent } => {
            let dots_w = name_w + 5 - name.chars().count();
            let dots_w = if dots_w > 0 { dots_w } else { 0 };
            if percent.is_empty() {
                format!("{}{}{} {:>amount_w$}", line_prefix, name, "·".repeat(dots_w), amount, amount_w = amount_w)
            } else {
                format!("{}{}{} {:>amount_w$} [{}]", line_prefix, name, "·".repeat(dots_w), amount, percent, amount_w = amount_w)
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
    lines.push(formatted);
    let len = tree.children.len();
    for (i, child) in tree.children.iter().enumerate() {
        let last = i + 1 == len;
        let mut new_parent_has_sibling = parent_has_sibling.to_vec();
        new_parent_has_sibling.push(!last);
        lines.extend(render_plan_tree_text_impl(child, &new_parent_has_sibling, last, name_w, amount_w));
    }
    lines
}

pub(crate) fn to_text(tree: &TreeNode<PlanNode>) -> String {
    let (name_w, amount_w) = max_widths(tree);
    let lines = render_plan_tree_text_impl(tree, &[], true, name_w, amount_w);
    lines.join("\n")
}
