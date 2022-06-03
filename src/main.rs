#![doc = include_str!("../README.md")]
#![allow(unused_must_use)]
#![allow(unused_variables)]

mod format;
mod interactive_search;
mod meilisearch;
mod options;

use anyhow::Result;
use options::{Command, KeyCommand, Options};

use structopt::StructOpt;

use crate::options::IndexesCommand;

type DocId = u32;
type UpdateId = u32;
type TaskId = u32;
type DumpId = String;

fn main() -> Result<()> {
    let opt = Options::from_args();
    let meili = opt.meilisearch;

    match opt.command {
        Command::Get { document_id: None } => meili.get_all_documents()?,
        Command::Get {
            document_id: Some(id),
        } => meili.get_one_document(id)?,
        Command::Add {
            content_type,
            file,
            primary,
        } => {
            meili.index_documents(file, primary, content_type, false)?;
        }
        Command::Update {
            content_type,
            file,
            primary,
        } => {
            meili.index_documents(file, primary, content_type, true)?;
        }
        Command::Delete { document_ids } => match document_ids.as_slice() {
            [] => meili.delete_all()?,
            [id] => meili.delete_one(*id)?,
            ids => meili.delete_batch(ids)?,
        },
        Command::Search {
            search_terms,
            interactive: false,
        } => meili.search(search_terms.join(" "))?,
        Command::Search {
            search_terms,
            interactive: true,
        } => meili.interactive_search(search_terms.join(" "))?,
        Command::Settings => meili.settings()?,
        Command::Index { command } => match command {
            IndexesCommand::List => meili.get_all_indexes()?,
            IndexesCommand::Get { index } => meili.get_index(index)?,
            IndexesCommand::Create { index, primary } => meili.create_index(index, primary)?,
            IndexesCommand::Update { index, primary } => meili.update_index(index, primary)?,
            IndexesCommand::Delete { index } => meili.delete_index(index)?,
        },
        Command::Dump { dump_id: None } => meili.create_dump()?,
        Command::Dump {
            dump_id: Some(dump_id),
        } => meili.dump_status(dump_id)?,
        Command::Health => meili.healthcheck()?,
        Command::Version => meili.version()?,
        Command::Stats => meili.stats()?,
        Command::Status { update_id } => meili.status(update_id)?,
        Command::Task { task_id, all: true } => meili.global_task(task_id)?,
        Command::Task {
            task_id,
            all: false,
        } => meili.task_by_index(task_id)?,
        Command::Key { command } => match command {
            KeyCommand::List => meili.get_keys()?,
            KeyCommand::Get { k } => meili.get_key(k)?,
            KeyCommand::Create => meili.create_key()?,
            KeyCommand::Update { k } => meili.update_key(k)?,
            KeyCommand::Delete { k } => meili.delete_key(k)?,
            KeyCommand::Template => meili.template()?,
        },
    }

    Ok(())
}
