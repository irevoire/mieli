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

type UpdateId = u32;
type TaskId = u32;
type DumpId = String;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Options::from_args();
    let meili = opt.meilisearch;

    match opt.command {
        Command::Inner(command) => command.execute().await,
        Command::Documents(command) => command.execute(meili).await,
        Command::Search {
            search_terms,
            interactive: false,
        } => meili.search(search_terms.join(" ")).await,
        Command::Search {
            search_terms,
            interactive: true,
        } => meili.interactive_search(search_terms.join(" ")).await,
        Command::Settings => meili.settings().await,
        Command::Index(command) => command.execute(meili).await,
        Command::Dump { dump_id: None } => meili.create_dump().await,
        Command::Dump {
            dump_id: Some(dump_id),
        } => meili.dump_status(dump_id).await,
        Command::Health => meili.healthcheck().await,
        Command::Version => meili.version().await,
        Command::Stats => meili.stats().await,
        Command::Status { update_id } => meili.status(update_id).await,
        Command::Tasks {
            task_id,
            task_filter,
        } => meili.tasks(task_id, task_filter).await,
        Command::Key(command) => command.execute(meili).await,
    }
}
