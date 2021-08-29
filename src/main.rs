#![allow(unused_must_use)]
#![allow(unused_variables)]

mod meilisearch;
mod options;

use anyhow::Result;
use options::{Command, Options};
use reqwest::blocking::Response;
use serde_json::Value;
use structopt::StructOpt;
use termion::color;

use crate::meilisearch::Meilisearch;

type DocId = u32;
type UpdateId = u32;
type DumpId = String;

fn main() -> Result<()> {
    let opt = Options::from_args();
    let meili = Meilisearch::from(&opt);

    match opt.command {
        Command::Get { document_id: None } => meili.get_all_documents()?,
        Command::Get { document_id: Some(id) } => meili.get_one_document(id)?,
        Command::Add {
            content_type,
            r#async,
            file,
        } => {
            meili.index_documents(file, content_type, false)?;
        }
        Command::Update {
            content_type,
            r#async,
            file,
        } => {
            meili.index_documents(file, content_type, true)?;
        }
        Command::Delete {
            document_ids,
            r#async,
        } => match document_ids.as_slice() {
            [] => meili.delete_all()?,
            [id] => meili.delete_one(*id)?,
            ids => meili.delete_batch(ids)?,
        },
        Command::Dump { r#async, dump_id: None } => meili.create_dump()?,
        Command::Dump { r#async, dump_id: Some(dump_id) } => meili.dump_status(dump_id)?,
        Command::Health => meili.healthcheck()?,
        Command::Version => meili.version()?,
        Command::Stats => meili.stats()?,
        Command::Status { update_id } => meili.status(update_id)?,
        Command::Search { message, all } => todo!(),
    }

    Ok(())
}

fn handle_response(response: Response) -> Result<()> {
    print!("{}", color::Fg(color::Cyan));
    for (key, value) in response.headers() {
        println!("{}: {:?}", key, value);
    }
    println!("{}", color::Fg(color::Reset));
    println!(
        "{}",
        serde_json::to_string_pretty(&response.json::<Value>()?)?
    );
    Ok(())
}
