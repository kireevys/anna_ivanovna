use crate::api::Page;
use yew::Context;

use super::{App, AppMsg, BudgetEntry, PaginatableDataState};

impl App {
    pub(super) fn handle_switch_view(
        &mut self,
        ctx: &Context<Self>,
        view: super::View,
    ) -> bool {
        self.view = view;
        if self.view == super::View::History {
            self.history.set_data(PaginatableDataState::Loading);
            ctx.link().send_message(AppMsg::LoadHistory);
        }
        true
    }

    pub(super) fn handle_load_history(&mut self, ctx: &Context<Self>) -> bool {
        let cursor = self.history.prepare_load();
        self.load_history_async(cursor, ctx.link());
        true
    }

    pub(super) fn handle_history_loaded(
        &mut self,
        result: Result<Page<BudgetEntry>, String>,
    ) -> bool {
        match result {
            Ok(page) => {
                self.history.merge_page(page);
            }
            Err(e) => {
                self.history.set_data(PaginatableDataState::Error(e));
            }
        }
        true
    }

    pub(super) fn load_history_async(
        &self,
        cursor: Option<super::Cursor>,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_history(cursor).await.map_err(|e| e.to_string());
            link.send_message(AppMsg::HistoryLoaded(result));
        });
    }
}
