#![doc = include_str!("../README.md")]
#![allow(unused_must_use)]
#![allow(unused_variables)]

mod format;
mod interactive_search;
mod meilisearch;
mod options;

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

    match opt.command {
        Command::Get { document_id: None } => meili.get_all_documents()?,
        Command::Get {
            document_id: Some(id),
        } => meili.get_one_document(id)?,
        Command::Add {
            content_type,
            file,
            r#async,
        } => {
            meili
                .r#async(r#async)
                .index_documents(file, content_type, false)?;
        }
        Command::Update {
            content_type,
            file,
            r#async,
        } => {
            meili
                .r#async(r#async)
                .index_documents(file, content_type, true)?;
        }
        Command::Delete {
            document_ids,
            r#async,
        } => {
            let meili = meili.r#async(r#async);
            match document_ids.as_slice() {
                [] => meili.delete_all()?,
                [id] => meili.delete_one(*id)?,
                ids => meili.delete_batch(ids)?,
            }
        }
        Command::Search {
            search_terms,
            interactive: false,
        } => meili.search(search_terms.join(" "))?,
        Command::Search {
            search_terms,
            interactive: true,
        } => meili.interactive_search(search_terms.join(" "))?,
        Command::Settings { r#async } => meili.r#async(r#async).settings()?,
        Command::Dump {
            r#async,
            dump_id: None,
        } => meili.r#async(r#async).create_dump()?,
        Command::Dump {
            r#async,
            dump_id: Some(dump_id),
        } => meili.dump_status(dump_id)?,
        Command::Health => meili.healthcheck()?,
        Command::Version => meili.version()?,
        Command::Stats => meili.stats()?,
        Command::Status { update_id, watch } => meili.r#async(!watch).status(update_id)?,
    }

    Ok(())
}
