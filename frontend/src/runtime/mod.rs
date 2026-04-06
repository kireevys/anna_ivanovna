use std::rc::Rc;

use yew::{Component, Context, Html};

use crate::{
    api::ApiClient,
    config::API_V1_BASE_URL,
    engine::{
        app::{
            cmd,
            model::{AppModel, View},
            msg,
        },
        core::{Model, PaginatedList, Shell},
    },
};

mod history;
mod onboarding;
mod plan;
mod view;

pub struct App {
    model: Option<AppModel>,
    api: Rc<ApiClient>,
}

struct AppShell {
    api: Rc<ApiClient>,
    link: yew::html::Scope<App>,
}

impl Shell<AppModel> for AppShell {
    fn execute(&self, cmd: cmd::Cmd) {
        match cmd {
            cmd::Cmd::Plan(plan_cmd) => {
                let shell = plan::PlanShell {
                    api: self.api.clone(),
                    link: self.link.clone(),
                };
                shell.execute(plan_cmd);
            }
            cmd::Cmd::History(history_cmd) => {
                let shell = history::HistoryShell {
                    api: self.api.clone(),
                    link: self.link.clone(),
                };
                shell.execute(history_cmd);
            }
            cmd::Cmd::Onboarding(onboarding_cmd) => {
                let shell = onboarding::OnboardingShell {
                    link: self.link.clone(),
                };
                shell.execute(onboarding_cmd);
            }
        }
    }
}

impl Component for App {
    type Message = msg::Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (initial_onboarding, init_cmds) = onboarding::resolve_initial(ctx);

        let app = Self {
            model: Some(AppModel {
                onboarding: initial_onboarding,
                view: View::Plan,
                plan: crate::engine::plan::model::PlanModel::Loading,
                history: crate::engine::history::HistoryModel {
                    data: PaginatedList::loading(),
                },
            }),
            api: Rc::new(ApiClient::new(API_V1_BASE_URL.clone())),
        };

        let shell = AppShell {
            api: app.api.clone(),
            link: ctx.link().clone(),
        };
        for cmd in init_cmds {
            shell.execute(cmd);
        }

        app
    }

    // FIXME(#65): Option<AppModel> + take — костыль для owned handle(self).
    // Целевое решение: AppModel enum с Fatal вариантом.
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let Some(model) = self.model.take() else {
            return true;
        };
        let (new_model, cmds) = model.handle(msg);
        self.model = Some(new_model);

        let shell = AppShell {
            api: self.api.clone(),
            link: ctx.link().clone(),
        };
        for cmd in cmds {
            shell.execute(cmd);
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        view::render(self.model.as_ref(), &self.api, ctx)
    }
}
