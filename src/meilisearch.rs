use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

use crate::{
    format::{write_json, write_response_full},
    DocId, DumpId, Options, UpdateId,
};
use anyhow::Result;
use indicatif::ProgressBar;
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    header::USER_AGENT,
};
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Meilisearch {
    pub addr: String,
    pub index: String,
    pub key: Option<String>,
    pub interval: usize,
    pub r#async: bool,
    pub user_agent: String,
}

impl From<&Options> for Meilisearch {
    fn from(options: &Options) -> Self {
        Self {
            addr: options.addr.clone(),
            index: options.index.clone(),
            key: options.key.clone(),
            interval: options.interval,
            r#async: true,
            user_agent: options
                .user_agent
                .clone()
                .unwrap_or_else(|| format!("mieli/{}", env!("CARGO_PKG_VERSION"))),
        }
    }
}

impl Meilisearch {
    pub fn r#async(self, r#async: bool) -> Self {
        Self { r#async, ..self }
    }

    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.get(url.as_ref()))
    }

    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.post(url.as_ref()))
    }

    pub fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.get(url.as_ref()))
    }

    pub fn delete(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.delete(url.as_ref()))
    }

    fn request(
        &self,
        closure: impl Fn(Client) -> RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        let req_builder = closure(Client::new());
        if let Some(ref key) = self.key {
            req_builder
                .header("X-Meili-API-Key", key)
                .header(USER_AGENT, &self.user_agent)
        } else {
            req_builder.header(USER_AGENT, &self.user_agent)
        }
    }

    pub fn get_one_document(&self, docid: DocId) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn get_all_documents(&self) -> Result<()> {
        // TODO: we should cycle to get ALL the documents
        let response = self
            .get(&format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn index_documents(
        &self,
        filepath: Option<PathBuf>,
        content_type: String,
        reindex: bool,
    ) -> Result<()> {
        let url = format!("{}/indexes/{}/documents", self.addr, self.index);
        let client = match reindex {
            false => self.post(url),
            true => self.put(url),
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
        self.handle_response(response)?;
        Ok(())
    }

    pub fn delete_all(&self) -> Result<()> {
        let response = self
            .delete(format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn delete_one(&self, docid: DocId) -> Result<()> {
        let response = self
            .delete(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn delete_batch(&self, docids: &[DocId]) -> Result<()> {
        let response = self
            .post(format!(
                "{}/indexes/{}/documents/delete-batch",
                self.addr, self.index
            ))
            .json(docids)
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn search(&self) -> Result<()> {
        if atty::is(atty::Stream::Stdin) {
            self.interactive_search()?;
        } else {
            let mut buffer = Vec::new();
            stdin().read_to_end(&mut buffer);

            let response = self
                .post(format!("{}/indexes/{}/search", self.addr, self.index))
                .header("Content-Type", "application/json")
                .body(buffer)
                .send()?;

            self.handle_response(response)?;
        }
        Ok(())
    }

    pub fn settings(&self) -> Result<()> {
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer);

        let response = self
            .post(format!("{}/indexes/{}/settings", self.addr, self.index))
            .header("Content-Type", "application/json")
            .body(buffer)
            .send()?;

        self.handle_response(response)
    }

    pub fn status(&self, uid: UpdateId) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/updates/{}",
                self.addr, self.index, uid
            ))
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn create_dump(&self) -> Result<()> {
        let response = self.post(format!("{}/dumps", self.addr)).send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn dump_status(&self, dump_id: DumpId) -> Result<()> {
        let response = self
            .get(format!("{}/dumps/{}/status", self.addr, dump_id))
            .send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn healthcheck(&self) -> Result<()> {
        let response = self.get(format!("{}/health", self.addr)).send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn version(&self) -> Result<()> {
        let response = self.get(format!("{}/version", self.addr)).send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn stats(&self) -> Result<()> {
        let response = self.get(format!("{}/stats", self.addr)).send()?;
        self.handle_response(response)?;
        Ok(())
    }

    pub fn handle_response(&self, response: Response) -> Result<()> {
        let response = write_response_full(response)?;
        if self.r#async {
            return Ok(());
        }

        let spinner = ProgressBar::new_spinner();

        let buffer = String::new();

        if let Some(uid) = response["updateId"].as_i64() {
            loop {
                let response = self
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
                        write_json(json);
                        break;
                    }
                    Some(status) => spinner.set_message(status.to_string()),
                }
                std::thread::sleep(std::time::Duration::from_millis(self.interval as u64));
            }
        }
        Ok(())
    }
}
