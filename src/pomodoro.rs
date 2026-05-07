use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, BufRead, Write};
use std::thread;
use std::time::Duration;

/// Pomodoro timer configuration and runner
pub struct Pomodoro {
    pub work_duration: u64,
    pub short_break: u64,
    pub long_break: u64,
    pub long_break_interval: u64,
}

impl Pomodoro {
    pub fn new(
        work_duration: u64,
        short_break: u64,
        long_break: u64,
        long_break_interval: u64,
    ) -> Self {
        Self {
            work_duration,
            short_break,
            long_break,
            long_break_interval,
        }
    }

    /// Start a pomodoro timer with terminal animation.
    ///
    /// Displays a progress bar with a spinner that counts down the work duration.
    /// After completion, prints a success message.
    pub fn start(&self, work_duration_min: u64) -> Result<()> {
        let total_secs = work_duration_min * 60;
        let pb = ProgressBar::new(total_secs);

        pb.set_style(
            ProgressStyle::with_template(
                "🍅 Pomodoro: {bar:20.cyan/blue} {msg} remaining {spinner}",
            )?
            .progress_chars("█▓░")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        for remaining in (0..total_secs).rev() {
            let mins = remaining / 60;
            let secs = remaining % 60;
            pb.set_message(format!("{:02}:{:02}", mins, secs));
            pb.set_position(total_secs - remaining);
            thread::sleep(Duration::from_secs(1));
        }

        pb.finish_with_message("✅ done!");

        println!();
        println!("🍅 Pomodoro complete!");
        Ok(())
    }

    /// Show current pomodoro settings / status.
    ///
    /// Since there is no background daemon, this just displays the configured settings.
    pub fn status(&self) -> Result<()> {
        println!("🍅 Pomodoro settings:");
        println!(
            "  Work: {}min, Short break: {}min, Long break: {}min (every {} pomodoros)",
            self.work_duration, self.short_break, self.long_break, self.long_break_interval
        );
        Ok(())
    }

    /// Stop: no background process to stop.
    pub fn stop(&self) -> Result<()> {
        println!("No active pomodoro timer.");
        Ok(())
    }

    /// Run a break timer with a simple progress bar.
    pub fn run_break(&self, break_duration_min: u64) -> Result<()> {
        let total_secs = break_duration_min * 60;
        let pb = ProgressBar::new(total_secs);

        pb.set_style(
            ProgressStyle::with_template(
                "☕ Break: {bar:20.green/yellow} {msg} remaining {spinner}",
            )?
            .progress_chars("█▓░")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        for remaining in (0..total_secs).rev() {
            let mins = remaining / 60;
            let secs = remaining % 60;
            pb.set_message(format!("{:02}:{:02}", mins, secs));
            pb.set_position(total_secs - remaining);
            thread::sleep(Duration::from_secs(1));
        }

        pb.finish_with_message("✅ done!");
        println!();
        println!("☕ Break over! Ready for another pomodoro?");
        Ok(())
    }

    /// Prompt user yes/no for a break.
    /// Returns true if the user wants a break (default yes).
    pub fn prompt_break(break_min: u64) -> bool {
        print!("Take a {}min break? [Y/n] ", break_min);
        io::stdout().flush().ok();
        let mut input = String::new();
        if io::stdin().lock().read_line(&mut input).is_err() {
            return true;
        }
        let answer = input.trim().to_lowercase();
        answer.is_empty() || answer == "y" || answer == "yes"
    }
}
