use anyhow::Result;
use clap::Parser;

mod cmd;
mod config;
mod pomodoro;
mod storage;
mod task;
mod utils;

use cmd::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd::init::run(),
        Commands::New(args) => cmd::new_cmd::run(args),
        Commands::List(args) => cmd::list_cmd::run(args),
        Commands::Start(args) => cmd::start_cmd::run(args),
        Commands::Stop => cmd::stop_cmd::run(),
        Commands::Done(args) => cmd::done_cmd::run(args),
        Commands::Finish => cmd::finish_cmd::run(),
        Commands::Link(args) => cmd::link_cmd::run(args),
        Commands::Day => cmd::day_cmd::run(),
        Commands::Week => cmd::week_cmd::run(),
        Commands::Pomodoro(args) => cmd::pomodoro_cmd::run(args),
    }
}
