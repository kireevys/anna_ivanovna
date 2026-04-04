use yew::Context;

use crate::{
    engine::plan::{
        cmd::PlanCmd,
        msg::{LoadingMsg, PersistMsg, PlanMsg, TemplateMsg},
    },
    runtime::{App, AppMsg},
};

fn scroll_to_top() {
    if let Some(window) = web_sys::window() {
        window.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

impl App {
    pub(crate) fn execute_plan_cmds(&self, cmds: Vec<PlanCmd>, ctx: &Context<Self>) {
        for cmd in cmds {
            match cmd {
                PlanCmd::LoadPlan => self.load_plan_async(ctx.link()),
                PlanCmd::LoadTemplates => self.load_templates_async(ctx.link()),
                PlanCmd::SavePlan { id, plan } => {
                    self.save_plan_async(id, plan, ctx.link())
                }
                PlanCmd::CreatePlan { plan } => {
                    self.create_plan_async(plan, ctx.link())
                }
                PlanCmd::ScrollToTop => scroll_to_top(),
            }
        }
    }

    pub(crate) fn load_plan_async(&self, link: &yew::html::Scope<Self>) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_plan().await;
            link.send_message(AppMsg::Plan(PlanMsg::Loading(LoadingMsg::Loaded(
                result,
            ))));
        });
    }

    fn save_plan_async(
        &self,
        id: String,
        core_plan: ai_core::plan::Plan,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.update_plan(&id, &core_plan).await;
            link.send_message(AppMsg::Plan(PlanMsg::Persist(
                PersistMsg::SaveFinished(result),
            )));
        });
    }

    fn load_templates_async(&self, link: &yew::html::Scope<Self>) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_collections().await.map_err(|e| e.to_string());
            link.send_message(AppMsg::Plan(PlanMsg::Template(
                TemplateMsg::TemplatesLoaded(result),
            )));
        });
    }

    fn create_plan_async(
        &self,
        core_plan: ai_core::plan::Plan,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.create_plan(&core_plan).await;
            link.send_message(AppMsg::Plan(PlanMsg::Persist(
                PersistMsg::CreateFinished(result),
            )));
        });
    }
}
