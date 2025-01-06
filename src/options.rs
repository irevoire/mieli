use clap::Parser;

use crate::{
    batches::BatchesCommand, inner::Inner, meilisearch::Meilisearch, tasks::TasksCommand,
    DocumentsCommand, IndexesCommand, Key,
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
    /// Get information on the task queue
    #[clap(subcommand, aliases = &["task", "t"])]
    Tasks(TasksCommand),
    /// Get information about the batches
    #[clap(subcommand, aliases = &["batch", "b"])]
    Batches(BatchesCommand),
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
