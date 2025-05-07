use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use sysinfo::System;
use crate::cli::config::Config;

/// Input mode for the TUI
pub enum InputMode {
    /// Normal mode (navigation)
    Normal,
    /// Editing mode (form input)
    Editing,
}

/// Menu items for the TUI
#[derive(Copy, Clone)]
pub enum MenuItem {
    /// Dashboard view
    Dashboard,
    /// Servers view
    Servers,
    /// Clients view
    Clients,
    /// Settings view
    Settings,
    /// Logs view
    Logs,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Dashboard => 0,
            MenuItem::Servers => 1,
            MenuItem::Clients => 2,
            MenuItem::Settings => 3,
            MenuItem::Logs => 4,
        }
    }
}

/// Application state for the TUI
pub struct App {
    /// Current menu item
    pub menu_state: MenuItem,
    /// Server list state
    pub servers_state: ListState,
    /// Client list state
    pub clients_state: ListState,
    /// List of servers
    pub servers: Vec<String>,
    /// List of clients
    pub clients: Vec<String>,
    /// Input mode
    pub input_mode: InputMode,
    /// System information
    pub system: System,
    /// Configuration
    pub config: Config,
    /// Whether the application should exit
    pub should_quit: bool,
}

impl App {
    /// Create a new application
    pub fn new(config: Config) -> App {
        let mut servers_state = ListState::default();
        servers_state.select(Some(0));
        let mut clients_state = ListState::default();
        clients_state.select(Some(0));

        // Extract server names from config
        let servers = config.servers.iter()
            .map(|s| format!("{} ({})", s.name, s.url))
            .collect();

        // Extract client names from config
        let clients = config.clients.iter()
            .map(|c| format!("{} ({})", c.name, c.id))
            .collect();

        App {
            menu_state: MenuItem::Dashboard,
            servers_state,
            clients_state,
            servers,
            clients,
            input_mode: InputMode::Normal,
            system: System::new_all(),
            config,
            should_quit: false,
        }
    }

    /// Navigate to the next menu item
    pub fn next_menu(&mut self) {
        self.menu_state = match self.menu_state {
            MenuItem::Dashboard => MenuItem::Servers,
            MenuItem::Servers => MenuItem::Clients,
            MenuItem::Clients => MenuItem::Settings,
            MenuItem::Settings => MenuItem::Logs,
            MenuItem::Logs => MenuItem::Dashboard,
        };
    }

    /// Navigate to the previous menu item
    pub fn previous_menu(&mut self) {
        self.menu_state = match self.menu_state {
            MenuItem::Dashboard => MenuItem::Logs,
            MenuItem::Servers => MenuItem::Dashboard,
            MenuItem::Clients => MenuItem::Servers,
            MenuItem::Settings => MenuItem::Clients,
            MenuItem::Logs => MenuItem::Settings,
        };
    }

    /// Navigate to the next server
    pub fn next_server(&mut self) {
        if self.servers.is_empty() {
            return;
        }

        let i = match self.servers_state.selected() {
            Some(i) => {
                if i >= self.servers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.servers_state.select(Some(i));
    }

    /// Navigate to the previous server
    pub fn previous_server(&mut self) {
        if self.servers.is_empty() {
            return;
        }

        let i = match self.servers_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.servers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.servers_state.select(Some(i));
    }

    /// Navigate to the next client
    pub fn next_client(&mut self) {
        if self.clients.is_empty() {
            return;
        }

        let i = match self.clients_state.selected() {
            Some(i) => {
                if i >= self.clients.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.clients_state.select(Some(i));
    }

    /// Navigate to the previous client
    pub fn previous_client(&mut self) {
        if self.clients.is_empty() {
            return;
        }

        let i = match self.clients_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.clients.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.clients_state.select(Some(i));
    }

    /// Update system information
    pub fn update_system_info(&mut self) {
        self.system.refresh_all();
    }
}

/// Run the TUI application
pub fn run_tui(config: Config) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config);

    // Set up a ticker for periodic updates
    let tick_rate = Duration::from_millis(app.config.ui.refresh_rate);
    let mut last_tick = std::time::Instant::now();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                            break;
                        },
                        KeyCode::Down | KeyCode::Char('j') => {
                            match app.menu_state {
                                MenuItem::Servers => app.next_server(),
                                MenuItem::Clients => app.next_client(),
                                _ => {}
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            match app.menu_state {
                                MenuItem::Servers => app.previous_server(),
                                MenuItem::Clients => app.previous_client(),
                                _ => {}
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => app.next_menu(),
                        KeyCode::Left | KeyCode::Char('h') => app.previous_menu(),
                        KeyCode::Tab => app.next_menu(),
                        KeyCode::Char('1') => app.menu_state = MenuItem::Dashboard,
                        KeyCode::Char('2') => app.menu_state = MenuItem::Servers,
                        KeyCode::Char('3') => app.menu_state = MenuItem::Clients,
                        KeyCode::Char('4') => app.menu_state = MenuItem::Settings,
                        KeyCode::Char('5') => app.menu_state = MenuItem::Logs,
                        KeyCode::Char('e') => {
                            app.input_mode = InputMode::Editing;
                        }
                        _ => {}
                    },
                    InputMode::Editing => if key.code == KeyCode::Esc {
                        app.input_mode = InputMode::Normal;
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update_system_info();
            last_tick = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Render the UI
fn ui(f: &mut Frame, app: &mut App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
        )
        .split(f.area());

    // Create title
    let title = Paragraph::new("MCP Daemon")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Create horizontal layout for the main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(chunks[1]);

    // Create the menu
    let menu_titles = ["Dashboard", "Servers", "Clients", "Settings", "Logs"];
    let menu_lines: Vec<Line> = menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![Span::styled(
                format!("{}{}", first, rest),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            )])
        })
        .collect();

    let tabs = Tabs::new(menu_lines)
        .select(Some(app.menu_state.into())) // Wrapped in Some()
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, main_chunks[0]);

