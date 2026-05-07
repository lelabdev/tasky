use anyhow::Result;

use super::PomodoroArgs;

pub fn run(args: PomodoroArgs) -> Result<()> {
    match args.command {
        super::PomodoroCommands::Start { task: _ } => {
            anyhow::bail!("not yet implemented: tasky pomodoro start")
        }
        super::PomodoroCommands::Stop => {
            anyhow::bail!("not yet implemented: tasky pomodoro stop")
        }
        super::PomodoroCommands::Status => {
            anyhow::bail!("not yet implemented: tasky pomodoro status")
        }
        super::PomodoroCommands::Configure => {
            anyhow::bail!("not yet implemented: tasky pomodoro configure")
        }
    }
}
