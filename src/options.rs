use std::path::PathBuf;

use clap::Parser;
use serde::Serialize;

use crate::{meilisearch::Meilisearch, DocId, TaskId, UpdateId};

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
    #[clap(name = "self")]
    Inner {
        #[clap(subcommand)]
        command: InnerCommand,
    },
    /// Manipulate documents, add `--help` to see all the subcommands.
    #[clap(aliases = &["document", "doc", "docs", "d"])]
    Documents {
        #[clap(subcommand)]
        command: DocumentsCommand,
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
    /// Manipulate indexes, add `--help` to see all the subcommands.
    #[clap(aliases = &["indexes", "i"])]
    Index {
        #[clap(subcommand)]
        command: IndexesCommand,
    },
    /// Get the keys
    #[clap(aliases = &["keys", "k"])]
    Key {
        #[clap(subcommand)]
        command: KeyCommand,
    },
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
    #[clap(long)]
    status: Option<String>,
    /// Filter tasks by their type.
    #[clap(long, aliases = &["ty"])]
    r#type: Option<String>,
    /// Filter tasks by their index uid.
    #[clap(long, name = "uid")]
    uid: Option<String>,
}

#[derive(Debug, Parser, Serialize)]
pub struct GetDocumentParameter {
    /// Number of documents to return.
    #[clap(long, aliases = &["limits"])]
    limit: Option<usize>,
    /// Skip the n first documents.
    #[clap(long)]
    from: Option<usize>,
    /// Select fields from the documents.
    #[clap(long, aliases = &["field"])]
    fields: Option<String>,
}

#[derive(Debug, Parser)]
pub enum DocumentsCommand {
    /// Get one document. If no argument are specified it returns all documents.
    #[clap(aliases = &["g"])]
    Get {
        /// The id of the document you want to retrieve
        document_id: Option<DocId>,
        /// Query parameters.
        #[clap(flatten)]
        param: GetDocumentParameter,
    },
    /// Add documents with the `post` verb
    /// You can pipe your documents in the command
    /// Will try to infer the content-type from the file extension if it fail
    /// it'll be set as json.
    #[clap(aliases = &["a"])]
    Add {
        /// Set the content-type of your file.
        #[clap(short)]
        content_type: Option<String>,
        /// The primary key
        #[clap(short, long)]
        primary: Option<String>,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Replace documents with the `put` verb
    /// You can pipe your documents in the command
    /// Will try to infer the content-type from the file extension if it fail
    /// it'll be set as json.
    #[clap(aliases = &["u"])]
    Update {
        /// Set the content-type of your file
        #[clap(short)]
        content_type: Option<String>,
        /// The primary key
        #[clap(short, long)]
        primary: Option<String>,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Delete documents. If no argument are specified all documents are deleted.
    #[clap(aliases = &["d"])]
    Delete {
        /// The list of document ids you want to delete
        document_ids: Vec<DocId>,
    },
}

#[derive(Debug, Parser)]
pub enum IndexesCommand {
    /// List all indexes.
    #[clap(aliases = &["all"])]
    List,
    /// Get an index, by default use the index provided by `-i`.
    Get {
        /// The index you want to retrieve.
        #[clap(name = "idx")]
        index: Option<String>,
    },
    /// Create an index, by default use the index provided by `-i`.
    Create {
        /// The index you want to create.
        #[clap(name = "idx")]
        index: Option<String>,
        /// Primary key
        #[clap(short, long)]
        primary: Option<String>,
    },
    /// Update an index, by default use the index provided by `-i`.
    Update {
        /// The index you want to update.
        #[clap(name = "idx")]
        index: Option<String>,
        /// Primary key
        #[clap(short, long)]
        primary: Option<String>,
    },
    /// Delete an index, by default use the index provided by `-i`.
    Delete {
        /// The index you want to delete.
        #[clap(name = "idx")]
        index: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub enum KeyCommand {
    /// List all keys.
    #[clap(aliases = &["all"])]
    List,
    /// Get a key, by default use the key provided by `-k`.
    Get {
        /// The key you want to retrieve.
        k: Option<String>,
    },
    /// Create a key. The json needs to be piped in the command.
    #[clap(aliases = &["post"])]
    Create,
    /// Update a key. The json needs to be piped in the command.
    #[clap(aliases = &["patch"])]
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

#[derive(Debug, Parser)]
pub enum InnerCommand {
    /// Generate the autocomplete file for your shell.
    AutoComplete { shell: Option<String> },
}
