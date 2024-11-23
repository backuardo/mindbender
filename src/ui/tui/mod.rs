use crate::core::operations::{self, Progress};
use crate::error::ApplicationError;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, Paragraph, Scrollbar, ScrollbarOrientation},
    Terminal,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const SPLASH_DURATION: Duration = Duration::from_secs(3);

#[derive(PartialEq, Clone, Copy)]
enum AppState {
    Splash,
    Main,
    FileSelect(FileSelectType),
    KeyInput,
    Processing,
    Complete,
}

#[derive(PartialEq, Clone, Copy)]
enum FileSelectType {
    Data,
    Carrier,
}

#[derive(PartialEq, Clone, Copy)]
enum OperationType {
    Encode,
    Decode,
}

struct ProgressState {
    message: String,
    progress: f64,
    result: Option<String>,
}

struct TuiProgress {
    state: Arc<Mutex<ProgressState>>,
}

impl Progress for TuiProgress {
    fn update(&self, message: &str) {
        let mut state = self.state.lock().unwrap();
        state.message = message.to_string();
        // Map the messages to progress values more granularly
        state.progress = match message {
            "Starting operation..." => 0.0,
            "Loading carrier image..." => 0.1,
            "Reading data file..." => 0.2,
            "Encrypting data..." => 0.4,
            "Encoding data into image..." => 0.6,
            "Saving encoded image..." => 0.8,
            "Decoding data from image..." => 0.5,
            "Decrypting data..." => 0.7,
            "Saving decoded message..." => 0.9,
            _ => state.progress,
        };
    }

    fn finish_with_message(&self, message: &str) {
        let mut state = self.state.lock().unwrap();
        state.message = message.to_string();
        state.progress = 1.0;
        state.result = Some(message.to_string());
    }
}

struct FileExplorer {
    tree_state: TreeState<String>,
    tree_items: Vec<TreeItem<'static, String>>,
    current_path: PathBuf,
}

impl FileExplorer {
    fn new() -> io::Result<Self> {
        let current_path = std::env::current_dir()?;
        println!("Starting path: {:?}", current_path); // Debug
        let mut explorer = Self {
            tree_state: TreeState::default(),
            tree_items: Vec::new(),
            current_path,
        };
        explorer.refresh_entries()?;
        Ok(explorer)
    }

    fn build_tree_items(path: &Path) -> io::Result<Vec<TreeItem<'static, String>>> {
        let mut items = Vec::new();

        // Add parent directory if not at root
        if let Some(_) = path.parent() {
            items.push(TreeItem::new_leaf("../".to_string(), "../"));
        }

        let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(|entry| entry.ok()).collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let display_text = if entry_path.is_dir() {
                format!("📁 {}", file_name)
            } else {
                format!("📄 {}", file_name)
            };

            if entry_path.is_dir() {
                let children = Self::build_tree_items(&entry_path)?;
                items.push(
                    TreeItem::new(
                        display_text.clone(), // Use display text as identifier
                        display_text,         // And as display text
                        children,
                    )
                    .expect("unique identifiers"),
                );
            } else {
                items.push(TreeItem::new_leaf(
                    display_text.clone(), // Use display text as identifier
                    display_text,         // And as display text
                ));
            }
        }

        Ok(items)
    }

    fn refresh_entries(&mut self) -> io::Result<()> {
        println!("Refreshing entries for path: {:?}", self.current_path); // Debug
        self.tree_items = Self::build_tree_items(&self.current_path)?;
        Ok(())
    }

    fn selected_path(&self) -> Option<PathBuf> {
        let selected_indices = self.tree_state.selected();
        if !selected_indices.is_empty() {
            // Change type to slice
            let mut current: &[TreeItem<String>] = &self.tree_items;
            let mut path = self.current_path.clone();

            for selected_name in selected_indices {
                if selected_name == "../" {
                    path = path.parent()?.to_path_buf();
                    continue;
                }

                // Find matching item
                if let Some(item) = current
                    .iter()
                    .find(|item| item.identifier() == selected_name)
                {
                    // Add this part to the path
                    let file_name = selected_name
                        .trim_start_matches("📁 ")
                        .trim_start_matches("📄 ");
                    path = path.join(file_name);

                    // Move to children if any
                    current = item.children();
                }
            }

            Some(path)
        } else {
            None
        }
    }

    fn get_debug_info(&self) -> String {
        let selected = self.tree_state.selected();
        let path = self.selected_path();
        format!(
            "Current path: {:?}\nSelected: {:?}\nResolved path: {:?}",
            self.current_path, selected, path
        )
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let widget = Tree::new(&self.tree_items)
            .expect("unique identifiers")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", self.current_path.display())),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("→ ")
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ));

        frame.render_stateful_widget(widget, area, &mut self.tree_state);
    }
}

