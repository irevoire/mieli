use clap::Parser;
use serde::Serialize;

use crate::{
    inner::Inner, meilisearch::Meilisearch, DocumentsCommand, IndexesCommand, Key, TaskId,
};

#[derive(Debug, Parser)]
#[clap(about = "A stupid wrapper around meilisearch")]
pub struct Options {
    #[clap(flatten)]
    pub meilisearch: Meilisearch,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Modify the `mieli` installation.
    #[clap(subcommand, name = "self")]
    Inner(Inner),
    /// Manipulate indexes, add `--help` to see all the subcommands.
    #[clap(subcommand, aliases = &["indexes", "i"])]
    Index(IndexesCommand),
    /// Manipulate documents, add `--help` to see all the subcommands.
    #[clap(subcommand, aliases = &["document", "doc", "docs", "d"])]
    Documents(DocumentsCommand),
    /// Create a dump
    Dump,
    /// Get information about the task of an index.
    #[clap(aliases = &["task", "t"])]
    Tasks {
        /// The task you want to inspect.
        task_id: Option<TaskId>,
        /// The task filters you want to apply.
        #[clap(flatten)]
        task_filter: TasksFilter,
    },
    /// Do an healthcheck
    Health,
    /// Return the version of the running meilisearch instance
    #[clap(aliases = &["ver", "v"])]
    Version,
    /// Return the stats about the indexes
    #[clap(aliases = &["stat"])]
    Stats,
    /// Do a search. You can pipe your parameter in the command as a json.
    /// Or you can specify directly what you want to search in the arguments.
    Search {
        /// What you want to search. If nothing was piped in the command a simple request with only `q` will be ran.
        /// If you piped some configuration the `q` parameter will be replaced with the one specified in the arguments.
        search_terms: Vec<String>,

        /// If you want to use the interactive search. It's a beta feature
        #[clap(long)]
        interactive: bool,
    },
    /// Get or update the settings.
    /// You can pipe your settings in the command.
    #[clap(aliases = &["set", "setting"])]
    Settings,
    /// Get the keys
    #[clap(subcommand, aliases = &["keys", "k"])]
    Key(Key),
}

#[derive(Debug, Parser, Serialize)]
pub struct TasksFilter {
    /// Number of tasks to return.
    #[clap(long)]
    limit: Option<usize>,
    /// Task id of the first task returned.
    #[clap(long)]
    from: Option<usize>,
    /// Filter tasks by their status.
    #[clap(long, aliases = &["status"])]
    statuses: Option<String>,
    /// Filter tasks by their type.
    #[clap(long, aliases = &["ty", "type"])]
    types: Option<String>,
    /// Filter tasks by their index uid.
    #[clap(long)]
    #[serde(rename = "indexUids")]
    indexes: Option<String>,
    /// Filter tasks by their uids.
    #[clap(long, aliases = &["uid"])]
    uids: Option<String>,
}
