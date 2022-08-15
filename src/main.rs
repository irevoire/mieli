#![doc = include_str!("../README.md")]

mod documents;
mod format;
mod indexes;
mod inner;
mod interactive_search;
mod keys;
mod meilisearch;
mod options;

pub use crate::documents::DocumentsCommand;
pub use crate::indexes::IndexesCommand;
pub use crate::keys::KeyCommand;
pub use crate::meilisearch::Meilisearch;
pub use crate::options::{Command, InnerCommand, Options};

use clap::Parser;
use inner::auto_complete;
use miette::Result;

type UpdateId = u32;
type TaskId = u32;
type DumpId = String;

fn main() -> Result<()> {
    let opt = Options::from_args();
    let meili = opt.meilisearch;

    match opt.command {
        Command::Inner { command } => match command {
            InnerCommand::AutoComplete { shell } => auto_complete(shell),
        },
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
        Command::Dump { dump_id: None } => meili.create_dump(),
        Command::Dump {
            dump_id: Some(dump_id),
        } => meili.dump_status(dump_id),
        Command::Health => meili.healthcheck(),
        Command::Version => meili.version(),
        Command::Stats => meili.stats(),
        Command::Status { update_id } => meili.status(update_id),
        Command::Tasks {
            task_id,
            task_filter,
        } => meili.tasks(task_id, task_filter),
        Command::Key(command) => command.execute(meili),
    }
}
