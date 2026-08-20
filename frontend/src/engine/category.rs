use serde::{Deserialize, Serialize};

pub const NO_CATEGORY: &str = "Без категории";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum CategoryKey {
    NoCategory,
    Named(String),
}

impl CategoryKey {
    pub fn named(name: &str) -> Self {
        if name == NO_CATEGORY {
            CategoryKey::NoCategory
        } else {
            CategoryKey::Named(name.to_string())
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            CategoryKey::NoCategory => NO_CATEGORY,
            CategoryKey::Named(name) => name,
        }
    }
}
