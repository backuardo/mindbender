use crate::core::operations::{self, Progress};
use crate::error::ApplicationError;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation},
    Terminal,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(PartialEq, Clone, Copy)]
enum AppState {
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
    file_type: FileSelectType,
    operation: OperationType,
}
impl FileExplorer {
    fn new(file_type: FileSelectType, operation: OperationType) -> io::Result<Self> {
        let current_path = std::env::current_dir()?;
        let mut explorer = Self {
            tree_state: TreeState::default(),
            tree_items: Vec::new(),
            current_path,
            file_type,
            operation,
        };
        explorer.refresh_entries()?;

        // Set initial selection to first item
        if !explorer.tree_items.is_empty() {
            explorer
                .tree_state
                .select(vec![explorer.tree_items[0].identifier().to_string()]);
        }

        Ok(explorer)
    }

    fn is_valid_file(&self, path: &Path) -> bool {
        match (self.file_type, self.operation) {
            (FileSelectType::Data, OperationType::Encode) => {
                // For data file, only show text files
                path.is_dir() || path.extension().map_or(false, |ext| ext == "txt")
            }
            (FileSelectType::Carrier, OperationType::Encode) => {
                // For carrier image in encode mode, show all image files
                path.is_dir()
                    || path.extension().map_or(false, |ext| {
                        let ext = ext.to_string_lossy().to_lowercase();
                        matches!(
                            ext.as_str(),
                            "png" | "jpg" | "jpeg" | "bmp" | "webp" | "tiff"
                        )
                    })
            }
            (FileSelectType::Carrier, OperationType::Decode) => {
                // For decode mode, only show PNG files as they contain the hidden message
                path.is_dir() || path.extension().map_or(false, |ext| ext == "png")
            }
            _ => false,
        }
    }

    fn build_tree_items(&self, path: &Path) -> io::Result<Vec<TreeItem<'static, String>>> {
        let mut items = Vec::new();

        // Add parent directory if not at root
        if let Some(_) = path.parent() {
            items.push(TreeItem::new_leaf("../".to_string(), "../"));
        }

        let mut entries: Vec<_> = fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| self.is_valid_file(&entry.path()))
            .collect();

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

            // Add file type indicators and additional info
            let display_text = if entry_path.is_dir() {
                format!("📁 {}", file_name)
            } else {
                let icon = match entry_path.extension().and_then(|e| e.to_str()) {
                    Some("txt") => "📄",
                    Some("png") => "🖼️",
                    Some("jpg" | "jpeg") => "📸",
                    _ => "📄",
                };
                format!("{} {}", icon, file_name)
            };

            let item = if entry_path.is_dir() {
                let children = self.build_tree_items(&entry_path)?;
                TreeItem::new(file_name.clone(), display_text, children)
            } else {
                Ok(TreeItem::new_leaf(file_name.clone(), display_text))
            };

            items.push(item.expect("unique identifiers"));
        }

        Ok(items)
    }

    fn refresh_entries(&mut self) -> io::Result<()> {
        self.tree_items = self.build_tree_items(&self.current_path)?;
        Ok(())
    }

    fn selected_path(&self) -> Option<PathBuf> {
        let selected_indices = self.tree_state.selected();
        if !selected_indices.is_empty() {
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
                    // Use the actual file name instead of display text
                    path = path.join(item.identifier());

                    // Move to children if any
                    current = item.children();
                }
            }

            Some(path)
        } else {
            None
        }
    }

    fn get_help_text(&self, file_type: FileSelectType, operation: OperationType) -> String {
        match (file_type, operation) {
            (FileSelectType::Data, OperationType::Encode) => {
                "Select a text file (.txt) containing the secret message to encode".into()
            }
            (FileSelectType::Carrier, OperationType::Encode) => {
                "Select any image file. Non-PNG files will be automatically converted.".into()
            }
            (FileSelectType::Carrier, OperationType::Decode) => {
                "Select a PNG file containing a hidden message.".into()
            }
            _ => "".into(),
        }
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
    selected_menu: usize,
    file_explorer: Option<FileExplorer>,
    data_path: Option<PathBuf>,
    carrier_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    encryption_key: Option<String>,
    progress_state: Arc<Mutex<ProgressState>>,
    progress_bar: Option<ProgressBar>,
}

impl App {
    fn new() -> io::Result<Self> {
        Ok(Self {
            state: AppState::Main,
            operation: None,
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
            progress_bar: None,
        })
    }

    fn setup_progress_bar(&mut self) {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .progress_chars("█▇▆▅▄▃▂▁  "),
        );
        self.progress_bar = Some(pb);
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

    fn handle_menu_selection(&mut self) -> io::Result<()> {
        match self.selected_menu {
            0 => {
                self.operation = Some(OperationType::Encode);
                self.file_explorer = Some(FileExplorer::new(
                    FileSelectType::Data,
                    OperationType::Encode,
                )?);
                self.state = AppState::FileSelect(FileSelectType::Data);
            }
            1 => {
                self.operation = Some(OperationType::Decode);
                self.file_explorer = Some(FileExplorer::new(
                    FileSelectType::Carrier,
                    OperationType::Decode,
                )?);
                self.state = AppState::FileSelect(FileSelectType::Carrier);
            }
            2 => self.state = AppState::KeyInput,
            3 => return Ok(()),
            _ => {}
        }
        Ok(())
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
        terminal.draw(|frame| {
            let area = frame.area();
            match app.state {
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

                    let status = Paragraph::new("↑↓ to navigate | Enter to select | q to quit")
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[3]);
                }
                // Add this to the file explorer rendering
                AppState::FileSelect(select_type) => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // Title/Instructions
                            Constraint::Length(2), // Help text
                            Constraint::Min(0),    // Main content
                            Constraint::Length(3), // File info
                            Constraint::Length(1), // Status
                        ])
                        .split(area);

