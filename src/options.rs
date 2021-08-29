use std::path::PathBuf;

use structopt::*;

use crate::{DocId, UpdateId};

#[derive(Debug, StructOpt)]
#[structopt(about = "A stupid wrapper around meilisearch")]
pub struct Options {
    /// The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700)
    #[structopt(
        short,
        long,
        default_value = "http://localhost:7700",
        env = "MEILI_ADDR"
    )]
    pub addr: String,

    /// The name of the index
    #[structopt(short, long, default_value = "mieli")]
    pub index: String,

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
        #[structopt(short, long)]
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
        #[structopt(short, long)]
        r#async: bool,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Delete documents. If no argument are specified all documents are deleted.
    Delete {
        /// The list of document ids you want to delete
        document_ids: Vec<DocId>,
        /// The command will exit immediatly after sending the documents ids
        #[structopt(short, long)]
        r#async: bool,
    },
    /// Create a dump or get the status of a dump
    Dump {
        /// The command will exit immediatly after asking for a dump
        #[structopt(short, long)]
        r#async: bool,
        /// The dump you want info from
        dump_id: Option<String>,
    },
    /// Return the status of an update
    Status { update_id: UpdateId },
    /// Do an healthcheck
    Health,
    /// Return the version of the running meilisearch instance
    Version,
    /// Return the stats about the indexes
    Stats,
    /// Do a search
    Search {
        #[structopt(short)]
        message: Option<String>,
        #[structopt(short)]
        all: bool,
    },
}
