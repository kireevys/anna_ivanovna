use crossterm::{
    event, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::{io, rc::Rc, vec};
use thiserror::Error;
use tracing::error;

use crate::api::{self, CoreRepo};
use crate::tree::{PlanNode, TreeNode};

const WELCOME: &str = "Anna Ivanovna помогает автоматически распределять ваши доходы по заранее составленному плану бюджета.\n\nСоздайте план один раз, и программа будет автоматически рассчитывать, сколько денег тратить на каждую категорию при получении дохода.";

const PAGE_SIZE: usize = 20;
#[derive(Debug, Error)]
pub enum Error {
    #[error("cant init tui")]
    TuiInit,
    #[error("cant load plan")]
    PlanLoad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Quit,
    Back,
    History,
    Menu,
    Down,
    Up,
    Right,
    Left,
    Enter,
    NewIncome,
    Unknown,
}

impl From<&event::KeyCode> for Action {
    fn from(value: &event::KeyCode) -> Self {
        match value {
            event::KeyCode::Char(c) => {
                let c = c.to_lowercase().next().unwrap_or(*c);
                match c {
                    'q' | 'й' => Action::Quit,
                    'b' | 'и' => Action::Back,
                    'h' | 'р' => Action::History,
                    'm' | 'ь' => Action::Menu,
                    'j' | 'о' => Action::Down,
                    'k' | 'л' => Action::Up,
                    'n' | 'т' => Action::NewIncome,
                    _ => Action::Unknown,
                }
            }
            event::KeyCode::Down => Action::Down,
            event::KeyCode::Up => Action::Up,
            event::KeyCode::Right => Action::Right,
            event::KeyCode::Left => Action::Left,
            event::KeyCode::Enter => Action::Enter,
            event::KeyCode::Esc => Action::Menu,
            _ => Action::Unknown,
        }
    }
}

impl Action {
    fn hint(&self) -> &'static str {
        match self {
            Action::Quit => "q — выход",
            Action::Back => "b — назад",
            Action::History => "h — история",
            Action::Menu => "Esc — меню",
            Action::Down => "↓ — вниз",
            Action::Up => "↑ — вверх",
            Action::Right => "→ — влево",
            Action::Left => "← — вправо",
            Action::Enter => "Enter — ввод",
            Action::NewIncome => "Enter — новый доход",
            Action::Unknown => "",
        }
    }
    fn hint_style(&self) -> Style {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SaveChoice {
    Yes,
    No,
}

impl SaveChoice {
    fn swap(self) -> Self {
        match self {
            SaveChoice::Yes => SaveChoice::No,
            SaveChoice::No => SaveChoice::Yes,
        }
    }
    fn style(&self, selected: bool) -> Style {
        if selected {
            match self {
                SaveChoice::Yes => Style::default()
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                SaveChoice::No => Style::default().bg(Color::Red).add_modifier(Modifier::BOLD),
            }
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }
}

impl std::fmt::Display for SaveChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveChoice::Yes => write!(f, "Да"),
            SaveChoice::No => write!(f, "Нет"),
        }
    }
}

#[derive(Clone, Debug)]
enum Screen {
    Welcome {
        selected_plan_index: usize,
    },
    Budgets {
        selected: usize,
        tree: Option<TreeNode<PlanNode>>,
        history_items: Vec<String>,
        error: Option<String>,
    },
    NewIncome {
        selected_source: usize,
        amount: String,
        error: Option<String>,
    },
    DistributionResult {
        budget: crate::core::distribute::Budget,
        selected: SaveChoice,
    },
    #[allow(dead_code)]
    Error {
        message: String,
        back_to: Box<Screen>,
    },
}

impl Screen {
    fn hints(&self) -> Paragraph {
        let actions = match self {
            Screen::Welcome { .. } => vec![
                Action::Down,
                Action::Up,
                Action::NewIncome,
                Action::History,
                Action::Quit,
            ],
            Screen::Budgets { .. } => vec![
                Action::Down,
                Action::Up,
                Action::Left,
                Action::Right,
                Action::Menu,
                Action::Quit,
            ],
            Screen::NewIncome { .. } => vec![
                Action::Down,
                Action::Up,
                Action::Enter,
                Action::Menu,
                Action::Back,
                Action::Quit,
            ],
            Screen::DistributionResult { .. } => vec![Action::Left, Action::Right, Action::Enter],
            Screen::Error { .. } => vec![Action::Menu, Action::Back, Action::History, Action::Quit],
        };
        let spans = actions
            .into_iter()
            .filter(|a| !a.hint().is_empty())
            .map(|a| Span::styled(format!(" {} ", a.hint()), a.hint_style()))
            .collect::<Vec<_>>();
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
    }
}

