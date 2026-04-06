use serde::{Deserialize, Serialize};

use crate::engine::core::Model;

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum OnboardingModel {
    /// Checking if app is configured (Tauri only)
    Checking,
    /// First run — show welcome/onboarding screen
    Setup {
        default_path: String,
        chosen_path: Option<String>,
        error: Option<String>,
        saving: bool,
    },
    /// App is ready — backend running, show main UI
    Ready,
}

#[derive(Deserialize, Serialize)]
pub enum Msg {
    PhaseResolved(OnboardingModel),
    PickFolder,
    FolderPicked(Option<String>),
    CompleteSetup,
    SetupFinished(Result<(), String>),
}

#[derive(Serialize)]
pub enum Cmd {
    ResolvePhase,
    PickFolder,
    CompleteSetup { buh_home: String },
}

impl Model for OnboardingModel {
    type Msg = Msg;
    type Cmd = Cmd;

    fn handle(self, msg: Self::Msg) -> (Self, Vec<Self::Cmd>) {
        match msg {
            Msg::PhaseResolved(phase) => (phase, vec![]),
            Msg::PickFolder => (self, vec![Cmd::PickFolder]),
            Msg::FolderPicked(path) => {
                let mut new = self;
                if let OnboardingModel::Setup { chosen_path, .. } = &mut new {
                    *chosen_path = path;
                }
                (new, vec![])
            }
            Msg::CompleteSetup => {
                let buh_home = match &self {
                    OnboardingModel::Setup {
                        chosen_path,
                        default_path,
                        ..
                    } => chosen_path.as_deref().unwrap_or(default_path).to_string(),
                    _ => return (self, vec![]),
                };
                let mut new = self;
                if let OnboardingModel::Setup { saving, error, .. } = &mut new {
                    *saving = true;
                    *error = None;
                }
                (new, vec![Cmd::CompleteSetup { buh_home }])
            }
            Msg::SetupFinished(result) => match result {
                Ok(()) => (OnboardingModel::Ready, vec![]),
                Err(e) => {
                    let mut new = self;
                    if let OnboardingModel::Setup { error, saving, .. } = &mut new {
                        *error = Some(e);
                        *saving = false;
                    }
                    (new, vec![])
                }
            },
        }
    }
}