struct App {
    state: AppState,
    operation: Option<OperationType>,
    splash_start: Instant,
    selected_menu: usize,
    file_explorer: Option<FileExplorer>,
    data_path: Option<PathBuf>,
    carrier_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    encryption_key: Option<String>,
    progress_state: Arc<Mutex<ProgressState>>,
}

impl App {
    fn new() -> io::Result<Self> {
        Ok(Self {
            state: AppState::Splash,
            operation: None,
            splash_start: Instant::now(),
            selected_menu: 0,
            file_explorer: None,
            data_path: None,
            carrier_path: None,
            output_path: None,
            encryption_key: None,
            progress_state: Arc::new(Mutex::new(ProgressState {
                message: String::from("Welcome to Mindbender"),
                progress: 0.0,
                result: None,
            })),
        })
    }

    fn update(&mut self) {
        if let AppState::Splash = self.state {
            if self.splash_start.elapsed() >= SPLASH_DURATION {
                self.state = AppState::Main;
            }
        }
    }

    fn get_instruction_message(&self) -> String {
        match (self.state, self.operation) {
            (AppState::FileSelect(FileSelectType::Data), Some(OperationType::Encode)) => {
                "Select the text file containing the secret message to encode".into()
            }
            (AppState::FileSelect(FileSelectType::Carrier), Some(OperationType::Encode)) => {
                "Select a carrier image (PNG or BMP). This is the image that will contain the hidden message".into()
            }
            // (AppState::FileSelect(FileSelectType::Output), Some(OperationType::Encode)) => {
            //     "Choose where to save the encoded image".into()
            // }
            (AppState::FileSelect(FileSelectType::Carrier), Some(OperationType::Decode)) => {
                "Select the image containing the hidden message".into()
            }
            // (AppState::FileSelect(FileSelectType::Output), Some(OperationType::Decode)) => {
            //     "Choose where to save the decoded message".into()
            // }
            (AppState::KeyInput, _) => {
                "Enter an encryption key (optional) - press Enter when done".into()
            }
            _ => String::new()
        }
    }

    fn get_output_path(&self) -> PathBuf {
        match self.operation {
            Some(OperationType::Encode) => {
                if let Some(carrier) = &self.carrier_path {
                    let stem = carrier.file_stem().unwrap_or_default();
                    carrier.with_file_name(format!("{}-encoded.png", stem.to_string_lossy()))
                } else {
                    PathBuf::from("encoded.png")
                }
            }
            Some(OperationType::Decode) => {
                if let Some(carrier) = &self.carrier_path {
                    let stem = carrier.file_stem().unwrap_or_default();
                    carrier.with_file_name(format!("{}-decoded.txt", stem.to_string_lossy()))
                } else {
                    PathBuf::from("decoded.txt")
                }
            }
            None => PathBuf::from("output"),
        }
    }

