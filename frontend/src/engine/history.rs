use serde::{Deserialize, Serialize};

use crate::{
    api::{BudgetEntry, Cursor, Page},
    engine::core::{Model, PageStatus, PaginatedList},
    presentation::history::HistoryEntry,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct HistoryModel {
    pub(crate) data: PaginatedList<HistoryEntry>,
}

#[derive(Deserialize, Serialize)]
pub enum Msg {
    Load,
    Loaded(Result<Page<BudgetEntry>, String>),
}

#[derive(Serialize)]
pub enum Cmd {
    Fetch { cursor: Option<Cursor> },
}

impl Model for HistoryModel {
    type Msg = Msg;
    type Cmd = Cmd;

    fn handle(self, msg: Self::Msg) -> (Self, Vec<Self::Cmd>) {
        match msg {
            Msg::Load => {
                let cursor = self.data.next_cursor.clone();
                let new = HistoryModel {
                    data: PaginatedList {
                        status: PageStatus::Loading,
                        ..self.data.clone()
                    },
                };
                (new, vec![Cmd::Fetch { cursor }])
            }
            Msg::Loaded(result) => match result {
                Ok(page) => {
                    let new_entries: Vec<HistoryEntry> =
                        page.items.iter().map(HistoryEntry::from).collect();
                    let mut items = self.data.items.clone();
                    items.extend(new_entries);
                    (
                        HistoryModel {
                            data: PaginatedList {
                                items,
                                next_cursor: page.next_cursor,
                                status: PageStatus::Idle,
                            },
                        },
                        vec![],
                    )
                }
                Err(e) => (
                    HistoryModel {
                        data: PaginatedList {
                            status: PageStatus::Error(e),
                            ..self.data.clone()
                        },
                    },
                    vec![],
                ),
            },
        }
    }
}
