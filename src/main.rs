#![doc = include_str!("../README.md")]
#![allow(unused_must_use)]
#![allow(unused_variables)]

mod format;
mod meilisearch;
mod options;

use std::io::stdout;

use anyhow::Result;
use options::{Command, Options};

use structopt::StructOpt;

use crate::meilisearch::Meilisearch;

type DocId = u32;
type UpdateId = u32;
type DumpId = String;

fn main() -> Result<()> {
    let opt = Options::from_args();
    let meili = Meilisearch::from(&opt);
    let stdout = &mut stdout();

    match opt.command {
        Command::Get { document_id: None } => meili.get_all_documents(stdout)?,
        Command::Get {
            document_id: Some(id),
        } => meili.get_one_document(stdout, id)?,
        Command::Add {
            content_type,
            file,
            r#async,
        } => {
            meili
                .r#async(r#async)
                .index_documents(stdout, file, content_type, false)?;
        }
        Command::Update {
            content_type,
            file,
            r#async,
        } => {
            meili
                .r#async(r#async)
                .index_documents(stdout, file, content_type, true)?;
        }
        Command::Delete {
            document_ids,
            r#async,
        } => {
            let meili = meili.r#async(r#async);
            match document_ids.as_slice() {
                [] => meili.delete_all(stdout)?,
                [id] => meili.delete_one(stdout, *id)?,
                ids => meili.delete_batch(stdout, ids)?,
            }
        }
        Command::Search { all } => meili.search(stdout)?,
        Command::Settings { r#async } => meili.r#async(r#async).settings(stdout)?,
        Command::Dump {
            r#async,
            dump_id: None,
        } => meili.r#async(r#async).create_dump(stdout)?,
        Command::Dump {
            r#async,
            dump_id: Some(dump_id),
        } => meili.dump_status(stdout, dump_id)?,
        Command::Health => meili.healthcheck(stdout)?,
        Command::Version => meili.version(stdout)?,
        Command::Stats => meili.stats(stdout)?,
        Command::Status { update_id, watch } => meili.r#async(!watch).status(stdout, update_id)?,
    }

    Ok(())
}
