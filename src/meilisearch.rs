use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

use crate::{
    format::{write_json, write_response_full, write_response_headers},
    DocId, DumpId, Options, TaskId, UpdateId,
};
use anyhow::Result;
use indicatif::ProgressBar;
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use serde_json::{json, Map, Value};

#[derive(Debug, Default)]
pub struct Meilisearch {
    pub addr: String,
    pub index: String,
    pub key: Option<String>,
    pub interval: usize,
    pub r#async: bool,
    pub user_agent: String,
    pub verbose: usize,
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
            verbose: options.verbose,
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
        self.request(|c| c.put(url.as_ref()))
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
                .header(AUTHORIZATION, &format!("Bearer {}", key))
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
        self.handle_response(response)
    }

    pub fn get_all_documents(&self) -> Result<()> {
        // TODO: we should cycle to get ALL the documents
        let response = self
            .get(&format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        self.handle_response(response)
    }

    pub fn index_documents(
        &self,
        filepath: Option<PathBuf>,
        primary_key: Option<String>,
        content_type: String,
        reindex: bool,
    ) -> Result<()> {
        let url = format!("{}/indexes/{}/documents", self.addr, self.index);
        let client = match reindex {
            false => self.post(url),
            true => self.put(url),
        };
        let client = client.header(CONTENT_TYPE, content_type);
        let client = if let Some(primary_key) = primary_key {
            client.query(&[("primaryKey", primary_key)])
        } else {
            client
        };

        let response = match filepath {
            Some(filepath) => {
                let file = File::open(filepath)?;
                client.body(file).send()?
            }
            None => {
                // TODO: is this the only way to do it?
                let mut buffer = Vec::new();
                stdin().read_to_end(&mut buffer);

                client.body(buffer).send()?
            }
        };
        self.handle_response(response)
    }

    pub fn delete_all(&self) -> Result<()> {
        let response = self
            .delete(format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        self.handle_response(response)
    }

    pub fn delete_one(&self, docid: DocId) -> Result<()> {
        let response = self
            .delete(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()?;
        self.handle_response(response)
    }

    pub fn delete_batch(&self, docids: &[DocId]) -> Result<()> {
        let response = self
            .post(format!(
                "{}/indexes/{}/documents/delete-batch",
                self.addr, self.index
            ))
            .json(docids)
            .send()?;
        self.handle_response(response)
    }

    pub fn search(&self, search: String) -> Result<()> {
        let mut value: Map<String, Value> = if atty::isnt(atty::Stream::Stdin) {
            serde_json::from_reader(stdin())?
        } else {
            Map::new()
        };
        if !search.is_empty() {
            value.insert("q".to_string(), json!(search));
        }
        let response = self
            .post(format!("{}/indexes/{}/search", self.addr, self.index))
            .header("Content-Type", "application/json")
            .json(&value)
            .send()?;

        self.handle_response(response)
    }

    pub fn interactive_search(&self, search: String) -> Result<()> {
        if atty::isnt(atty::Stream::Stdout) {
            return self.search(search);
        }

        let mut value: Map<String, Value> = if atty::isnt(atty::Stream::Stdin) {
            serde_json::from_reader(stdin())?
        } else {
            Map::new()
        };
        if !search.is_empty() {
            value.insert("q".to_string(), json!(search));
        }

        self.run_interactive_search(search, value)
    }

    pub fn settings(&self) -> Result<()> {
        let response = if atty::is(atty::Stream::Stdin) {
            self.get(format!("{}/indexes/{}/settings", self.addr, self.index))
                .send()?
        } else {
            let mut buffer = Vec::new();
            stdin().read_to_end(&mut buffer);

            self.post(format!("{}/indexes/{}/settings", self.addr, self.index))
                .header("Content-Type", "application/json")
                .body(buffer)
                .send()?
        };

        self.handle_response(response)
    }

    pub fn get_all_indexes(&self) -> Result<()> {
        let response = self.get(format!("{}/indexes", self.addr)).send()?;
        self.handle_response(response)
    }

    pub fn get_index(&self, index: Option<String>) -> Result<()> {
        let index = index.unwrap_or(self.index.to_string());
        let response = self
            .get(format!("{}/indexes/{}", self.addr, index))
            .send()?;
        self.handle_response(response)
    }

    pub fn create_index(&self, index: Option<String>, primary_key: Option<String>) -> Result<()> {
        let index = index.unwrap_or(self.index.to_string());
        let mut body = json!({ "uid": index });
        if let Some(primary_key) = primary_key {
            body["primaryKey"] = json!(primary_key);
        }
        let response = self
            .post(format!("{}/indexes", self.addr))
            .json(&body)
            .send()?;
        self.handle_response(response)
    }

    pub fn update_index(&self, index: Option<String>, primary_key: Option<String>) -> Result<()> {
        let index = index.unwrap_or(self.index.to_string());
        let mut body = json!({});
        if let Some(primary_key) = primary_key {
            body["primaryKey"] = json!(primary_key);
        }
        let response = self
            .put(format!("{}/indexes/{}", self.addr, index))
            .json(&body)
            .send()?;
        self.handle_response(response)
    }

    pub fn delete_index(&self, index: Option<String>) -> Result<()> {
        let index = index.unwrap_or(self.index.to_string());
        let response = self
            .delete(format!("{}/indexes/{}", self.addr, index))
            .send()?;
        self.handle_response(response)
    }

    pub fn status(&self, uid: Option<UpdateId>) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/updates/{}",
                self.addr,
                self.index,
                uid.map_or("".to_string(), |uid| uid.to_string())
            ))
            .send()?;
        self.handle_response(response)
    }

    pub fn task_by_index(&self, tid: Option<TaskId>) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/tasks/{}",
                self.addr,
                self.index,
                tid.map_or("".to_string(), |uid| uid.to_string())
            ))
            .send()?;
        self.handle_response(response)
    }

    pub fn global_task(&self, tid: Option<TaskId>) -> Result<()> {
        let response = self
            .get(format!(
                "{}/tasks/{}",
                self.addr,
                tid.map_or("".to_string(), |uid| uid.to_string())
            ))
            .send()?;
        self.handle_response(response)
    }

    pub fn create_dump(&self) -> Result<()> {
        let response = self.post(format!("{}/dumps", self.addr)).send()?;
        self.handle_response(response)
    }

    pub fn dump_status(&self, dump_id: DumpId) -> Result<()> {
        let response = self
            .get(format!("{}/dumps/{}/status", self.addr, dump_id))
            .send()?;
        self.handle_response(response)
    }

    pub fn healthcheck(&self) -> Result<()> {
        let response = self.get(format!("{}/health", self.addr)).send()?;
        self.handle_response(response)
    }

    pub fn version(&self) -> Result<()> {
        let response = self.get(format!("{}/version", self.addr)).send()?;
        self.handle_response(response)
    }

    pub fn stats(&self) -> Result<()> {
        let response = self.get(format!("{}/stats", self.addr)).send()?;
        self.handle_response(response)
    }

    pub fn keys(&self) -> Result<()> {
        let response = self.get(format!("{}/keys", self.addr)).send()?;
        self.handle_response(response)
    }

    pub fn handle_response(&self, response: Response) -> Result<()> {
        if response.status() == StatusCode::NO_CONTENT {
            return write_response_headers(&response, self.verbose);
        }
        let response = write_response_full(response, self.verbose)?;
        if self.r#async {
            return Ok(());
        }

        let spinner = ProgressBar::new_spinner();

        let buffer = String::new();

        if let Some(uid) = response["uid"].as_i64() {
            loop {
                let response = self.get(format!("{}/tasks/{}", self.addr, uid)).send()?;
                let json = response.json::<Value>()?;
                match json["status"].as_str() {
                    None => {
                        return Ok(());
                    }
                    Some(msg @ "succeeded") | Some(msg @ "failed") => {
                        spinner.finish_with_message(msg.to_string());
                        write_json(json);
                        break;
                    }
                    Some(status) => spinner.set_message(status.to_string()),
                }
                std::thread::sleep(std::time::Duration::from_millis(self.interval as u64));
            }
        } else if let Some(uid) = response["updateId"].as_i64() {
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
