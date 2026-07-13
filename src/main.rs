use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod git;

#[derive(Parser)]
#[command(name = "rit")]
#[command(about = "A tiny Git implementation written in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init,

    HashObject {
        path: String,
    },

    CatFile {
        hash: String,
    },

    WriteTree,

    Commit {
        #[arg(short, long)]
        message: String,
    },

    Log,

    Checkout {
        hash: String,
    },

    Status,

    Add {
        path:String,
    },

    Branch {
        name: Option<String>,
    },

    Diff {
        #[arg(long)]
        cached: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

match cli.command {
    Command::Init => commands::init::run(),
    Command::HashObject { path } => commands::hash_object::run(&path),
    Command::CatFile { hash } => commands::cat_file::run(&hash),
    Command::WriteTree => commands::write_tree::run(),
    Command::Commit { message } => commands::commit::run(&message),
    Command::Log => commands::log::run(),
    Command::Checkout { hash } => commands::checkout::run(&hash),
    Command::Status => commands::status::run(),
    Command::Add { path } => commands::add::run(&path),
    Command::Branch { name } => commands::branch::run(name.as_deref()),
    Command::Diff { cached } => commands::diff::run(cached),}

}