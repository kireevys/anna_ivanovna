use serde::{Deserialize, Serialize};

use crate::api::Cursor;

pub trait Model: Clone {
    type Msg;
    type Cmd;

    fn handle(self, msg: Self::Msg) -> (Self, Vec<Self::Cmd>);
}

pub trait Shell<M: Model> {
    fn execute(&self, cmd: M::Cmd);
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum DataState<T> {
    Loading,
    Loaded(T),
    Error(String),
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum PageStatus {
    Idle,
    Loading,
    Error(String),
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct PaginatedList<T> {
    pub(crate) items: Vec<T>,
    pub(crate) next_cursor: Option<Cursor>,
    pub(crate) status: PageStatus,
}

impl<T> PaginatedList<T> {
    pub fn loading() -> Self {
        Self {
            items: vec![],
            next_cursor: None,
            status: PageStatus::Loading,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status, PageStatus::Loading)
    }
}