// Компоненты TUI
#[derive(Clone, Debug)]
enum Component<'a> {
    Greeting {
        storage_name: &'a str,
    },
    WelcomeText,
    AddIncomeButton,
    Plan {
        plan_tree: &'a TreeNode<PlanNode>,
        selected: usize,
        area_height: usize,
    },
    HistoryList {
        items: &'a [String],
        selected: usize,
        area_height: usize,
    },
    BudgetDetails {
        budget_tree: Option<&'a TreeNode<PlanNode>>,
        error: Option<&'a str>,
    },
    IncomeInput {
        sources: &'a [crate::core::planning::IncomeSource],
        selected_source: usize,
        amount: &'a str,
        error: Option<&'a str>,
    },
    FooterHints {
        screen: &'a Screen,
    },
}

enum WidgetComponent<'a> {
    Paragraph(Paragraph<'a>),
    List(List<'a>),
}

fn component_widget<'a>(component: &Component<'a>) -> WidgetComponent<'a> {
    match component {
        Component::Greeting { storage_name } => WidgetComponent::Paragraph(
            Paragraph::new(format!("Хранилище: {storage_name}"))
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        ),
        Component::WelcomeText => WidgetComponent::Paragraph(
            Paragraph::new(WELCOME)
                .block(Block::default().title("Добро пожаловать!"))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
        ),
        Component::AddIncomeButton => WidgetComponent::Paragraph(
            Paragraph::new("[ Enter / n ] Добавить новый доход")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .bg(Color::Green)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
        ),
        Component::Plan {
            plan_tree,
            selected,
            area_height,
        } => {
            let plan_items_raw: Vec<ListItem<'_>> = (*plan_tree).into();
            let height = *area_height;
            let mut plan_items = Vec::new();
            let offset = if *selected >= height {
                *selected + 1 - height
            } else {
                0
            };
            for (i, item) in plan_items_raw
                .into_iter()
                .enumerate()
                .skip(offset)
                .take(height)
            {
                let mut item = item;
                if i == *selected {
                    item = item.style(
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                plan_items.push(item);
            }
            WidgetComponent::List(
                List::new(plan_items)
                    .block(Block::default().title("Текущий план").borders(Borders::ALL)),
            )
        }
        Component::HistoryList {
            items,
            selected,
            area_height,
        } => {
            let height = *area_height;
            let mut list_items = Vec::new();
            let orig_len = items
                .iter()
                .filter(|s| !s.contains("←") && !s.contains("→"))
                .count();
            let offset = if *selected >= height {
                *selected + 1 - height
            } else {
                0
            };
            for (i, item) in items.iter().enumerate().skip(offset).take(height) {
                let mut list_item = ListItem::new(item.clone());
                if i >= orig_len {
                    // Пагинационные элементы — всегда жёлтые
                    if i == *selected {
                        list_item = list_item.style(
                            Style::default()
                                .bg(Color::Cyan)
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        );
                    } else {
                        list_item = list_item.style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                } else if i == *selected {
                    list_item = list_item.style(
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                list_items.push(list_item);
            }
            WidgetComponent::List(
                List::new(list_items)
                    .block(Block::default().title("История").borders(Borders::ALL)),
            )
        }
        Component::BudgetDetails { budget_tree, error } => {
            if let Some(tree) = budget_tree {
                let items: Vec<ListItem> = (*tree).into();
                WidgetComponent::List(
                    List::new(items).block(Block::default().title("Детали").borders(Borders::ALL)),
                )
            } else if let Some(err) = error {
                WidgetComponent::Paragraph(
                    Paragraph::new(err.to_string())
                        .style(Style::default().fg(Color::Red))
                        .block(Block::default().title("Детали").borders(Borders::ALL))
                        .wrap(Wrap { trim: true }),
                )
            } else {
                WidgetComponent::Paragraph(
                    Paragraph::new("Нет данных для отображения")
                        .style(Style::default().fg(Color::Red))
                        .block(Block::default().title("Детали").borders(Borders::ALL))
                        .wrap(Wrap { trim: true }),
                )
            }
        }
        Component::IncomeInput {
            sources,
            selected_source,
            amount,
            error,
        } => {
            // Вертикальный layout: поле суммы, список источников, ошибка (если есть)
            let mut lines = vec![Line::from(vec![Span::styled(
                format!("Сумма: {amount}"),
                Style::default().add_modifier(Modifier::BOLD),
            )])];
            lines.push(Line::from(""));
            for (i, src) in sources.iter().enumerate() {
                let mut style = Style::default();
                if i == *selected_source {
                    style = style
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD);
                }
                lines.push(Line::from(Span::styled(format!("{src}"), style)));
            }
            if let Some(err) = error {
                lines.push(Line::from(Span::styled(
                    err.to_string(),
                    Style::default().fg(Color::Red),
                )));
            }
            WidgetComponent::Paragraph(
                Paragraph::new(lines)
                    .block(Block::default().title("Новый доход").borders(Borders::ALL))
                    .wrap(Wrap { trim: true }),
            )
        }
        Component::FooterHints { screen } => WidgetComponent::Paragraph(screen.hints()),
    }
}

pub fn run_tui<R: CoreRepo>(repo: &R) -> Result<(), Error> {
    let storage_name = repo.location();
    enable_raw_mode().map_err(|e| {
        error!("[TUI] Ошибка enable_raw_mode: {e}");
        Error::TuiInit
    })?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| {
        error!("[TUI] Ошибка EnterAlternateScreen: {e}");
        Error::TuiInit
    })?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        error!("[TUI] Ошибка Terminal::new: {e}");
        Error::TuiInit
    })?;

    let res = run_app(&mut terminal, repo, storage_name).map_err(|e| {
        error!("[TUI] Ошибка run_app: {e}");
        Error::TuiInit
    });

    disable_raw_mode().map_err(|e| {
        error!("[TUI] Ошибка disable_raw_mode: {e}");
        Error::TuiInit
    })?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| {
        error!("[TUI] Ошибка LeaveAlternateScreen: {e}");
        Error::TuiInit
    })?;
    terminal.show_cursor().map_err(|e| {
        error!("[TUI] Ошибка show_cursor: {e}");
        Error::TuiInit
    })?;
    res
}

