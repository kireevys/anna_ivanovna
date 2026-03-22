use crate::{
    api::{
        ApiClient,
        ApiError,
        BudgetEntry,
        Collection,
        Cursor,
        Page,
        StoragePlanFrontend,
    },
    components::{AppLayout, WelcomeScreen},
    config::API_V1_BASE_URL,
    presentation::{history::HistoryEntry, plan::Plan},
};
use rust_decimal::Decimal;
use std::{rc::Rc, str::FromStr};
use yew::{Component, Context, Html, html};

mod history;
mod onboarding;
mod plan;
mod view;

#[derive(Clone, PartialEq)]
#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
pub enum AppPhase {
    /// Checking if app is configured (Tauri only)
    Checking,
    /// First run — show welcome/onboarding screen
    Onboarding {
        default_path: String,
        chosen_path: Option<String>,
        error: Option<String>,
        saving: bool,
    },
    /// App is ready — backend running, show main UI
    Ready,
}

#[derive(Clone, PartialEq)]
pub enum View {
    Plan,
    History,
}

#[derive(Clone, PartialEq)]
enum DataState<T> {
    Loading,
    Loaded(T),
    Error(String),
}

#[derive(Clone, PartialEq)]
enum PaginatableDataState<T> {
    Loading,
    Loaded {
        items: Vec<T>,
        next_cursor: Option<Cursor>,
    },
    LoadingMore {
        items: Vec<T>,
        next_cursor: Option<Cursor>,
    },
    Error(String),
}

impl<T> PaginatableDataState<T> {
    fn is_paginating(&self) -> bool {
        matches!(self, Self::LoadingMore { .. })
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PlanMode {
    View,
    Edit,
    Creating,
}

#[derive(Clone, PartialEq)]
enum PlanValidation {
    Valid,
    FormatInvalid { messages: Vec<String> },
    BusinessInvalid { messages: Vec<String> },
}

#[derive(Clone, Copy, PartialEq)]
enum SaveState {
    Idle,
    CanSave,
    Disabled,
    Saving,
}

struct PlanState {
    data: DataState<Plan>,
    meta: Option<StoragePlanFrontend>,
    mode: PlanMode,
    incomes: Vec<crate::presentation::editable_plan::EditableIncomeSource>,
    expenses: Vec<crate::presentation::editable_plan::EditableExpense>,
    validation: PlanValidation,
    save_state: SaveState,
    edited_core_plan: Option<ai_core::plan::Plan>,
    templates: DataState<Vec<Collection>>,
}

impl PlanState {
    fn set_data(&mut self, data: DataState<Plan>) {
        self.data = data;
    }

    fn set_meta(&mut self, meta: Option<StoragePlanFrontend>) {
        self.meta = meta;
    }

    fn set_mode(&mut self, mode: PlanMode) {
        self.mode = mode;
    }

    fn set_incomes(
        &mut self,
        incomes: Vec<crate::presentation::editable_plan::EditableIncomeSource>,
    ) {
        self.incomes = incomes;
    }

    fn set_expenses(
        &mut self,
        expenses: Vec<crate::presentation::editable_plan::EditableExpense>,
    ) {
        self.expenses = expenses;
    }

    fn reset_validation(&mut self) {
        self.validation = PlanValidation::Valid;
        self.save_state = SaveState::Idle;
    }

    /// Пересчитать валидацию после локального изменения плана (до запроса на бэк)
    /// `business_invalid` должен быть true, если по бизнес-инвариантам
    /// (например, расходы > доходы) план некорректен.
    fn recompute_validation_after_edit(&mut self, business_invalid: bool) {
        let mut format_messages = Vec::new();

        Self::validate_named_items(
            self.incomes
                .iter()
                .map(|i| (i.name.as_str(), i.amount.as_str(), i.is_valid)),
            "дохода",
            &mut format_messages,
        );

        Self::validate_named_items(
            self.expenses
                .iter()
                .map(|e| (e.name.as_str(), e.amount.as_str(), e.is_valid)),
            "расхода",
            &mut format_messages,
        );

        if !format_messages.is_empty() {
            self.validation = PlanValidation::FormatInvalid {
                messages: format_messages,
            };
            self.save_state = SaveState::Disabled;
        } else if business_invalid {
            // Формат ок, но нарушены бизнес-правила.
            // Строим более подробные сообщения по полям, если можем.
            let mut business_messages = Vec::new();

            for income in &self.incomes {
                if income.is_valid
                    && !income.amount.is_empty()
                    && let Ok(v) = Decimal::from_str(&income.amount)
                    && v <= Decimal::ZERO
                {
                    business_messages.push(format!(
                        "Доход \"{}\" должен быть больше 0",
                        income.name
                    ));
                }
            }

            for expense in &self.expenses {
                if expense.is_valid
                    && !expense.amount.is_empty()
                    && let Ok(v) = Decimal::from_str(&expense.amount)
                    && v <= Decimal::ZERO
                {
                    business_messages.push(format!(
                        "Расход \"{}\" должен быть больше 0",
                        expense.name
                    ));
                }
            }

            if business_messages.is_empty() {
                business_messages
                    .push("План некорректен: расходы превышают доходы".into());
            }

            self.validation = PlanValidation::BusinessInvalid {
                messages: business_messages,
            };
            self.save_state = SaveState::Disabled;
        } else {
            self.validation = PlanValidation::Valid;
            self.save_state = match self.save_state {
                SaveState::Idle | SaveState::Disabled => SaveState::CanSave,
                SaveState::Saving => SaveState::Saving,
                SaveState::CanSave => SaveState::CanSave,
            };
        }
    }

    fn validate_named_items<'a>(
        items: impl Iterator<Item = (&'a str, &'a str, bool)>,
        label: &str,
        messages: &mut Vec<String>,
    ) {
        let items: Vec<_> = items.collect();

        for &(name, amount, is_valid) in &items {
            if !is_valid && !amount.is_empty() {
                messages.push(format!(
                    "{}: некорректное число",
                    Self::item_display_name(name, label),
                ));
            }
        }

        if items.iter().any(|&(name, _, _)| name.trim().is_empty()) {
            messages.push(format!("Не указано название {label}"));
        }

        if items.iter().any(|&(_, amount, _)| amount.is_empty()) {
            messages.push(format!("Не указана сумма {label}"));
        }

        let mut seen = std::collections::HashSet::new();
        for &(name, _, _) in &items {
            let trimmed = name.trim().to_lowercase();
            if !trimmed.is_empty() && !seen.insert(trimmed) {
                messages.push(format!(
                    "Дублирующееся название {label}: \"{}\"",
                    name.trim()
                ));
            }
        }
    }

    fn item_display_name(name: &str, label: &str) -> String {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            format!("(без названия {label})")
        } else {
            format!("{label} \"{trimmed}\"")
        }
    }
}

