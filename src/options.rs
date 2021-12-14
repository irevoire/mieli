use std::path::PathBuf;

use structopt::*;

use crate::{DocId, TaskId, UpdateId};

#[derive(Debug, StructOpt)]
#[structopt(about = "A stupid wrapper around meilisearch")]
pub struct Options {
    /// Verbose mode (-v, -vv, etc)
    #[structopt(global = true, short, parse(from_occurrences))]
    pub verbose: usize,

    /// The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700)
    #[structopt(
        global = true,
        short,
        long,
        default_value = "http://localhost:7700",
        env = "MEILI_ADDR"
    )]
    pub addr: String,

    /// The name of the index
    #[structopt(
        global = true,
        short,
        long,
        default_value = "mieli",
        env = "MIELI_INDEX"
    )]
    pub index: String,

    /// Your secret API key <https://docs.meilisearch.com/reference/api/keys.html#get-keys>
    #[structopt(global = true, short, long, env = "MEILI_MASTER_KEY")]
    pub key: Option<String>,

    /// Use a specific http User-Agent for your request
    #[structopt(global = true, long)]
    pub user_agent: Option<String>,

    /// Interval between each status check (in milliseconds)
    #[structopt(global = true, long, default_value = "200")]
    pub interval: usize,

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
        /// The command will exit immediatly after sending the documents
        #[structopt(long)]
        r#async: bool,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Replace documents with the `put` verb
    /// You can pipe your documents in the command
    Update {
        /// Set the content-type of your file
        #[structopt(short, default_value = "application/json")]
        content_type: String,
        /// The command will exit immediatly after sending the documents
        #[structopt(long)]
        r#async: bool,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Delete documents. If no argument are specified all documents are deleted.
    Delete {
        /// The list of document ids you want to delete
        document_ids: Vec<DocId>,
        /// The command will exit immediatly after sending the documents ids
        #[structopt(long)]
        r#async: bool,
    },
    /// Create a dump or get the status of a dump
    Dump {
        /// The command will exit immediatly after asking for a dump
        #[structopt(long)]
        r#async: bool,
        /// The dump you want info from
        dump_id: Option<String>,
    },
    /// Return the status updates
    Status {
        /// The update id you want the status of
        update_id: Option<UpdateId>,
        /// If the flag is set, the command will wait until the update finishes
        #[structopt(short, long)]
        watch: bool,
    },
    /// Get information about the tasks.
    Task {
        /// The task you want to inspect.
        task_id: Option<TaskId>,
        /// If the flag is set, the command will wait until the update finishes
        #[structopt(short, long)]
        watch: bool,
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

        /// If you want to use the interactive search. It's a beta feature
        #[structopt(long)]
        interactive: bool,
    },
    /// Get or update the settings.
    /// You can pipe your settings in the command.
    #[structopt(aliases = &["set", "setting"])]
    Settings {
        /// The command will exit immediatly after sending the new settings
        #[structopt(long)]
        r#async: bool,
    },
    /// Manipulate indexes, add `--help` to see all the subcommands.
    #[structopt(aliases = &["indexes"])]
    Index {
        #[structopt(subcommand)]
        command: IndexesCommand,
    },
    /// Get the keys
    #[structopt(aliases = &["keys"])]
    Key,
}

#[derive(Debug, StructOpt)]
pub enum IndexesCommand {
    /// List all indexes.
    All,
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
