use std::rc::Rc;

use yew::html::Scope;

use crate::{
    api::ApiClient,
    engine::{
        app::msg,
        core::Shell,
        plan::{
            self,
            msg::{LoadingMsg, Msg, PersistMsg, TemplateMsg},
        },
    },
    runtime::App,
};

pub struct PlanShell {
    pub api: Rc<ApiClient>,
    pub link: Scope<App>,
}

impl Shell<plan::model::PlanModel> for PlanShell {
    fn execute(&self, cmd: plan::cmd::Cmd) {
        let api = self.api.clone();
        let link = self.link.clone();
        match cmd {
            plan::cmd::Cmd::LoadPlan => {
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api.get_plan().await;
                    link.send_message(msg::Msg::Plan(Msg::Loading(
                        LoadingMsg::Loaded(result),
                    )));
                });
            }
            plan::cmd::Cmd::LoadTemplates => {
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api.get_collections().await.map_err(|e| e.to_string());
                    link.send_message(msg::Msg::Plan(Msg::Template(
                        TemplateMsg::TemplatesLoaded(result),
                    )));
                });
            }
            plan::cmd::Cmd::SavePlan { id, plan } => {
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api.update_plan(&id, &plan).await;
                    link.send_message(msg::Msg::Plan(Msg::Persist(
                        PersistMsg::SaveFinished(result),
                    )));
                });
            }
            plan::cmd::Cmd::CreatePlan { plan } => {
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api.create_plan(&plan).await;
                    link.send_message(msg::Msg::Plan(Msg::Persist(
                        PersistMsg::CreateFinished(result),
                    )));
                });
            }
            plan::cmd::Cmd::ScrollToTop => {
                if let Some(window) = web_sys::window() {
                    window.scroll_to_with_x_and_y(0.0, 0.0);
                }
            }
        }
    }
}
