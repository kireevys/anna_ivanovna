use ai_core::planning::IncomeKind;

pub const SALARY_LABEL: &str = "Зарплата";
pub const OTHER_LABEL: &str = "Другое";

pub fn kind_label(kind: &IncomeKind) -> &'static str {
    match kind {
        IncomeKind::Salary { .. } => SALARY_LABEL,
        IncomeKind::Other { .. } => OTHER_LABEL,
    }
}
