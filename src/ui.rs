use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

const PROGRESS_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
const PROGRESS_INTERVAL: u64 = 80;

pub struct ProgressTracker {
    progress: ProgressBar,
}

impl ProgressTracker {
    pub fn new() -> Self {
        let progress = ProgressBar::new(100);
        progress.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
                .tick_chars(PROGRESS_CHARS),
        );
        progress.enable_steady_tick(Duration::from_millis(PROGRESS_INTERVAL));

        Self { progress }
    }

    pub fn update(&self, message: &str) {
        let styled_message = message.bright_green().bold().italic().to_string();
        self.progress.set_message(styled_message);
    }

    pub fn finish_with_message(&self, message: &str) {
        self.progress
            .finish_with_message(message.green().bold().to_string());
    }
}
