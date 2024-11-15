use crate::error::ApplicationError;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Style},
    widgets::Paragraph,
    Terminal,
};
use std::io;

pub fn launch_tui() -> Result<(), ApplicationError> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app and handle cleanup
    let res = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Return potential error
    res.map_err(|e| ApplicationError::IoError(e))
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let area = frame.area();
            let hello = Paragraph::new("Mindbender\nPress 'q' to quit.")
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center);
            frame.render_widget(hello, area);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}
