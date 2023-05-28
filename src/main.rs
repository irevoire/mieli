#![doc = include_str!("../README.md")]

mod documents;
mod format;
mod indexes;
mod inner;
mod interactive_search;
mod keys;
mod meilisearch;
mod options;

pub use crate::documents::Documents;
pub use crate::indexes::IndexesCommand;
pub use crate::inner::Inner;
pub use crate::keys::Key;
pub use crate::meilisearch::Meilisearch;
pub use crate::options::{Command, Options};

use clap::Parser;
use miette::Result;

type TaskId = u32;

fn main() -> Result<()> {
    let opt = Options::from_args();
    let meili = opt.meilisearch;

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
        Command::Health => meili.healthcheck(),
        Command::Version => meili.version(),
        Command::Stats => meili.stats(),
        Command::Tasks {
            task_id,
            task_filter,
        } => meili.tasks(task_id, task_filter),
        Command::Key(command) => command.execute(meili),
    }
}
