use crate::error::ApplicationError;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

const SPLASH_DURATION: Duration = Duration::from_secs(3);

enum AppState {
    Splash,
    Main,
}

struct App {
    state: AppState,
    splash_start: Instant,
    selected_menu: usize,
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::Splash,
            splash_start: Instant::now(),
            selected_menu: 0,
        }
    }

    fn update(&mut self) {
        if let AppState::Splash = self.state {
            if self.splash_start.elapsed() >= SPLASH_DURATION {
                self.state = AppState::Main;
            }
        }
    }

    fn next_menu(&mut self) {
        self.selected_menu = (self.selected_menu + 1) % 4;
    }

    fn prev_menu(&mut self) {
        self.selected_menu = (self.selected_menu + 3) % 4;
    }
}

pub fn launch_tui() -> Result<(), ApplicationError> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = run_app(&mut terminal);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    res.map_err(|e| ApplicationError::IoError(e))
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        app.update();

        terminal.draw(|frame| {
            let area = frame.area();
            match app.state {
                AppState::Splash => {
                    let main_block = Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Cyan));
                    let inner_area = main_block.inner(area);
                    frame.render_widget(main_block, area);

                    let splash = Paragraph::new("MINDBENDER")
                        .style(Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center);
                    frame.render_widget(splash, inner_area);
                }
                AppState::Main => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // Title
                            Constraint::Min(0),    // Content
                            Constraint::Length(1), // Status
                        ])
                        .split(area);

                    // Title
                    let title = Paragraph::new("MINDBENDER v1.0")
                        .style(Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    // Menu items
                    let menu_items = vec![
                        (0, "Encode Message"),
                        (1, "Decode Message"),
                        (2, "Analyze File"),
                        (3, "Exit"),
                    ];

                    let menu_lines: Vec<Line> = menu_items
                        .iter()
                        .map(|(i, item)| {
                            let style = if *i == app.selected_menu {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default().fg(Color::White)
                            };
                            Line::styled(format!("  [{:^1}] {}", i + 1, item), style)
                        })
                        .collect();

                    let menu = Paragraph::new(menu_lines)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(Alignment::Left);
                    frame.render_widget(menu, chunks[1]);

                    // Status bar
                    let status = Paragraph::new("↑↓ to navigate | Enter to select | q to quit")
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[2]);
                }
            }
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Splash => {
                        app.state = AppState::Main;
                    }
                    AppState::Main => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up => app.prev_menu(),
                        KeyCode::Down => app.next_menu(),
                        KeyCode::Char('1') => app.selected_menu = 0,
                        KeyCode::Char('2') => app.selected_menu = 1,
                        KeyCode::Char('3') => app.selected_menu = 2,
                        KeyCode::Char('4') => app.selected_menu = 3,
                        _ => {}
                    },
                }
            }
        }
    }
}