fn build_buidget<R: CoreRepo>(
    repo: &R,
    mut selected: usize,
    page: &api::Page<String>,
    has_prev: bool,
    has_next: bool,
) -> Screen {
    let mut history_items: Vec<String> = page.iter().map(|id| format!("> {id}")).collect();
    // Добавляем пагинационные кнопки в конец списка
    let pag_items = match (has_prev, has_next) {
        (true, true) => vec!["← Назад Вперед →".to_string()],
        (true, false) => vec!["← Назад".to_string()],
        (false, true) => vec!["       Вперед →".to_string()],
        (false, false) => vec![],
    };
    let orig_len = history_items.len();
    history_items.extend(pag_items);
    // selected не должен выходить за пределы истории
    if selected >= orig_len && orig_len > 0 {
        selected = orig_len - 1;
    }
    if page.is_empty() {
        selected = 0;
    }
    let mut trees: Vec<TreeNode<PlanNode>> = Vec::new();
    let mut tree = None;
    let mut error = None;
    let budget_index = selected;
    if !page.is_empty() && selected < orig_len {
        if let Some(budget_id) = page.get(budget_index) {
            if let Some(budget) = api::budget_by_id(repo, budget_id) {
                trees.push(TreeNode::<PlanNode>::from(&budget.budget));
                let tree_ref = trees.last().unwrap();
                tree = Some(tree_ref.clone());
            } else {
                error = Some("Ошибка загрузки бюджета".to_string());
            }
        } else {
            error = Some("Не выбран бюджет".to_string());
        }
    } else if page.is_empty() {
        error = Some("Нет данных для отображения".to_string());
    }
    Screen::Budgets {
        selected,
        tree,
        history_items,
        error,
    }
}

