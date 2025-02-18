#![doc = include_str!("../README.md")]

mod batches;
mod documents;
mod experimental;
mod format;
mod indexes;
mod inner;
mod interactive_search;
mod keys;
mod log;
mod meilisearch;
mod options;
mod tasks;

pub use crate::documents::DocumentsCommand;
pub use crate::indexes::IndexesCommand;
pub use crate::inner::Inner;
pub use crate::keys::Key;
pub use crate::meilisearch::Meilisearch;
pub use crate::options::{Command, Options};

use clap::Parser;
use env_logger::Env;
use miette::Result;
use tasks::TasksCommand;

fn main() -> Result<()> {
    let opt = Options::parse();
    let meili = opt.meilisearch;
    let log_levels = [
        "mieli=info",
        "mieli=debug",
        "mieli=trace",
        "debug,mieli=trace",
        "trace",
    ];
    let log_level = log_levels[(meili.verbose as usize).clamp(0, log_levels.len() - 1)];
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();
    ::log::trace!("verbosity selected: {log_level}");

    match opt.command {
        Command::Inner(command) => command.execute(),
        Command::Documents(command) => command.execute(meili),
        Command::Da(params) => DocumentsCommand::Add(params).execute(meili),
        Command::Dd { ids, filter } => DocumentsCommand::Delete { ids, filter }.execute(meili),
        Command::Search {
            search_terms,
            interactive: false,
        } => meili.search(search_terms.join(" ")),
        Command::Search {
            search_terms,
            interactive: true,
        } => meili.interactive_search(search_terms.join(" ")),
        Command::Settings => meili.settings(),
        Command::Index(command) => command.execute(meili),
        Command::Dump => meili.create_dump(),
        Command::Snapshot => meili.create_snapshot(),
        Command::Health => meili.healthcheck(),
        Command::Version => meili.version(),
        Command::Stats => meili.stats(),
        Command::Tasks(command) => command.execute(meili),
        Command::Tl { params, id } => TasksCommand::List { params, id }.execute(meili),
        Command::Batches(command) => command.execute(meili),
        Command::Key(command) => command.execute(meili),
        Command::Log(command) => command.execute(meili),
        Command::Experimental(command) => command.execute(meili),
    }
}