    // Render the appropriate content based on the selected menu item
    match app.menu_state {
        MenuItem::Dashboard => render_dashboard(f, app, main_chunks[1]),
        MenuItem::Servers => render_servers(f, app, main_chunks[1]),
        MenuItem::Clients => render_clients(f, app, main_chunks[1]),
        MenuItem::Settings => render_settings(f, app, main_chunks[1]),
        MenuItem::Logs => render_logs(f, app, main_chunks[1]),
    }
}

/// Render the dashboard view
fn render_dashboard(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Connections panel
    let connections = Paragraph::new(vec![
        Line::from("Connections"),
        Line::from(""),
        Line::from(format!("Servers: {} ■■", app.servers.len())),
        Line::from(format!("Clients: {} ■■■", app.clients.len())),
    ])
    .block(Block::default().title("Connections").borders(Borders::ALL));
    f.render_widget(connections, top_chunks[0]);

    // Traffic panel
    let traffic = Paragraph::new(vec![
        Line::from("Traffic"),
        Line::from(""),
        Line::from("In:   ▂▃▅▂▇█▃▂  ▂▃▅▆▇"),
        Line::from("Out:  ▂  ▂▃ ▂▃▅▂ ▂▃▂ "),
    ])
    .block(Block::default().title("Traffic").borders(Borders::ALL));
    f.render_widget(traffic, top_chunks[1]);

    // System panel
    let cpu_usage = app.system.global_cpu_usage().round() as u64;
    let mem_used = app.system.used_memory() / 1024 / 1024; // Convert to MB

    let system = Paragraph::new(vec![
        Line::from("System"),
        Line::from(""),
        Line::from(format!("CPU:  ▂▃▂  ▂▃▂  {}%", cpu_usage)),
        Line::from(format!("MEM:   ▂▂▂▃▃▃▂▂ {}MB", mem_used)),
        Line::from("NET:  ▂▃▅▂ ▂▃▂  1.2MB/s"),
    ])
    .block(Block::default().title("System").borders(Borders::ALL));
    f.render_widget(system, bottom_chunks[0]);

    // Recent events panel
    let events = Paragraph::new(vec![
        Line::from("Recent Events"),
        Line::from(""),
        Line::from("12:01 Client connected"),
        Line::from("12:00 Server connected"),
        Line::from("11:58 Started daemon"),
    ])
    .block(Block::default().title("Recent Events").borders(Borders::ALL));
    f.render_widget(events, bottom_chunks[1]);
}

/// Render the servers view
fn render_servers(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .servers
        .iter()
        .map(|s| {
            ListItem::new(Line::from(vec![Span::styled(
                s.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let servers = List::new(items)
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(servers, area, &mut app.servers_state);
}

/// Render the clients view
fn render_clients(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .clients
        .iter()
        .map(|s| {
            ListItem::new(Line::from(vec![Span::styled(
                s.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let clients = List::new(items)
        .block(Block::default().title("Clients").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(clients, area, &mut app.clients_state);
}

/// Render the settings view
fn render_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let settings = Paragraph::new(vec![
        Line::from("Settings"),
        Line::from(""),
        Line::from(format!("Log Level: {}", app.config.general.log_level)),
        Line::from(format!("Silent Mode: {}", app.config.general.silent_mode)),
        Line::from(format!("Theme: {}", app.config.ui.theme)),
        Line::from(format!("Refresh Rate: {} ms", app.config.ui.refresh_rate)),
    ])
    .block(Block::default().title("Settings").borders(Borders::ALL));
    f.render_widget(settings, area);
}

/// Render the logs view
fn render_logs(f: &mut Frame, _app: &mut App, area: Rect) {
    let logs = Paragraph::new("Logs (Not implemented yet)")
        .block(Block::default().title("Logs").borders(Borders::ALL));
    f.render_widget(logs, area);
}
