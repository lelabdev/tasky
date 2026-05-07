use anyhow::Result;

/// Pomodoro timer state
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

    /// Start a pomodoro timer with terminal animation
    pub fn start(&self) -> Result<()> {
        anyhow::bail!("not yet implemented: pomodoro timer")
    }

    /// Stop the current pomodoro
    pub fn stop(&self) -> Result<()> {
        anyhow::bail!("not yet implemented: pomodoro stop")
    }

    /// Show current pomodoro status
    pub fn status(&self) -> Result<()> {
        anyhow::bail!("not yet implemented: pomodoro status")
    }
}
