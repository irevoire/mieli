use std::{
    fs::File,
    io::{stdin, Read, Write},
    path::PathBuf,
};

use crate::{
    format::{write_json, write_response_full},
    DocId, DumpId, Options, UpdateId,
};
use anyhow::Result;
use indicatif::ProgressBar;
use reqwest::blocking::{Client, Response};
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Meilisearch {
    addr: String,
    index: String,
    interval: usize,
    r#async: bool,
}

impl From<&Options> for Meilisearch {
    fn from(options: &Options) -> Self {
        Self {
            addr: options.addr.clone(),
            index: options.index.clone(),
            interval: options.interval,
            r#async: true,
        }
    }
}

impl Meilisearch {
    pub fn r#async(self, r#async: bool) -> Self {
        Self { r#async, ..self }
    }

    pub fn get_one_document(&self, output: &mut dyn Write, docid: DocId) -> Result<()> {
        let response = Client::new()
            .get(&format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn get_all_documents(&self, output: &mut dyn Write) -> Result<()> {
        // TODO: we should cycle to get ALL the documents
        let response = Client::new()
            .get(&format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn index_documents(
        &self,
        output: &mut dyn Write,
        filepath: Option<PathBuf>,
        content_type: String,
        reindex: bool,
    ) -> Result<()> {
        let url = format!("{}/indexes/{}/documents", self.addr, self.index);
        let client = match reindex {
            false => Client::new().post(url),
            true => Client::new().put(url),
        };

        let response = match filepath {
            Some(filepath) => {
                let file = File::open(filepath)?;
                client
                    .header("Content-Type", content_type)
                    .body(file)
                    .send()?
            }
            None => {
                // TODO: is this the only way to do it?
                let mut buffer = Vec::new();
                stdin().read_to_end(&mut buffer);

                client
                    .header("Content-Type", content_type)
                    .body(buffer)
                    .send()?
            }
        };
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn delete_all(&self, output: &mut dyn Write) -> Result<()> {
        let response = Client::new()
            .delete(format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn delete_one(&self, output: &mut dyn Write, docid: DocId) -> Result<()> {
        let response = Client::new()
            .delete(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn delete_batch(&self, output: &mut dyn Write, docids: &[DocId]) -> Result<()> {
        let response = Client::new()
            .post(format!(
                "{}/indexes/{}/documents/delete-batch",
                self.addr, self.index
            ))
            .json(docids)
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn search(&self, output: &mut dyn Write) -> Result<()> {
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer);

        let response = Client::new()
            .post(format!("{}/indexes/{}/search", self.addr, self.index))
            .header("Content-Type", "application/json")
            .body(buffer)
            .send()?;

        self.handle_response(output, response)
    }

    pub fn settings(&self, output: &mut dyn Write) -> Result<()> {
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer);

        let response = Client::new()
            .post(format!("{}/indexes/{}/settings", self.addr, self.index))
            .header("Content-Type", "application/json")
            .body(buffer)
            .send()?;

        self.handle_response(output, response)
    }

    pub fn status(&self, output: &mut dyn Write, uid: UpdateId) -> Result<()> {
        let response = Client::new()
            .get(format!(
                "{}/indexes/{}/updates/{}",
                self.addr, self.index, uid
            ))
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn create_dump(&self, output: &mut dyn Write) -> Result<()> {
        let response = Client::new().post(format!("{}/dumps", self.addr)).send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn dump_status(&self, output: &mut dyn Write, dump_id: DumpId) -> Result<()> {
        let response = Client::new()
            .get(format!("{}/dumps/{}/status", self.addr, dump_id))
            .send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn healthcheck(&self, output: &mut dyn Write) -> Result<()> {
        let response = Client::new().get(format!("{}/health", self.addr)).send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn version(&self, output: &mut dyn Write) -> Result<()> {
        let response = Client::new().get(format!("{}/version", self.addr)).send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn stats(&self, output: &mut dyn Write) -> Result<()> {
        let response = Client::new().get(format!("{}/stats", self.addr)).send()?;
        self.handle_response(output, response)?;
        Ok(())
    }

    pub fn handle_response(&self, output: &mut dyn Write, response: Response) -> Result<()> {
        let response = write_response_full(output, response)?;
        if self.r#async {
            return Ok(());
        }

        let spinner = ProgressBar::new_spinner();

        let buffer = String::new();

        if let Some(uid) = response["updateId"].as_i64() {
            loop {
                let response = Client::new()
                    .get(format!(
                        "{}/indexes/{}/updates/{}",
                        self.addr, self.index, uid
                    ))
                    .send()?;
                let json = response.json::<Value>()?;
                match json["status"].as_str() {
                    None => {
                        return Ok(());
                    }
                    Some(msg @ "processed") | Some(msg @ "failed") => {
                        spinner.finish_with_message(msg.to_string());
                        write_json(output, json);
                        break;
                    }
                    Some(status) => spinner.set_message(status.to_string()),
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
        Ok(())
    }
}