struct HistoryState {
    data: PaginatableDataState<HistoryEntry>,
}

impl HistoryState {
    fn set_data(&mut self, data: PaginatableDataState<HistoryEntry>) {
        self.data = data;
    }

    fn prepare_load(&mut self) -> Option<Cursor> {
        match &self.data {
            PaginatableDataState::Loaded { items, next_cursor } => {
                let items = items.clone();
                let next_cursor = next_cursor.clone();
                self.set_data(PaginatableDataState::LoadingMore {
                    items,
                    next_cursor: next_cursor.clone(),
                });
                next_cursor
            }
            PaginatableDataState::LoadingMore { next_cursor, .. } => {
                next_cursor.clone()
            }
            _ => {
                self.set_data(PaginatableDataState::Loading);
                None
            }
        }
    }

    fn merge_page(&mut self, page: Page<BudgetEntry>) {
        let new_entries: Vec<HistoryEntry> =
            page.items.iter().map(HistoryEntry::from).collect();

        let was_loading_more =
            matches!(self.data, PaginatableDataState::LoadingMore { .. });

        match &mut self.data {
            PaginatableDataState::Loading => {
                self.set_data(PaginatableDataState::Loaded {
                    items: new_entries,
                    next_cursor: page.next_cursor,
                });
            }
            PaginatableDataState::Loaded { items, next_cursor }
            | PaginatableDataState::LoadingMore { items, next_cursor } => {
                items.extend(new_entries);
                *next_cursor = page.next_cursor.clone();

                // Переводим из LoadingMore в Loaded
                if was_loading_more {
                    let items = items.clone();
                    let next_cursor = next_cursor.clone();
                    self.set_data(PaginatableDataState::Loaded { items, next_cursor });
                }
            }
            _ => {}
        }
    }
}