fn run_app<B: Backend, R: CoreRepo>(
    terminal: &mut Terminal<B>,
    repo: &R,
    storage_name: &str,
) -> io::Result<()> {
    let mut screen = Screen::Welcome {
        selected_plan_index: 0,
    };
    let mut page = api::budget_ids(repo, None, PAGE_SIZE);
    let selected: usize = 0;
    let mut next_cursor = page.next_cursor.clone();
    let mut prev_cursors: Vec<Option<String>> = vec![None];

    let red = Rc::new(Style::default().fg(Color::Red));

    let plan = api::get_plan(repo).unwrap();
    let plan_tree = TreeNode::<PlanNode>::from(&plan);
    let income_sources = plan.sources.clone();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().borders(Borders::ALL);
            match &screen {
                Screen::Welcome {
                    selected_plan_index,
                } => {
                    let vmain = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(1),                                          // storage_name
                            Constraint::Length((size.height as f32 * 0.13).round() as u16), // приветствие 13%
                            Constraint::Length((size.height as f32 * 0.03).max(1.0).round() as u16), // кнопка 3%
                            Constraint::Min(1),    // план
                            Constraint::Length(1), // футер
                        ])
                        .split(size);
                    // Компоненты
                    let components = vec![
                        Component::Greeting { storage_name },
                        Component::WelcomeText,
                        Component::AddIncomeButton,
                        Component::Plan {
                            plan_tree: &plan_tree,
                            selected: *selected_plan_index,
                            area_height: vmain[3].height as usize,
                        },
                        Component::FooterHints { screen: &screen },
                    ];
                    // Рендер
                    for (i, comp) in components.iter().enumerate() {
                        match component_widget(comp) {
                            WidgetComponent::Paragraph(p) => f.render_widget(p, vmain[i]),
                            WidgetComponent::List(l) => f.render_widget(l, vmain[i]),
                        }
                    }
                }
                Screen::Budgets {
                    selected,
                    tree,
                    history_items,
                    error,
                } => {
                    // Вертикальный layout: основной контент + хинты
                    let vchunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(1),    // основной контент
                            Constraint::Length(1), // хинты
                        ])
                        .split(size);

                    // Горизонтальный split внутри основного контента
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(vchunks[0]);

                    // История
                    let history_component = Component::HistoryList {
                        items: history_items,
                        selected: *selected,
                        area_height: chunks[0].height as usize,
                    };
                    // Детали (используем отдельный вектор для хранения деревьев)
                    let details_component = Component::BudgetDetails {
                        budget_tree: tree.as_ref(),
                        error: error.as_deref(),
                    };
                    let footer = Component::FooterHints { screen: &screen };
                    // Рендер
                    // История
                    match component_widget(&history_component) {
                        WidgetComponent::Paragraph(p) => f.render_widget(p, chunks[0]),
                        WidgetComponent::List(l) => f.render_widget(l, chunks[0]),
                    }
                    // Детали
                    match component_widget(&details_component) {
                        WidgetComponent::Paragraph(p) => f.render_widget(p, chunks[1]),
                        WidgetComponent::List(l) => f.render_widget(l, chunks[1]),
                    }
                    // Хинты
                    match component_widget(&footer) {
                        WidgetComponent::Paragraph(p) => f.render_widget(p, vchunks[1]),
                        WidgetComponent::List(l) => f.render_widget(l, vchunks[1]),
                    }
                }
                Screen::NewIncome {
                    selected_source,
                    amount,
                    error,
                } => {
                    let vchunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(3),    // основной контент (всё)
                            Constraint::Length(1), // хинты
                        ])
                        .split(size);
                    let comps = [
                        Component::IncomeInput {
                            sources: &income_sources,
                            selected_source: *selected_source,
                            amount,
                            error: error.as_deref(),
                        },
                        Component::FooterHints { screen: &screen },
                    ];
                    for (i, comp) in comps.iter().enumerate() {
                        let area = vchunks[i];
                        match component_widget(comp) {
                            WidgetComponent::Paragraph(p) => f.render_widget(p, area),
                            WidgetComponent::List(l) => f.render_widget(l, area),
                        }
                    }
                }
                Screen::DistributionResult { budget, selected } => {
                    let vchunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(5),
                            Constraint::Length(3),
                            Constraint::Length(1), // подпись
                        ])
                        .split(size);
                    let tree = TreeNode::<PlanNode>::from(budget);
                    let items: Vec<ListItem> = (&tree).into();
                    let list =
                        List::new(items).block(block.clone().title("Результат распределения"));
                    f.render_widget(list, vchunks[0]);
                    // Кнопки Да/Нет
                    let save_hint =
                        Paragraph::new("Сохранить результат?\nЭто может привести к перезаписи")
                            .alignment(Alignment::Center);
                    f.render_widget(save_hint, vchunks[1]);
                    let btns = [SaveChoice::Yes, SaveChoice::No];
                    let btns_row = btns
                        .iter()
                        .map(|choice| {
                            let selected_flag = *selected == *choice;
                            let style = choice.style(selected_flag);
                            let label = format!(" {choice} ");
                            Span::styled(label, style)
                        })
                        .collect::<Vec<_>>();
                    let btns_line = Line::from(btns_row).alignment(Alignment::Center);
                    let btns_paragraph = Paragraph::new(btns_line);
                    f.render_widget(btns_paragraph, vchunks[2]);
                }
                Screen::Error { message, .. } => {
                    let vchunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(1)])
                        .split(size);
                    f.render_widget(Clear, vchunks[0]);
                    let error_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(*red)
                        .title("❌ Ошибка");
                    let error_paragraph = Paragraph::new(message.to_string())
                        .block(error_block)
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: true });
                    f.render_widget(error_paragraph, vchunks[0]);
                    // Хинты
                    f.render_widget(screen.hints(), vchunks[1]);
                }
            }
        })?;
        // --- event pool ---
        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match &mut screen {
                    Screen::Welcome {
                        selected_plan_index,
                    } => {
                        let plan_items_raw: Vec<ListItem<'_>> = (&plan_tree).into();
                        let plan_len = plan_items_raw.len();
                        const EXTRA_SCROLL: usize = 6;
                        match Action::from(&key.code) {
                            Action::Down => {
                                if *selected_plan_index + 1 < plan_len + EXTRA_SCROLL {
                                    *selected_plan_index += 1;
                                }
                            }
                            Action::Up => {
                                if *selected_plan_index > 0 {
                                    *selected_plan_index -= 1;
                                }
                            }
                            Action::NewIncome | Action::Enter => {
                                screen = Screen::NewIncome {
                                    selected_source: 0,
                                    amount: String::new(),
                                    error: None,
                                };
                            }
                            Action::History => {
                                let has_prev = prev_cursors.len() > 1;
                                let has_next = next_cursor.is_some();
                                screen = build_buidget(repo, selected, &page, has_prev, has_next);
                            }
                            Action::Quit => break,
                            _ => {}
                        }
                    }
                    Screen::Budgets { selected, .. } => {
                        let mut rebuild = false;
                        let mut next_screen = None;
                        let has_prev = prev_cursors.len() > 1;
                        let has_next = next_cursor.is_some();
                        let items_count = page.len() + has_prev as usize + has_next as usize;
                        // Пропускать "Назад" и "Вперед" при выборе
                        let min_sel = if has_prev { 1 } else { 0 };
                        let max_sel = items_count - if has_next { 2 } else { 1 };
                        match Action::from(&key.code) {
                            Action::Quit => break,
                            Action::Menu => {
                                next_screen = Some(Screen::Welcome {
                                    selected_plan_index: 0,
                                });
                            }
                            Action::History => {
                                rebuild = true;
                            }
                            Action::Down => {
                                if *selected < max_sel {
                                    *selected += 1;
                                    // если попали на "Вперед", перескочить через него
                                    if has_next && *selected == items_count - 1 {
                                        *selected = max_sel;
                                    }
                                    rebuild = true;
                                }
                            }
                            Action::Up => {
                                if *selected > min_sel {
                                    *selected -= 1;
                                    // если попали на "Назад", перескочить через него
                                    if has_prev && *selected == 0 {
                                        *selected = min_sel;
                                    }
                                    rebuild = true;
                                }
                            }
                            Action::Right => {
                                if has_next && *selected == items_count - 1 {
                                    prev_cursors.push(next_cursor.clone());
                                    page = api::budget_ids(repo, next_cursor.clone(), PAGE_SIZE);
                                    next_cursor = page.next_cursor.clone();
                                    *selected = 0;
                                } else if let Some(ref cursor) = next_cursor {
                                    prev_cursors.push(Some(cursor.clone()));
                                    page = api::budget_ids(repo, Some(cursor.clone()), PAGE_SIZE);
                                    next_cursor = page.next_cursor.clone();
                                    *selected = 0;
                                }
                                rebuild = true;
                            }
                            Action::Left => {
                                if (has_prev && *selected == 0) || prev_cursors.len() > 1 {
                                    prev_cursors.pop();
                                    let prev = prev_cursors.last().cloned().unwrap_or(None);
                                    page = api::budget_ids(repo, prev.clone(), PAGE_SIZE);
                                    next_cursor = page.next_cursor.clone();
                                    *selected = 0;
                                    rebuild = true;
                                }
                            }
                            _ => {}
                        }
                        screen = if let Some(new_screen) = next_screen {
                            new_screen
                        } else if rebuild {
                            let has_prev = prev_cursors.len() > 1;
                            let has_next = next_cursor.is_some();
                            build_buidget(repo, *selected, &page, has_prev, has_next)
                        } else {
                            screen
                        }
                    }
                    Screen::NewIncome {
                        selected_source,
                        amount,
                        error,
                    } => match Action::from(&key.code) {
                        Action::Quit => break,
                        Action::Menu | Action::Back => {
                            screen = Screen::Welcome {
                                selected_plan_index: 0,
                            };
                        }
                        Action::Down => {
                            if *selected_source + 1 < income_sources.len() {
                                *selected_source += 1;
                            }
                        }
                        Action::Up => {
                            if *selected_source > 0 {
                                *selected_source -= 1;
                            }
                        }
                        Action::Enter => {
                            // Парсим сумму и создаём Income
                            let src = income_sources.get(*selected_source).cloned();
                            let money = amount.parse::<crate::core::finance::Money>();
                            match (src, money) {
                                (Some(source), Ok(money)) => {
                                    let income =
                                        crate::core::distribute::Income::new_today(source, money);
                                    match api::distribute_budget(&plan, &income) {
                                        Ok(budget) => {
                                            screen = Screen::DistributionResult {
                                                budget,
                                                selected: SaveChoice::Yes,
                                            };
                                        }
                                        Err(e) => {
                                            *error = Some(format!("Ошибка распределения: {e}"));
                                        }
                                    }
                                }
                                (_, Err(_)) => {
                                    *error = Some("Некорректная сумма".to_string());
                                }
                                (None, _) => {
                                    *error = Some("Не выбран источник".to_string());
                                }
                            }
                        }
                        _ => {
                            // Ввод суммы
                            match key.code {
                                event::KeyCode::Char(c)
                                    if c.is_ascii_digit()
                                        || c == ','
                                        || c == '.'
                                        || c == 'б'
                                        || c == 'ю' =>
                                {
                                    if (c == '.' || c == ',' || c == 'б' || c == 'ю')
                                        && amount.contains('.')
                                    {
                                        // Уже есть точка — не добавлять ещё одну
                                    } else if c == '.' || c == ',' || c == 'б' || c == 'ю' {
                                        amount.push('.');
                                    } else {
                                        amount.push(c);
                                    }
                                }
                                event::KeyCode::Backspace => {
                                    amount.pop();
                                }
                                _ => {}
                            }
                        }
                    },
                    Screen::DistributionResult { budget, selected } => {
                        match Action::from(&key.code) {
                            Action::Left | Action::Right => {
                                *selected = selected.swap();
                            }
                            Action::Enter => {
                                let budget_cloned = budget.clone();
                                let budget = budget.clone();
                                if *selected == SaveChoice::Yes {
                                    // Да — сохранить
                                    match api::save_budget(budget, repo) {
                                        Ok(_) => {
                                            page = api::budget_ids(repo, None, PAGE_SIZE);
                                            screen = build_buidget(repo, 0, &page, false, false);
                                        }
                                        Err(e) => {
                                            screen = Screen::Error {
                                                message: format!("Ошибка сохранения: {e}"),
                                                back_to: Box::new(Screen::DistributionResult {
                                                    budget: budget_cloned,
                                                    selected: SaveChoice::Yes,
                                                }),
                                            };
                                        }
                                    }
                                } else {
                                    // Нет — возврат на главный экран
                                    screen = Screen::Welcome {
                                        selected_plan_index: 0,
                                    };
                                }
                            }
                            _ => {}
                        }
                    }
                    Screen::Error { back_to, .. } => match Action::from(&key.code) {
                        Action::Quit => break,
                        Action::Back => {
                            screen = *back_to.clone();
                        }
                        Action::Menu => {
                            screen = Screen::Welcome {
                                selected_plan_index: 0,
                            };
                        }
                        Action::History => {
                            let has_prev = prev_cursors.len() > 1;
                            let has_next = next_cursor.is_some();
                            screen = build_buidget(repo, selected, &page, has_prev, has_next);
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}