    fn run_operation(&mut self) -> Result<(), ApplicationError> {
        let progress = TuiProgress {
            state: Arc::clone(&self.progress_state),
        };

        {
            let mut state = self.progress_state.lock().unwrap();
            state.message = String::from("Starting operation...");
            state.progress = 0.0;
            state.result = None;
        }

        let operation = self.operation;
        let data_path = self.data_path.clone();
        let carrier_path = self.carrier_path.clone();
        let output_path = self.get_output_path();
        let encryption_key = self.encryption_key.clone();
        let progress_state = Arc::clone(&self.progress_state);

        // Debug print the paths
        println!("Data: {:?}", data_path);
        println!("Carrier: {:?}", carrier_path);

        thread::spawn(move || {
            let result = match operation {
                Some(OperationType::Encode) => {
                    if let (Some(data), Some(carrier)) = (data_path, carrier_path) {
                        operations::encode(
                            &data.to_string_lossy(),
                            &carrier.to_string_lossy(),
                            &output_path.to_string_lossy(),
                            encryption_key,
                            &progress,
                        )
                    } else {
                        Ok(())
                    }
                }
                Some(OperationType::Decode) => {
                    if let Some(carrier) = carrier_path {
                        operations::decode(
                            &carrier.to_string_lossy(),
                            &output_path.to_string_lossy(),
                            encryption_key,
                            &progress,
                        )
                    } else {
                        Ok(())
                    }
                }
                None => Ok(()),
            };

            let mut state = progress_state.lock().unwrap();
            match result {
                Ok(()) => {
                    if state.result.is_none() {
                        state.result = Some(format!(
                            "Operation completed successfully => {}",
                            output_path.display()
                        ));
                    }
                    state.progress = 1.0;
                }
                Err(e) => {
                    state.message = format!("Error: {}", e);
                    state.progress = 1.0;
                    state.result = Some(format!("Operation failed: {}", e));
                }
            }
        });

        Ok(())
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new()?;

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
                            Constraint::Length(3),
                            Constraint::Min(0),
                            Constraint::Length(3),
                            Constraint::Length(1),
                        ])
                        .split(area);

                    let title = Paragraph::new("MINDBENDER v1.0")
                        .style(Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    let menu_items = vec![
                        (0, "Encode Message"),
                        (1, "Decode Message"),
                        (2, "Settings"),
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

                    let state_info = vec![
                        format!(
                            "Data File: {}",
                            app.data_path
                                .as_ref()
                                .map_or("Not selected", |p| p.to_str().unwrap_or("Invalid path"))
                        ),
                        format!(
                            "Carrier Image: {}",
                            app.carrier_path
                                .as_ref()
                                .map_or("Not selected", |p| p.to_str().unwrap_or("Invalid path"))
                        ),
                        format!(
                            "Output Path: {}",
                            app.output_path
                                .as_ref()
                                .map_or("Default", |p| p.to_str().unwrap_or("Invalid path"))
                        ),
                        format!(
                            "Encryption: {}",
                            if app.encryption_key.is_some() {
                                "Enabled"
                            } else {
                                "Disabled"
                            }
                        ),
                    ];

                    let state_text = Paragraph::new(state_info.join("\n"))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Current State"),
                        )
                        .alignment(Alignment::Left);
                    frame.render_widget(state_text, chunks[2]);

                    let status = Paragraph::new("↑↓ to navigate | Enter to select | q to quit")
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[3]);
                }
                AppState::FileSelect(_) => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // Title
                            Constraint::Min(0),    // File explorer
                            Constraint::Length(3), // Debug info
                            Constraint::Length(1), // Status
                        ])
                        .split(area);

                    let instruction = app.get_instruction_message();
                    let title = Paragraph::new(instruction)
                        .style(Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    if let Some(explorer) = &mut app.file_explorer {
                        explorer.render(frame, chunks[1]);

                        // Add debug info
                        let debug_info = Paragraph::new(explorer.get_debug_info())
                            .block(Block::default().borders(Borders::ALL).title("Debug Info"));
                        frame.render_widget(debug_info, chunks[2]);
                    }

                    let status = Paragraph::new(
                        "↑↓ to navigate | Enter to select | Backspace to go up | Esc to cancel",
                    )
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[3]);
                }
                AppState::KeyInput => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(0),
                            Constraint::Length(1),
                        ])
                        .split(area);

                    let title = Paragraph::new("Enter Encryption Key")
                        .style(Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    let key_display = if let Some(ref key) = app.encryption_key {
                        "•".repeat(key.len())
                    } else {
                        String::new()
                    };

                    let key_input = Paragraph::new(key_display)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(Alignment::Left);
                    frame.render_widget(key_input, chunks[1]);

                    let status = Paragraph::new("Enter to confirm | Esc to cancel")
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[2]);
                }
                AppState::Processing => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(0),
                            Constraint::Length(2),
                        ])
                        .split(area);

                    let progress_state = app.progress_state.lock().unwrap();
                    if progress_state.progress >= 1.0 {
                        drop(progress_state); // Release the lock before changing state
                        app.state = AppState::Complete;
                    } else {
                        let title = Paragraph::new("Processing")
                            .style(Style::default().fg(Color::Cyan))
                            .alignment(Alignment::Center)
                            .block(Block::default().borders(Borders::ALL));
                        frame.render_widget(title, chunks[0]);

                        let message = Paragraph::new(&*progress_state.message)
                            .block(Block::default().borders(Borders::ALL))
                            .alignment(Alignment::Center);
                        frame.render_widget(message, chunks[1]);

                        let gauge = Gauge::default()
                            .block(Block::default().borders(Borders::ALL))
                            .gauge_style(Style::default().fg(Color::Cyan))
                            .ratio(progress_state.progress);
                        frame.render_widget(gauge, chunks[2]);
                    }
                }
                AppState::Complete => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(0),
                            Constraint::Length(1),
                        ])
                        .split(area);

                    let title = Paragraph::new("Operation Complete")
                        .style(Style::default().fg(Color::Green))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    let progress_state = app.progress_state.lock().unwrap();
                    if let Some(ref result) = progress_state.result {
                        let message = Paragraph::new(result.as_str())
                            .block(Block::default().borders(Borders::ALL))
                            .alignment(Alignment::Center);
                        frame.render_widget(message, chunks[1]);
                    }

                    let status = Paragraph::new("Press any key to continue")
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
                        KeyCode::Up => {
                            if app.selected_menu > 0 {
                                app.selected_menu -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.selected_menu < 3 {
                                app.selected_menu += 1;
                            }
                        }
                        KeyCode::Enter => match app.selected_menu {
                            0 => {
                                app.operation = Some(OperationType::Encode);
                                app.file_explorer = Some(FileExplorer::new()?);
                                app.state = AppState::FileSelect(FileSelectType::Data);
                            }
                            1 => {
                                app.operation = Some(OperationType::Decode);
                                app.file_explorer = Some(FileExplorer::new()?);
                                app.state = AppState::FileSelect(FileSelectType::Carrier);
                            }
                            2 => app.state = AppState::KeyInput,
                            3 => return Ok(()),
                            _ => {}
                        },
                        _ => {}
                    },
                    AppState::FileSelect(select_type) => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Main;
                            app.file_explorer = None;
                        }
                        KeyCode::Enter => {
                            if let Some(ref mut explorer) = app.file_explorer {
                                if let Some(path) = explorer.selected_path() {
                                    match select_type {
                                        FileSelectType::Data => {
                                            app.data_path = Some(path);
                                            app.state =
                                                AppState::FileSelect(FileSelectType::Carrier);
                                        }
                                        FileSelectType::Carrier => {
                                            app.carrier_path = Some(path);
                                            app.state = AppState::KeyInput;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        KeyCode::Up => {
                            if let Some(ref mut explorer) = app.file_explorer {
                                explorer.tree_state.key_up();
                            }
                        }
                        KeyCode::Down => {
                            if let Some(ref mut explorer) = app.file_explorer {
                                explorer.tree_state.key_down();
                            }
                        }
                        KeyCode::Right => {
                            if let Some(ref mut explorer) = app.file_explorer {
                                explorer.tree_state.key_right();
                            }
                        }
                        KeyCode::Left => {
                            if let Some(ref mut explorer) = app.file_explorer {
                                explorer.tree_state.key_left();
                            }
                        }
                        _ => {}
                    },
                    AppState::KeyInput => match key.code {
                        KeyCode::Esc => {
                            app.state = AppState::Main;
                            app.encryption_key = None;
                        }
                        KeyCode::Enter => {
                            let output_path = app.get_output_path();
                            app.output_path = Some(output_path);
                            app.state = AppState::Processing;
                            if let Err(e) = app.run_operation() {
                                let mut state = app.progress_state.lock().unwrap();
                                state.message = format!("Error: {}", e);
                                state.progress = 1.0;
                                app.state = AppState::Complete;
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(ref mut key) = app.encryption_key {
                                key.push(c);
                            } else {
                                app.encryption_key = Some(c.to_string());
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(ref mut key) = app.encryption_key {
                                key.pop();
                                if key.is_empty() {
                                    app.encryption_key = None;
                                }
                            }
                        }
                        _ => {}
                    },
                    AppState::Processing => {}
                    AppState::Complete => {
                        app.state = AppState::Main;
                        app.progress_state = Arc::new(Mutex::new(ProgressState {
                            message: String::from("Welcome to Mindbender"),
                            progress: 0.0,
                            result: None,
                        }));
                    }
                }
            }
        }
    }
}