pub struct App {
    phase: AppPhase,
    view: View,
    plan: PlanState,
    history: HistoryState,
    api: Rc<ApiClient>,
}

#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
pub enum OnboardingMsg {
    PhaseResolved(AppPhase),
    PickFolder,
    FolderPicked(Option<String>),
    CompleteSetup,
    SetupFinished(Result<(), String>),
}

pub enum AppMsg {
    Onboarding(OnboardingMsg),
    SwitchView(View),
    LoadPlan,
    PlanLoaded(Result<StoragePlanFrontend, ApiError>),
    EnterEditMode,
    CancelEditMode,
    IncomeSourcesChanged(Vec<crate::presentation::editable_plan::EditableIncomeSource>),
    ExpensesChanged(Vec<crate::presentation::editable_plan::EditableExpense>),
    SavePlan,
    PlanSaveFinished(Result<(), ApiError>),
    TemplatesLoaded(Result<Vec<Collection>, String>),
    SelectTemplate(ai_core::plan::Plan),
    BackToTemplates,
    CreateFromScratch,
    CreatePlan,
    PlanCreateFinished(Result<String, ApiError>),
    LoadHistory,
    HistoryLoaded(Result<Page<BudgetEntry>, String>),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let initial_phase = onboarding::resolve_initial_phase(ctx);

        Self {
            phase: initial_phase,
            view: View::Plan,
            plan: PlanState {
                data: DataState::Loading,
                meta: None,
                mode: PlanMode::View,
                incomes: vec![],
                expenses: vec![],
                validation: PlanValidation::Valid,
                save_state: SaveState::Idle,
                edited_core_plan: None,
                templates: DataState::Loading,
            },
            history: HistoryState {
                data: PaginatableDataState::Loading,
            },
            api: Rc::new(ApiClient::new(API_V1_BASE_URL.clone())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Onboarding(msg) => onboarding::handle(self, ctx, msg),
            AppMsg::SwitchView(view) => self.handle_switch_view(ctx, view),
            AppMsg::LoadPlan => self.handle_load_plan(ctx),
            AppMsg::PlanLoaded(result) => self.handle_plan_loaded(ctx, result),
            AppMsg::EnterEditMode => self.handle_enter_edit_mode(),
            AppMsg::CancelEditMode => self.handle_cancel_edit_mode(),
            AppMsg::IncomeSourcesChanged(incomes) => {
                self.handle_income_sources_changed(incomes)
            }
            AppMsg::ExpensesChanged(expenses) => self.handle_expenses_changed(expenses),
            AppMsg::SavePlan => self.handle_save_plan(ctx),
            AppMsg::PlanSaveFinished(result) => {
                self.handle_plan_save_finished(ctx, result)
            }
            AppMsg::TemplatesLoaded(result) => self.handle_templates_loaded(result),
            AppMsg::SelectTemplate(plan) => self.handle_select_template(plan),
            AppMsg::BackToTemplates => self.handle_back_to_templates(),
            AppMsg::CreateFromScratch => self.handle_create_from_scratch(),
            AppMsg::CreatePlan => self.handle_create_plan(ctx),
            AppMsg::PlanCreateFinished(result) => {
                self.handle_plan_create_finished(ctx, result)
            }
            AppMsg::LoadHistory => self.handle_load_history(ctx),
            AppMsg::HistoryLoaded(result) => self.handle_history_loaded(result),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.phase {
            AppPhase::Checking => html! {
                <crate::components::Loading />
            },
            AppPhase::Onboarding {
                default_path,
                chosen_path,
                error,
                saving,
            } => html! {
                <WelcomeScreen
                    default_path={default_path.clone()}
                    chosen_path={chosen_path.clone()}
                    error={error.clone()}
                    saving={*saving}
                    on_pick_folder={ctx.link().callback(|_| AppMsg::Onboarding(OnboardingMsg::PickFolder))}
                    on_complete={ctx.link().callback(|_| AppMsg::Onboarding(OnboardingMsg::CompleteSetup))}
                />
            },
            AppPhase::Ready => html! {
                <AppLayout
                    current_view={self.view.clone()}
                    on_switch_view={ctx.link().callback(AppMsg::SwitchView)}
                    sticky_header={self.render_sticky_header()}
                >
                    {self.render_content(ctx)}
                </AppLayout>
            },
        }
    }
}

impl App {}
