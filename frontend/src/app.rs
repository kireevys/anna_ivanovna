use crate::{
    api::{ApiClient, ApiError, BudgetEntry, Cursor, Page, StoragePlanFrontend},
    components::AppLayout,
    config::API_V1_BASE_URL,
    presentation::{history::HistoryEntry, plan::Plan},
};
use rust_decimal::Decimal;
use std::{rc::Rc, str::FromStr};
use yew::{Component, Context, Html, html};

mod history;
mod plan;
mod view;

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

        // Ошибки формата (непарсящие значения)
        for income in &self.incomes {
            if !income.is_valid && !income.amount.is_empty() {
                format_messages
                    .push(format!("Доход \"{}\": некорректное число", income.name));
            }
        }

        for expense in &self.expenses {
            if !expense.is_valid && !expense.amount.is_empty() {
                format_messages
                    .push(format!("Расход \"{}\": некорректное число", expense.name));
            }
        }

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
    view: View,
    plan: PlanState,
    history: HistoryState,
    api: Rc<ApiClient>,
}

pub enum AppMsg {
    SwitchView(View),
    LoadPlan,
    PlanLoaded(Result<StoragePlanFrontend, String>),
    EnterEditMode,
    CancelEditMode,
    IncomeSourcesChanged(Vec<crate::presentation::editable_plan::EditableIncomeSource>),
    ExpensesChanged(Vec<crate::presentation::editable_plan::EditableExpense>),
    SavePlan,
    PlanSaveFinished(Result<(), ApiError>),
    LoadHistory,
    HistoryLoaded(Result<Page<BudgetEntry>, String>),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(AppMsg::LoadPlan);
        Self {
            view: View::Plan,
            plan: PlanState {
                data: DataState::Loading,
                meta: None,
                mode: PlanMode::View,
                incomes: vec![],
                expenses: vec![],
                validation: PlanValidation::Valid,
                save_state: SaveState::Idle,
            },
            history: HistoryState {
                data: PaginatableDataState::Loading,
            },
            api: Rc::new(ApiClient::new(API_V1_BASE_URL.clone())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::SwitchView(view) => self.handle_switch_view(ctx, view),
            AppMsg::LoadPlan => self.handle_load_plan(ctx),
            AppMsg::PlanLoaded(result) => self.handle_plan_loaded(result),
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
            AppMsg::LoadHistory => self.handle_load_history(ctx),
            AppMsg::HistoryLoaded(result) => self.handle_history_loaded(result),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <AppLayout
                current_view={self.view.clone()}
                on_switch_view={ctx.link().callback(AppMsg::SwitchView)}
                sticky_header={self.render_sticky_header()}
            >
                {self.render_content(ctx)}
            </AppLayout>
        }
    }
}

impl App {}
