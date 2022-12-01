// Stopwatch deps
use std::{
    collections::HashMap,
    fmt::Display,
    io::Stdout,
    time::{Duration, SystemTime},
};

// tui-rs deps
use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen},
    ExecutableCommand, Result,
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

fn main() {
    if let Err(e) = main_loop() {
        println!("Error: {:?}\r", e);
    }
}

fn main_loop() -> Result<()> {
    let mut terminal = setup_terminal()?;

    let mut app = App::new();
    app.start();

    let mut app_command_creator = AppCommandCreator::new();

    loop {
        if poll(Duration::from_millis(100))? {
            let event = read()?;

            if let Some(app_command) = app_command_creator.get_app_command_from_event(&event) {
                match app_command {
                    AppCommand::EnterRestMode => app.change_session_type(SessionType::Rest),
                    AppCommand::EnterFocusMode => app.change_session_type(SessionType::Focus),
                    AppCommand::Pause => app.toggle_pause(),
                    AppCommand::Quit => break,
                }
            }
        }

        draw_ui(&mut terminal, &app)?;
    }

    teardown_terminal(terminal)?;

    Ok(())
}

enum AppCommand {
    EnterRestMode,
    EnterFocusMode,
    Pause,
    Quit,
}

#[derive(Default)]
struct AppCommandCreator {
    previously_typed_chars: String,
}

impl AppCommandCreator {
    fn new() -> Self {
        Default::default()
    }

    fn get_app_command_from_event(&mut self, event: &Event) -> Option<AppCommand> {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char(char) => {
                    self.previously_typed_chars.push(char);
                    if self.previously_typed_chars.chars().count() > 4 {
                        self.previously_typed_chars = self.previously_typed_chars[1..].to_string();
                    }

                    if self.previously_typed_chars == "quit" {
                        Some(AppCommand::Quit)
                    } else {
                        match char {
                            'f' => Some(AppCommand::EnterFocusMode),
                            'r' => Some(AppCommand::EnterRestMode),
                            'p' => Some(AppCommand::Pause),
                            _ => None,
                        }
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

type StandardTerminal = Terminal<CrosstermBackend<Stdout>>;

fn teardown_terminal(mut terminal: StandardTerminal) -> Result<()> {
    let backend = terminal.backend_mut();
    backend.clear()?;
    backend.set_cursor(0, 0)?;
    disable_raw_mode()?;
    Ok(())
}

fn setup_terminal() -> Result<StandardTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn draw_ui(terminal: &mut StandardTerminal, app: &App) -> Result<()> {
    terminal.draw(|frame| {
        /*
            Layout is like this:

            ┌──────────────────────────────┐
            │ parent vertical              │
            │ ┌──────────────────────────┐ │
            │ │ session totals           │ │
            │ │ ┌──────────┬───────────┐ │ │
            │ │ │          │           │ │ │
            │ │ │          │           │ │ │
            │ │ │          │           │ │ │
            │ │ │          │           │ │ │
            │ │ └──────────┴───────────┘ │ │
            │ │                          │ │
            │ ├──────────────────────────┤ │
            │ │current session           │ │
            │ │                          │ │
            │ ├──────────────────────────┤ │
            │ │help                      │ │
            │ │                          │ │
            │ │                          │ │
            │ └──────────────────────────┘ │
            │                              │
            └──────────────────────────────┘

            Diagram made with asciiflow
        */

        let parent_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40),
            ])
            .split(frame.size());

        let session_totals_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(parent_vertical_layout[0]);

        draw_session_type_total(frame, &session_totals_layout[0], app, SessionType::Focus);
        draw_session_type_total(frame, &session_totals_layout[1], app, SessionType::Rest);
        draw_current_session(frame, &parent_vertical_layout[1], app);
        draw_help(frame, &parent_vertical_layout[2]);
    })?;

    Ok(())
}

fn draw_session_type_total(
    frame: &mut Frame<CrosstermBackend<Stdout>>,
    area: &Rect,
    app: &App,
    session_type: SessionType,
) {
    let mut title = format!("{}", session_type);
    if session_type == app.current_session_type {
        if app.paused {
            title = format!("{} (Paused)", title);
        } else {
            title = format!("{} (Active)", title);
        }
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(*area);
    let total_duration = app.get_session_type_total_duration(session_type);
    let formatted_total_duration = format_duration(total_duration);
    let total_text = vec![Spans::from(vec![Span::raw(format!(
        "Total time: {}",
        formatted_total_duration
    ))])];
    let paragraph = Paragraph::new(total_text);
    frame.render_widget(block, *area);
    frame.render_widget(paragraph, inner);
}

fn draw_current_session(frame: &mut Frame<CrosstermBackend<Stdout>>, area: &Rect, app: &App) {
    let mut statuses: Vec<String> = vec![format!("{}", app.current_session_type)];
    if app.paused {
        statuses.push("Paused".into());
    }
    let formatted_statuses = format!("({})", statuses.join(", "));
    let title = format!("{} {}", "Current Session".to_string(), formatted_statuses);
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(*area);
    let duration = app.get_current_session_duration();
    let formatted_duration = format_duration(duration);
    let text = vec![Spans::from(vec![Span::raw(formatted_duration)])];
    let paragraph = Paragraph::new(text);
    frame.render_widget(block, *area);
    frame.render_widget(paragraph, inner);
}

fn draw_help(frame: &mut Frame<CrosstermBackend<Stdout>>, area: &Rect) {
    let block = Block::default();
    let inner = block.inner(*area);
    let text = vec![
        Spans::from(vec![Span::raw("Press F to enter focus")]),
        Spans::from(vec![Span::raw("Press R to enter rest")]),
        Spans::from(vec![Span::raw("Press P to toggle pause")]),
        Spans::from(vec![Span::raw("Type quit to quit")]),
    ];
    let paragraph = Paragraph::new(text);
    frame.render_widget(block, *area);
    frame.render_widget(paragraph, inner);
}

fn format_duration(duration: Option<Duration>) -> String {
    if let Some(duration) = duration {
        let seconds = two_digit_string(duration.as_secs() % 60);
        let minutes = two_digit_string((duration.as_secs() / 60) % 60);
        let hours = two_digit_string((duration.as_secs() / 60) / 60);
        format!("{}:{}:{}", hours, minutes, seconds)
    } else {
        "00:00:00".to_string()
    }
}

fn two_digit_string(number: u64) -> String {
    let number = number.to_string();
    if number.len() < 2 {
        format!("0{}", number)
    } else {
        number
    }
}

struct App {
    sessions_by_type: HashMap<SessionType, Vec<Session>>,
    paused: bool,
    current_session_type: SessionType,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        if !self.paused {
            return;
        }
        self.paused = false;
        self.start_new_session(self.current_session_type);
    }

    pub fn pause(&mut self) {
        if self.paused {
            return;
        }
        self.paused = true;
        self.end_current_session();
    }

    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.start();
        } else {
            self.pause()
        }
    }

    pub fn change_session_type(&mut self, session_type: SessionType) {
        if self.paused {
            self.start();
        }

        if self.current_session_type == session_type {
            return;
        }

        self.end_current_session();
        self.start_new_session(session_type);
    }

    pub fn get_current_session_duration(&self) -> Option<Duration> {
        if let Some(session) = self.get_current_session() {
            let session_end = session.end.unwrap_or(SystemTime::now());
            let duration = session_end.duration_since(session.start).unwrap();
            Some(duration)
        } else {
            None
        }
    }

    pub fn get_session_type_total_duration(&self, session_type: SessionType) -> Option<Duration> {
        if let Some(sessions) = self.sessions_by_type.get(&session_type) {
            let total_duration = sessions.iter().fold(Duration::ZERO, |total, session| {
                let session_end = session.end.unwrap_or(SystemTime::now());
                let session_duration = session_end.duration_since(session.start).unwrap();
                total + session_duration
            });
            Some(total_duration)
        } else {
            None
        }
    }

    fn start_new_session(&mut self, new_session_type: SessionType) {
        self.current_session_type = new_session_type;
        let sessions = self.sessions_by_type.get_mut(&new_session_type).unwrap();
        sessions.push(Session::new());
    }

    fn end_current_session(&mut self) {
        if let Some(session) = self.get_current_session_mut() {
            if session.end.is_none() {
                session.end = Some(SystemTime::now());
            }
        }
    }

    fn get_current_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions_by_type
            .get_mut(&self.current_session_type)
            .unwrap()
            .last_mut()
    }

    fn get_current_session(&self) -> Option<&Session> {
        self.sessions_by_type
            .get(&self.current_session_type)
            .unwrap()
            .last()
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            sessions_by_type: HashMap::new(),
            paused: true,
            current_session_type: SessionType::Focus,
        };
        app.sessions_by_type.insert(SessionType::Focus, vec![]);
        app.sessions_by_type.insert(SessionType::Rest, vec![]);
        app
    }
}

