use std::{
    fs,
    path::{Path, PathBuf},
};

use rstest::rstest;
use serde::{Serialize, de::DeserializeOwned};

use frontend::engine::core::Model;

#[derive(Serialize)]
struct StepResult<M: Model>
where
    M: Serialize,
    M::Cmd: Serialize,
{
    step: String,
    model: M,
    cmds: Vec<M::Cmd>,
}

#[derive(serde::Deserialize)]
struct StoryConfig {
    state: StoryState,
}

#[derive(serde::Deserialize, PartialEq)]
enum StoryState {
    Enabled,
    Disabled,
}

const INITIAL_FILE: &str = "initial.json";
const RESULT_SNAPSHOT: &str = "result";

fn run_story<M>(config_path: &Path)
where
    M: Model + DeserializeOwned + Serialize,
    M::Msg: DeserializeOwned,
    M::Cmd: Serialize,
{
    let story_dir = config_path
        .parent()
        .expect("config.toml must be inside a story directory");

    let config_text =
        fs::read_to_string(config_path).expect("failed to read config.toml");
    let config: StoryConfig =
        toml::from_str(&config_text).expect("failed to parse config.toml");

    if matches!(config.state, StoryState::Disabled) {
        return;
    }

    let initial_path = story_dir.join(INITIAL_FILE);
    let initial_text =
        fs::read_to_string(&initial_path).expect("failed to read initial.json");
    let mut model: M =
        serde_json::from_str(&initial_text).expect("failed to parse initial.json");

    let mut msg_files: Vec<_> = fs::read_dir(story_dir)
        .expect("failed to read story directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".json")
                && name != INITIAL_FILE
                && name.as_bytes()[0].is_ascii_digit()
        })
        .collect();

    msg_files.sort_by_key(|entry| entry.file_name());

    let mut log: Vec<StepResult<M>> = vec![StepResult {
        step: "initial".to_string(),
        model: model.clone(),
        cmds: vec![],
    }];

    for entry in &msg_files {
        let step_name = entry
            .path()
            .file_stem()
            .expect("msg file must have a name")
            .to_string_lossy()
            .into_owned();
        let msg_text =
            fs::read_to_string(entry.path()).expect("failed to read message file");
        let msg: M::Msg =
            serde_json::from_str(&msg_text).expect("failed to parse message file");
        let (new_model, new_cmds) = model.handle(msg);
        model = new_model;
        log.push(StepResult {
            step: step_name,
            model: model.clone(),
            cmds: new_cmds,
        });
    }

    insta::with_settings!({
        omit_expression => true,
        prepend_module_to_snapshot => false,
        snapshot_path => story_dir,
    }, {
        insta::assert_yaml_snapshot!(RESULT_SNAPSHOT, &log);
    });
}

#[rstest]
fn plan(#[files("stories/plan/**/config.toml")] path: PathBuf) {
    run_story::<frontend::engine::plan::model::PlanModel>(&path);
}

#[rstest]
fn history(#[files("stories/history/**/config.toml")] path: PathBuf) {
    run_story::<frontend::engine::history::HistoryModel>(&path);
}

#[rstest]
fn onboarding(#[files("stories/onboarding/**/config.toml")] path: PathBuf) {
    run_story::<frontend::engine::onboarding::OnboardingModel>(&path);
}
