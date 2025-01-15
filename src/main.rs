#![doc = include_str!("../README.md")]

mod batches;
mod documents;
mod experimental;
mod format;
mod indexes;
mod inner;
mod interactive_search;
mod keys;
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

fn main() -> Result<()> {
    let opt = Options::parse();
    let meili = opt.meilisearch;
    env_logger::Builder::from_env(Env::default().default_filter_or("mieli=debug")).init();

    match opt.command {
        Command::Inner(command) => command.execute(),
        Command::Documents(command) => command.execute(meili),
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
        Command::Batches(command) => command.execute(meili),
        Command::Key(command) => command.execute(meili),
        Command::Experimental(command) => command.execute(meili),
    }
}
