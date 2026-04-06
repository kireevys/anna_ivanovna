use crate::engine::{
    app::{
        cmd::Cmd,
        model::{AppModel, View},
        msg::Msg,
    },
    core::{Model, PaginatedList},
    history,
    onboarding,
    plan,
};

pub(crate) fn handle(model: AppModel, msg: Msg) -> (AppModel, Vec<Cmd>) {
    match msg {
        Msg::Onboarding(msg) => handle_onboarding(model, msg),
        Msg::SwitchView(view) => handle_switch_view(model, view),
        Msg::Plan(plan_msg) => {
            let (new_plan, cmds) = model.plan.handle(plan_msg);
            let cmds = cmds.into_iter().map(Cmd::Plan).collect();
            (
                AppModel {
                    plan: new_plan,
                    ..model
                },
                cmds,
            )
        }
        Msg::History(history_msg) => {
            let (new_history, cmds) = model.history.handle(history_msg);
            let cmds = cmds.into_iter().map(Cmd::History).collect();
            (
                AppModel {
                    history: new_history,
                    ..model
                },
                cmds,
            )
        }
    }
}

fn handle_onboarding(model: AppModel, msg: onboarding::Msg) -> (AppModel, Vec<Cmd>) {
    let was_not_ready = model.onboarding != onboarding::OnboardingModel::Ready;
    let (new_onboarding, cmds) = model.onboarding.handle(msg);

    let mut app_cmds: Vec<Cmd> = cmds.into_iter().map(Cmd::Onboarding).collect();

    let plan = if was_not_ready && new_onboarding == onboarding::OnboardingModel::Ready
    {
        app_cmds.push(Cmd::Plan(plan::cmd::Cmd::LoadPlan));
        plan::model::PlanModel::Loading
    } else {
        model.plan
    };

    (
        AppModel {
            onboarding: new_onboarding,
            plan,
            view: model.view,
            history: model.history,
        },
        app_cmds,
    )
}

fn handle_switch_view(model: AppModel, view: View) -> (AppModel, Vec<Cmd>) {
    if view == View::History {
        (
            AppModel {
                view,
                history: history::HistoryModel {
                    data: PaginatedList::loading(),
                },
                ..model
            },
            vec![Cmd::History(history::Cmd::Fetch { cursor: None })],
        )
    } else {
        (AppModel { view, ..model }, vec![])
    }
}
