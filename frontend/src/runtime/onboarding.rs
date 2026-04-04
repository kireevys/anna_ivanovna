use yew::Context;

use crate::runtime::{App, AppMsg, AppPhase, OnboardingMsg};

pub fn handle(app: &mut App, ctx: &Context<App>, msg: OnboardingMsg) -> bool {
    match msg {
        OnboardingMsg::PhaseResolved(phase) => {
            app.phase = phase;
            if app.phase == AppPhase::Ready {
                app.load_plan_async(ctx.link());
            }
            true
        }
        OnboardingMsg::PickFolder => {
            pick_folder(ctx);
            false
        }
        OnboardingMsg::FolderPicked(path) => {
            if let AppPhase::Onboarding { chosen_path, .. } = &mut app.phase {
                *chosen_path = path;
            }
            true
        }
        OnboardingMsg::CompleteSetup => {
            complete_setup(app, ctx);
            false
        }
        OnboardingMsg::SetupFinished(result) => {
            match result {
                Ok(()) => {
                    app.phase = AppPhase::Ready;
                    app.load_plan_async(ctx.link());
                }
                Err(e) => {
                    if let AppPhase::Onboarding { error, saving, .. } = &mut app.phase {
                        *error = Some(e);
                        *saving = false;
                    }
                }
            }
            true
        }
    }
}

#[cfg(feature = "tauri")]
pub fn resolve_initial_phase(ctx: &Context<App>) -> AppPhase {
    let link = ctx.link().clone();
    wasm_bindgen_futures::spawn_local(async move {
        let phase = match crate::tauri_ipc::is_configured().await {
            Ok(true) => match async {
                crate::tauri_ipc::start_app_backend().await?;
                wait_for_backend().await
            }
            .await
            {
                Ok(()) => AppPhase::Ready,
                Err(e) => AppPhase::Onboarding {
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
                AppPhase::Onboarding {
                    default_path,
                    chosen_path: None,
                    error: None,
                    saving: false,
                }
            }
            Err(e) => AppPhase::Onboarding {
                default_path: String::new(),
                chosen_path: None,
                error: Some(format!("Ошибка проверки конфигурации: {e}")),
                saving: false,
            },
        };
        link.send_message(AppMsg::Onboarding(OnboardingMsg::PhaseResolved(phase)));
    });
    AppPhase::Checking
}

#[cfg(not(feature = "tauri"))]
pub fn resolve_initial_phase(ctx: &Context<App>) -> AppPhase {
    ctx.link().send_message(AppMsg::Plan(
        crate::engine::plan::msg::LoadingMsg::Reload.into(),
    ));
    AppPhase::Ready
}

#[cfg(feature = "tauri")]
fn pick_folder(ctx: &Context<App>) {
    let link = ctx.link().clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(path) = crate::tauri_ipc::pick_data_folder().await {
            link.send_message(AppMsg::Onboarding(OnboardingMsg::FolderPicked(path)));
        }
    });
}

#[cfg(not(feature = "tauri"))]
fn pick_folder(_ctx: &Context<App>) {}

#[cfg(feature = "tauri")]
fn complete_setup(app: &mut App, ctx: &Context<App>) {
    let buh_home = match &app.phase {
        AppPhase::Onboarding {
            chosen_path,
            default_path,
            ..
        } => chosen_path.as_deref().unwrap_or(default_path).to_string(),
        _ => return,
    };

    if let AppPhase::Onboarding { saving, error, .. } = &mut app.phase {
        *saving = true;
        *error = None;
    }

    let link = ctx.link().clone();
    wasm_bindgen_futures::spawn_local(async move {
        let result = async {
            crate::tauri_ipc::complete_setup(&buh_home).await?;
            wait_for_backend().await
        }
        .await;
        link.send_message(AppMsg::Onboarding(OnboardingMsg::SetupFinished(result)));
    });
}

#[cfg(not(feature = "tauri"))]
fn complete_setup(_app: &mut App, _ctx: &Context<App>) {}

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
