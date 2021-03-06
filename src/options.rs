use std::path::PathBuf;

use structopt::*;

use crate::{meilisearch::Meilisearch, DocId, TaskId, UpdateId};

#[derive(Debug, StructOpt)]
#[structopt(about = "A stupid wrapper around meilisearch")]
pub struct Options {
    #[structopt(flatten)]
    pub meilisearch: Meilisearch,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Get one document. If no argument are specified it returns all documents.
    Get {
        /// The id of the document you want to retrieve
        document_id: Option<DocId>,
    },
    /// Add documents with the `post` verb
    /// You can pipe your documents in the command
    Add {
        /// Set the content-type of your file
        #[structopt(short, default_value = "application/json")]
        content_type: String,
        /// The primary key
        #[structopt(short, long)]
        primary: Option<String>,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Replace documents with the `put` verb
    /// You can pipe your documents in the command
    Update {
        /// Set the content-type of your file
        #[structopt(short, default_value = "application/json")]
        content_type: String,
        /// The primary key
        #[structopt(short, long)]
        primary: Option<String>,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Delete documents. If no argument are specified all documents are deleted.
    Delete {
        /// The list of document ids you want to delete
        document_ids: Vec<DocId>,
    },
    /// Create a dump or get the status of a dump
    Dump {
        /// The dump you want info from
        dump_id: Option<String>,
    },
    /// Return the status updates
    Status {
        /// The update id you want the status of
        update_id: Option<UpdateId>,
    },
    /// Get information about the task of an index.
    #[structopt(aliases = &["tasks"])]
    Task {
        /// The task you want to inspect.
        task_id: Option<TaskId>,

        /// If the flag is set, the command will look in all the tasks instead of the tasks by indexes.
        #[structopt(long)]
        all: bool,
    },
    /// Do an healthcheck
    Health,
    /// Return the version of the running meilisearch instance
    #[structopt(aliases = &["ver"])]
    Version,
    /// Return the stats about the indexes
    #[structopt(aliases = &["stat"])]
    Stats,
    /// Do a search. You can pipe your parameter in the command as a json.
    /// Or you can specify directly what you want to search in the arguments.
    Search {
        /// What you want to search. If nothing was piped in the command a simple request with only `q` will be ran.
        /// If you piped some configuration the `q` parameter will be replaced with the one specified in the arguments.
        search_terms: Vec<String>,

        /// If you want to use the interactive search.???It's a beta feature
        #[structopt(long)]
        interactive: bool,
    },
    /// Get or update the settings.
    /// You can pipe your settings in the command.
    #[structopt(aliases = &["set", "setting"])]
    Settings,
    /// Manipulate indexes, add `--help` to see all the subcommands.
    #[structopt(aliases = &["indexes"])]
    Index {
        #[structopt(subcommand)]
        command: IndexesCommand,
    },
    /// Get the keys
    #[structopt(aliases = &["keys"])]
    Key {
        #[structopt(subcommand)]
        command: KeyCommand,
    },
}

#[derive(Debug, StructOpt)]
pub enum IndexesCommand {
    /// List all indexes.
    #[structopt(aliases = &["all"])]
    List,
    /// Get an index, by default use the index provided by `-i`.
    Get {
        /// The index you want to retrieve.
        #[structopt(name = "idx")]
        index: Option<String>,
    },
    /// Create an index, by default use the index provided by `-i`.
    Create {
        /// The index you want to create.
        #[structopt(name = "idx")]
        index: Option<String>,
        /// Primary key
        #[structopt(short, long)]
        primary: Option<String>,
    },
    /// Update an index, by default use the index provided by `-i`.
    Update {
        /// The index you want to update.
        #[structopt(name = "idx")]
        index: Option<String>,
        /// Primary key
        #[structopt(short, long)]
        primary: Option<String>,
    },
    /// Delete an index, by default use the index provided by `-i`.
    Delete {
        /// The index you want to delete.
        #[structopt(name = "idx")]
        index: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
pub enum KeyCommand {
    /// List all keys.
    #[structopt(aliases = &["all"])]
    List,
    /// Get a key, by default use the key provided by `-k`.
    Get {
        /// The key you want to retrieve.
        k: Option<String>,
    },
    /// Create a key. The json needs to be piped in the command.
    #[structopt(aliases = &["post"])]
    Create,
    /// Update a key. The json needs to be piped in the command.
    #[structopt(aliases = &["patch"])]
    Update {
        /// The key you want to update. If you don't provide
        /// it here you need to send it in the json.
        k: Option<String>,
    },
    /// Delete a key.
    Delete {
        /// The key you want to delete.
        k: String,
    },
    /// Show an example of a valid json you can send to create a key.
    Template,
}
