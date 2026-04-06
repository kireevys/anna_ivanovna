use yew::Context;

#[cfg(feature = "tauri")] use crate::engine::app::msg;
use crate::{
    engine::{
        app::cmd,
        core::Shell,
        onboarding::{self, OnboardingModel},
    },
    runtime::App,
};

pub struct OnboardingShell {
    pub link: yew::html::Scope<App>,
}

impl Shell<OnboardingModel> for OnboardingShell {
    fn execute(&self, cmd: onboarding::Cmd) {
        match cmd {
            onboarding::Cmd::ResolvePhase => {
                resolve_phase_async(&self.link);
            }
            onboarding::Cmd::PickFolder => {
                pick_folder(&self.link);
            }
            onboarding::Cmd::CompleteSetup { buh_home } => {
                complete_setup_async(&buh_home, &self.link);
            }
        }
    }
}

#[cfg(feature = "tauri")]
pub fn resolve_initial(ctx: &Context<App>) -> (OnboardingModel, Vec<cmd::Cmd>) {
    resolve_phase_async(ctx.link());
    (OnboardingModel::Checking, vec![])
}

#[cfg(not(feature = "tauri"))]
pub fn resolve_initial(_ctx: &Context<App>) -> (OnboardingModel, Vec<cmd::Cmd>) {
    (
        OnboardingModel::Ready,
        vec![cmd::Cmd::Plan(crate::engine::plan::cmd::Cmd::LoadPlan)],
    )
}

#[cfg(feature = "tauri")]
fn resolve_phase_async(link: &yew::html::Scope<App>) {
    let link = link.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let phase = match crate::tauri_ipc::is_configured().await {
            Ok(true) => match async {
                crate::tauri_ipc::start_app_backend().await?;
                wait_for_backend().await
            }
            .await
            {
                Ok(()) => OnboardingModel::Ready,
                Err(e) => OnboardingModel::Setup {
                    default_path: String::new(),
                    chosen_path: None,
                    error: Some(format!("Ошибка запуска: {e}")),
                    saving: false,
                },
            },
            Ok(false) => {
                let default_path = crate::tauri_ipc::get_default_buh_home()
                    .await
                    .unwrap_or_default();
                OnboardingModel::Setup {
                    default_path,
                    chosen_path: None,
                    error: None,
                    saving: false,
                }
            }
            Err(e) => OnboardingModel::Setup {
                default_path: String::new(),
                chosen_path: None,
                error: Some(format!("Ошибка проверки конфигурации: {e}")),
                saving: false,
            },
        };
        link.send_message(msg::Msg::Onboarding(onboarding::Msg::PhaseResolved(phase)));
    });
}

#[cfg(not(feature = "tauri"))]
fn resolve_phase_async(_link: &yew::html::Scope<App>) {}

#[cfg(feature = "tauri")]
fn pick_folder(link: &yew::html::Scope<App>) {
    let link = link.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(path) = crate::tauri_ipc::pick_data_folder().await {
            link.send_message(msg::Msg::Onboarding(onboarding::Msg::FolderPicked(
                path,
            )));
        }
    });
}

#[cfg(not(feature = "tauri"))]
fn pick_folder(_link: &yew::html::Scope<App>) {}

#[cfg(feature = "tauri")]
fn complete_setup_async(buh_home: &str, link: &yew::html::Scope<App>) {
    let link = link.clone();
    let buh_home = buh_home.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let result = async {
            crate::tauri_ipc::complete_setup(&buh_home).await?;
            wait_for_backend().await
        }
        .await;
        link.send_message(msg::Msg::Onboarding(onboarding::Msg::SetupFinished(result)));
    });
}

#[cfg(not(feature = "tauri"))]
fn complete_setup_async(_buh_home: &str, _link: &yew::html::Scope<App>) {}

#[cfg(feature = "tauri")]
async fn wait_for_backend() -> Result<(), String> {
    use crate::config::API_V1_BASE_URL;

    let health_url = API_V1_BASE_URL
        .join("plan")
        .map_err(|e| format!("invalid URL: {e}"))?
        .to_string();

    for _ in 0..30 {
        match gloo_net::http::Request::get(&health_url).send().await {
            Ok(_) => return Ok(()),
            Err(_) => {
                gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }

    Err("Backend не запустился".to_string())
}