                    let instruction = app.get_instruction_message();
                    let title = Paragraph::new(instruction)
                        .style(Style::default().fg(Color::Cyan))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    // Show context-sensitive help
                    if let Some(explorer) = &app.file_explorer {
                        let help_text = explorer.get_help_text(
                            select_type,
                            app.operation.unwrap_or(OperationType::Encode),
                        );
                        let help = Paragraph::new(help_text)
                            .style(Style::default().fg(Color::Gray))
                            .alignment(Alignment::Center);
                        frame.render_widget(help, chunks[1]);
                    }

                    // Split main content area
                    let content_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                        .split(chunks[2]);

                    if let Some(explorer) = &mut app.file_explorer {
                        // Render file explorer
                        explorer.render(frame, content_chunks[0]);

                        // Show file info/preview
                        if let Some(path) = explorer.selected_path() {
                            let file_info = if path.is_file() {
                                let metadata = fs::metadata(&path).ok();
                                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                                let modified = metadata
                                    .and_then(|m| m.modified().ok())
                                    .map(|t| t.elapsed().unwrap_or_default())
                                    .map(|e| format!("{:.1?} ago", e))
                                    .unwrap_or_else(|| "Unknown".into());

                                format!(
                                    "File: {}\nSize: {:.1} KB\nModified: {}",
                                    path.file_name().unwrap_or_default().to_string_lossy(),
                                    size as f64 / 1024.0,
                                    modified,
                                )
                            } else {
                                format!("Directory: {}", path.display())
                            };

                            let info_widget = Paragraph::new(file_info)
                                .block(Block::default().borders(Borders::ALL).title("File Info"))
                                .alignment(Alignment::Left);
                            frame.render_widget(info_widget, content_chunks[1]);
                        }
                    }

                    let status = Paragraph::new(format!(
                        "↑↓ navigate | Enter select | Esc back | {} files shown",
                        app.file_explorer
                            .as_ref()
                            .map(|e| e.tree_items.len())
                            .unwrap_or(0)
                    ))
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[4]);
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
                            Constraint::Length(3), // Title
                            Constraint::Length(3), // Progress info
                            Constraint::Min(0),    // Spacer for terminal flexibility
                            Constraint::Length(3), // Footer/Status bar
                        ])
                        .split(area);

                    // Initialize progress bar first, before borrowing progress_state
                    if app.progress_bar.is_none() {
                        app.setup_progress_bar();
                    }

                    // Now get the progress state
                    let progress_state = app.progress_state.lock().unwrap();

                    if progress_state.progress >= 1.0 {
                        // Drop state before changing app state
                        drop(progress_state);
                        if let Some(pb) = app.progress_bar.take() {
                            pb.finish_and_clear();
                        }
                        app.state = AppState::Complete;
                    } else {
                        let title = Paragraph::new(progress_state.message.clone())
                            .style(Style::default().fg(Color::Cyan))
                            .alignment(Alignment::Center)
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title("Processing..."),
                            );
                        frame.render_widget(title, chunks[0]);

                        // Update and render progress bar
                        if let Some(pb) = &app.progress_bar {
                            pb.set_message(progress_state.message.clone());
                            pb.set_position((progress_state.progress * 100.0) as u64);

                            // Get the progress bar display
                            let progress_display = pb.message();
                            let progress_widget = Paragraph::new(progress_display)
                                .block(Block::default().borders(Borders::ALL))
                                .alignment(Alignment::Left);
                            frame.render_widget(progress_widget, chunks[1]);
                        }
                    }
                }
                AppState::Complete => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // Title
                            Constraint::Length(3), // Result
                            Constraint::Length(3), // Stats
                            Constraint::Min(0),    // Extra space
                            Constraint::Length(1), // Status
                        ])
                        .split(area);

                    let progress_state = app.progress_state.lock().unwrap();

                    let title_style = if progress_state
                        .result
                        .as_ref()
                        .map_or(false, |r| r.contains("failed"))
                    {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    let title = Paragraph::new("Operation Complete")
                        .style(title_style)
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(title, chunks[0]);

                    if let Some(ref result) = progress_state.result {
                        let message = Paragraph::new(result.as_str())  // Changed this line
                            .block(Block::default().borders(Borders::ALL))
                            .alignment(Alignment::Center);
                        frame.render_widget(message, chunks[1]);
                    }

                    let status = Paragraph::new("Press any key to continue")
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Center);
                    frame.render_widget(status, chunks[4]);
                }
            }
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
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
                        KeyCode::Enter => {
                            app.handle_menu_selection()?;
                        }
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
                                            app.file_explorer = Some(FileExplorer::new(
                                                FileSelectType::Carrier,
                                                OperationType::Encode,
                                            )?);
                                            app.state =
                                                AppState::FileSelect(FileSelectType::Carrier);
                                        }
                                        FileSelectType::Carrier => {
                                            app.carrier_path = Some(path);
                                            app.state = AppState::KeyInput;
                                        }
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
                        _ => {} // Keep a single catch-all at the end
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
