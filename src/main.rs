#![allow(unused_must_use)]
#![allow(unused_variables)]

mod format;
mod meilisearch;
mod options;

use std::io::stdout;

use anyhow::Result;
use nom::branch::alt;
use nom::character::complete::alphanumeric1;
use nom::sequence::delimited;
use options::{Command, Options};

use serde_json::Value;
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
        Command::Search { arguments, all } => meili.search(cli_to_json(arguments)?, stdout)?,
        Command::Settings { arguments, r#async } => meili
            .r#async(r#async)
            .settings(cli_to_json(arguments)?, stdout)?,
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

use nom::bytes::complete::{tag, take, take_while1};
use nom::character::is_alphanumeric;
use nom::IResult;

fn value(i: &str) -> IResult<&str, &str> {
    alt((
        alphanumeric1,
        delimited(tag("\""), alphanumeric1, tag("\"")),
        delimited(tag("'"), alphanumeric1, tag("'")),
    ))(i)
}

fn cli_to_json(arguments: Vec<String>) -> anyhow::Result<Option<Value>> {
    let arguments = arguments.join(" ");
    dbg!(arguments);
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nom_value() {
        let result = value("hello").finish();
        assert_eq!();
    }
}

