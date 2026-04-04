use yew::Context;

use crate::api::{BudgetEntry, Page};

use crate::runtime::{App, AppMsg, PaginatableDataState};

impl App {
    pub(crate) fn handle_switch_view(
        &mut self,
        ctx: &Context<Self>,
        view: crate::runtime::View,
    ) -> bool {
        self.view = view;
        if self.view == crate::runtime::View::History {
            self.history.set_data(PaginatableDataState::Loading);
            ctx.link().send_message(AppMsg::LoadHistory);
        }
        true
    }

    pub(crate) fn handle_load_history(&mut self, ctx: &Context<Self>) -> bool {
        let cursor = self.history.prepare_load();
        self.load_history_async(cursor, ctx.link());
        true
    }

    pub(crate) fn handle_history_loaded(
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

    pub(crate) fn load_history_async(
        &self,
        cursor: Option<crate::api::Cursor>,
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
