use std::rc::Rc;

use yew::html::Scope;

use crate::{
    api::ApiClient,
    engine::{app::msg, core::Shell, history},
    runtime::App,
};

pub struct HistoryShell {
    pub api: Rc<ApiClient>,
    pub link: Scope<App>,
}

impl Shell<history::HistoryModel> for HistoryShell {
    fn execute(&self, cmd: history::Cmd) {
        match cmd {
            history::Cmd::Fetch { cursor } => {
                let api = self.api.clone();
                let link = self.link.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result =
                        api.get_history(cursor).await.map_err(|e| e.to_string());
                    link.send_message(msg::Msg::History(history::Msg::Loaded(result)));
                });
            }
        }
    }
}