struct Session {
    start: SystemTime,
    end: Option<SystemTime>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Session {
    fn default() -> Self {
        Session {
            start: SystemTime::now(),
            end: None,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
enum SessionType {
    Focus,
    Rest,
}

impl Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Focus => write!(f, "Focus"),
            SessionType::Rest => write!(f, "Rest"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn app_is_in_focus_mode_given_app_start_up() {
        let mut app = App::new();
        app.start();
        let current_mode = app.current_session_type;
        assert_eq!(current_mode, SessionType::Focus);
    }

    #[test]
    fn session_time_increases_given_time_passes() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(1));
        let duration = app.get_current_session_duration().unwrap();
        assert!(duration.as_millis() > 0);
    }

    #[test]
    fn session_type_total_time_keeps_increases_given_being_paused_and_restarted() {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(1));
        app.pause();
        thread::sleep(Duration::from_millis(1));
        app.start();
        thread::sleep(Duration::from_millis(1));
        let duration = app
            .get_session_type_total_duration(app.current_session_type)
            .unwrap();
        dbg!(duration.as_millis());
        assert!(duration.as_millis() == 2);
    }

    #[test]
    fn session_type_total_time_does_not_increase_given_it_is_paused() {
        let mut app = App::new();
        app.start();
        app.pause();
        thread::sleep(Duration::from_millis(1));
        let duration = app.get_current_session_duration().unwrap();
        assert!(duration.as_millis() == 0);
    }

    #[test]
    fn session_type_total_time_does_not_increase_given_a_different_session_type_time_is_increasing()
    {
        let mut app = App::new();
        app.start();
        thread::sleep(Duration::from_millis(1));
        app.change_session_type(SessionType::Rest);
        thread::sleep(Duration::from_millis(2));
        let duration = app
            .get_session_type_total_duration(SessionType::Focus)
            .unwrap();
        assert!(duration.as_millis() == 1);
    }
}
