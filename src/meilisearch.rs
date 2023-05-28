use std::io::{stdin, Read};

use crate::{
    format::{write_json, write_response_full, write_response_headers},
    options::TasksFilter,
    DumpId, TaskId,
};
use clap::Parser;
use indicatif::ProgressBar;
use miette::{IntoDiagnostic, Result};
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    header::{CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use serde_json::{json, Map, Value};

#[derive(Debug, Parser)]
pub struct Meilisearch {
    /// Verbose mode (-v, -vv, etc)
    #[structopt(global = true, short, parse(from_occurrences))]
    pub verbose: usize,

    /// The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700)
    #[structopt(
        global = true,
        short,
        long,
        default_value = "http://localhost:7700",
        env = "MEILI_ADDR"
    )]
    pub addr: String,

    /// The command will exit immediatly after executing.
    #[structopt(global = true, long)]
    pub r#async: bool,

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
    #[structopt(global = true, short, long, env = "MEILI_MASTER_KEY")]
    pub key: Option<String>,

    /// Use a specific http User-Agent for your request
    #[structopt(
        global = true,
        long,
        default_value = concat!("mieli/", env!("CARGO_PKG_VERSION")),
    )]
    pub user_agent: String,

    /// Use a specific http header for your request.
    /// Eg. `mieli search --custom-header "x-meilisearch-client: turbo-doggo/42.9000"`
    #[structopt(global = true, long)]
    pub custom_header: Option<String>,

    /// Interval between each status check (in milliseconds)
    #[structopt(global = true, long, default_value = "200")]
    pub interval: usize,
}

impl Meilisearch {
    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.get(url.as_ref()))
    }

    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.post(url.as_ref()))
    }

    pub fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.put(url.as_ref()))
    }

    pub fn patch(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.patch(url.as_ref()))
    }

    pub fn delete(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(|c| c.delete(url.as_ref()))
    }

    fn request(
        &self,
        closure: impl Fn(Client) -> RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        let mut req_builder = closure(Client::new());
        if let Some(ref key) = self.key {
            req_builder = req_builder.header("X-Meili-API-Key", key).bearer_auth(key);
        }
        if let Some((key, value)) = self.custom_header.as_ref().and_then(|h| h.split_once(':')) {
            req_builder = req_builder.header(key, value);
        }
        req_builder.header(USER_AGENT, &self.user_agent)
    }

    pub fn search(&self, search: String) -> Result<()> {
        let mut value: Map<String, Value> = if atty::isnt(atty::Stream::Stdin) {
            serde_json::from_reader(stdin()).into_diagnostic()?
        } else {
            Map::new()
        };
        if !search.is_empty() {
            value.insert("q".to_string(), json!(search));
        }
        let response = self
            .post(format!("{}/indexes/{}/search", self.addr, self.index))
            .header(CONTENT_TYPE, "application/json")
            .json(&value)
            .send()
            .into_diagnostic()?;

        self.handle_response(response)
    }

    pub fn interactive_search(&self, search: String) -> Result<()> {
        if atty::isnt(atty::Stream::Stdout) {
            return self.search(search);
        }

        let mut value: Map<String, Value> = if atty::isnt(atty::Stream::Stdin) {
            serde_json::from_reader(stdin()).into_diagnostic()?
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
                .send()
                .into_diagnostic()?
        } else {
            let mut buffer = Vec::new();
            stdin().read_to_end(&mut buffer).into_diagnostic()?;

            let url = format!("{}/indexes/{}/settings", self.addr, self.index);
            let mut response = self
                .patch(&url)
                .header(CONTENT_TYPE, "application/json")
                .body(buffer.clone())
                .send()
                .into_diagnostic()?;

            if response.status().as_u16() == 405 {
                response = self
                    .post(url)
                    .header(CONTENT_TYPE, "application/json")
                    .body(buffer)
                    .send()
                    .into_diagnostic()?;
            }
            response
        };

        self.handle_response(response)
    }

    pub fn tasks(&self, tid: Option<TaskId>, task_filter: TasksFilter) -> Result<()> {
        let response = self
            .get(format!(
                "{}/tasks/{}?{}",
                self.addr,
                tid.map_or("".to_string(), |uid| uid.to_string()),
                yaup::to_string(&task_filter).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    pub fn create_dump(&self) -> Result<()> {
        let response = self
            .post(format!("{}/dumps", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    pub fn dump_status(&self, dump_id: DumpId) -> Result<()> {
        let response = self
            .get(format!("{}/dumps/{}/status", self.addr, dump_id))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    pub fn healthcheck(&self) -> Result<()> {
        let response = self
            .get(format!("{}/health", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    pub fn version(&self) -> Result<()> {
        let response = self
            .get(format!("{}/version", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    pub fn stats(&self) -> Result<()> {
        let response = self
            .get(format!("{}/stats", self.addr))
            .send()
            .into_diagnostic()?;
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

        if let Some(uid) = response["uid"]
            .as_i64()
            .or_else(|| response["taskUid"].as_i64())
        {
            loop {
                let response = self
                    .get(format!("{}/tasks/{}", self.addr, uid))
                    .send()
                    .into_diagnostic()?;
                let json = response.json::<Value>().into_diagnostic()?;
                match json["status"].as_str() {
                    None => {
                        return Ok(());
                    }
                    Some(msg @ "succeeded") | Some(msg @ "failed") => {
                        spinner.finish_with_message(msg.to_string());
                        write_json(json)?;
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
                    .send()
                    .into_diagnostic()?;
                let json = response.json::<Value>().into_diagnostic()?;
                match json["status"].as_str() {
                    None => {
                        return Ok(());
                    }
                    Some(msg @ "processed") | Some(msg @ "failed") => {
                        spinner.finish_with_message(msg.to_string());
                        write_json(json)?;
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
