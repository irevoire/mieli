use std::path::PathBuf;

use structopt::*;

use crate::{DocId, UpdateId};

#[derive(Debug, StructOpt)]
#[structopt(about = "A stupid wrapper around meilisearch")]
pub struct Options {
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
    #[structopt(
        global = true,
        short,
        long,
        default_value = "",
        env = "MEILI_MASTER_KEY"
    )]
    pub key: String,

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
    /// Return the status of an update
    Status {
        /// The update id
        update_id: UpdateId,
        /// If the flag is set, the command will wait until the update finishes
        #[structopt(short, long)]
        watch: bool,
    },
    /// Do an healthcheck
    Health,
    /// Return the version of the running meilisearch instance
    Version,
    /// Return the stats about the indexes
    Stats,
    /// Do a search. You must pipe your parameter in the command as a json
    Search {
        #[structopt(long)]
        all: bool,
    },
    /// Update the settings. You must pipe your parameter in the command as a json.
    Settings {
        /// The command will exit immediatly after asking for a dump
        #[structopt(long)]
        r#async: bool,
    },
}
