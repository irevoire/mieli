use std::io::{stdin, Read};

use crate::format::{write_json, write_response_full, write_response_headers};
use clap::Parser;
use miette::{IntoDiagnostic, Result};
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    header::{CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Parser)]
pub struct Meilisearch {
    #[clap(global = true, short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// The server address in the format of ip_addr:port (ex: http://0.0.0.0:7700)
    #[clap(
        global = true,
        short,
        long,
        default_value = "http://localhost:7700",
        env = "MEILI_ADDR"
    )]
    pub addr: String,

    /// The command will exit immediatly after executing.
    #[clap(global = true, long)]
    pub r#async: bool,

    /// The name of the index
    #[clap(
        global = true,
        short,
        long,
        default_value = "mieli",
        env = "MIELI_INDEX"
    )]
    pub index: String,

    /// Your secret API key <https://docs.meilisearch.com/reference/api/keys.html#get-keys>
    #[clap(global = true, short, long, env = "MEILI_MASTER_KEY")]
    pub key: Option<String>,

    /// Use a specific http User-Agent for your request
    #[clap(
        global = true,
        long,
        default_value = concat!("mieli/", env!("CARGO_PKG_VERSION")),
    )]
    pub user_agent: String,

    /// Use a specific http header for your request.
    /// Eg. `mieli search --custom-header "x-meilisearch-client: turbo-doggo/42.9000"`
    #[clap(global = true, long)]
    pub custom_header: Option<String>,

    /// Interval between each status check (in milliseconds)
    #[clap(global = true, long, default_value = "200")]
    pub interval: usize,
}

impl Meilisearch {
    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        log::debug!("GET {}", url.as_ref());
        self.request(|c| c.get(url.as_ref()))
    }

    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        log::debug!("POST {}", url.as_ref());
        self.request(|c| c.post(url.as_ref()))
    }

    pub fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        log::debug!("PUT {}", url.as_ref());
        self.request(|c| c.put(url.as_ref()))
    }

    pub fn patch(&self, url: impl AsRef<str>) -> RequestBuilder {
        log::debug!("PATCH {}", url.as_ref());
        self.request(|c| c.patch(url.as_ref()))
    }

    pub fn delete(&self, url: impl AsRef<str>) -> RequestBuilder {
        log::debug!("DELETE {}", url.as_ref());
        self.request(|c| c.delete(url.as_ref()))
    }

    fn request(
        &self,
        closure: impl Fn(Client) -> RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        let mut req_builder = closure(Client::new());
        if let Some(ref key) = self.key {
            req_builder = req_builder.bearer_auth(key);
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

    pub fn create_dump(&self) -> Result<()> {
        let response = self
            .post(format!("{}/dumps", self.addr))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    pub fn create_snapshot(&self) -> Result<()> {
        let response = self
            .post(format!("{}/snapshots", self.addr))
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
        let mut response = write_response_full(response, self.verbose)?;
        if self.r#async {
            return Ok(());
        }

        let uid = response["taskUid"].as_i64().or(response["uid"].as_i64());
        if let Some(uid) = uid {
            if response["status"] == json!("processing") || response["status"] == json!("enqueued")
            {
                let mut progress = json!(null);
                println!();
                loop {
                    let new_response = self
                        .get(format!("{}/tasks/{}", self.addr, uid))
                        .send()
                        .into_diagnostic()?;
                    let new_response = new_response.json::<Value>().into_diagnostic()?;
                    let new_progress = match new_response["batchUid"].as_i64() {
                        Some(batch_uid) => {
                            let new_progress = self
                                .get(format!("{}/batches/{}", self.addr, batch_uid))
                            .send()
                            .into_diagnostic()?;
                                let new_progress = new_progress.json::<Value>().into_diagnostic()?;
                                new_progress["progress"].clone()
                            }
                            None => json!(null),
                        };
                    #[rustfmt::skip]
                    let lines = serde_json::to_string_pretty(&response).unwrap().lines().count()
                        + serde_json::to_string_pretty(&progress).unwrap().lines().count()
                        + 1; // because we're doing a print*ln*
                    println!("{}", "\x1b[K\x1b[A".repeat(lines));
                    let new_response = write_json(new_response)?;
                    let new_progress = write_json(new_progress)?;

                    match new_response["status"].as_str() {
                        None => {
                            return Ok(());
                        }
                        Some("succeeded" | "failed" | "canceled") => {
                            break;
                        }
                        _ => (),
                    }
                    std::thread::sleep(std::time::Duration::from_millis(self.interval as u64));

                    response = new_response;
                    progress = new_progress;
                }
            } else if response["progress"].is_null() {
                loop {
                    let new_response = self
                        .get(format!("{}/batches/{}", self.addr, uid))
                        .send()
                        .into_diagnostic()?;
                    let new_response = new_response.json::<Value>().into_diagnostic()?;
                    #[rustfmt::skip]
                    let lines = serde_json::to_string_pretty(&response).unwrap().lines().count()
                        + 1; // because we're doing a print*ln*
                    println!("{}", "\x1b[K\x1b[A".repeat(lines));
                    let new_response = write_json(new_response)?;
                    std::thread::sleep(std::time::Duration::from_millis(self.interval as u64));

                    response = new_response;
                }
            }
        }
        Ok(())
    }
}
