use std::io::{stdin, Read};

use crate::{DocId, DumpId, Options, UpdateId};
use anyhow::Result;
use reqwest::blocking::Client;

use crate::handle_response;

#[derive(Debug, Default)]
pub struct Meilisearch {
    addr: String,
    index: String,
}

impl From<&Options> for Meilisearch {
    fn from(options: &Options) -> Self {
        Self {
            addr: options.addr.clone(),
            index: options.index.clone(),
        }
    }
}

impl Meilisearch {
    pub fn get_one_document(&self, docid: DocId) -> Result<()> {
        let response = Client::new()
            .get(&format!("{}/indexes/{}/documents/{}", self.addr, self.index, docid))
            .send()?;
        handle_response(response)
    }

    pub fn get_all_documents(&self) -> Result<()> {
        // TODO: we should cycle to get ALL the documents
        let response = Client::new()
            .get(&format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        handle_response(response)
    }

    pub fn index_documents(&self, content_type: String, reindex: bool) -> Result<()> {
        // TODO: is this the only way to do it?
        let mut buffer = Vec::new();
        stdin().read_to_end(&mut buffer);

        let url = format!("{}/indexes/{}/documents", self.addr, self.index);

        let client = match reindex {
            false => Client::new().post(url),
            true => Client::new().put(url),
        };
        let response = client
            .header("Content-Type", content_type)
            .body(buffer)
            .send()?;
        handle_response(response)
    }

    pub fn delete_all(&self) -> Result<()> {
        let response = Client::new()
            .delete(format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()?;
        handle_response(response)
    }

    pub fn delete_one(&self, docid: DocId) -> Result<()> {
        let response = Client::new()
            .delete(format!("{}/indexes/{}/documents/{}", self.addr, self.index, docid))
            .send()?;
        handle_response(response)
    }

    pub fn delete_batch(&self, docids: &[DocId]) -> Result<()> {
        let response = Client::new()
            .post(format!("{}/indexes/{}/documents/delete-batch", self.addr, self.index))
            .json(docids)
            .send()?;
        handle_response(response)
    }

    pub fn status(&self, uid: UpdateId) -> Result<()> {
        let response = Client::new()
            .get(format!("{}/indexes/{}/updates/{}", self.addr, self.index, uid))
            .send()?;
        handle_response(response)
    }

    pub fn create_dump(&self) -> Result<()> {
        let response = Client::new()
            .post(format!("{}/dumps", self.addr))
            .send()?;
        handle_response(response)
    }

    pub fn dump_status(&self, dump_id: DumpId) -> Result<()> {
        let response = Client::new()
            .get(format!("{}/dumps/{}/status", self.addr, dump_id))
            .send()?;
        handle_response(response)
    }

    pub fn healthcheck(&self) -> Result<()> {
        let response = Client::new()
            .get(format!("{}/health", self.addr))
            .send()?;
        handle_response(response)
    }

    pub fn version(&self) -> Result<()> {
        let response = Client::new()
            .get(format!("{}/version", self.addr))
            .send()?;
        handle_response(response)
    }

    pub fn stats(&self) -> Result<()> {
        let response = Client::new()
            .get(format!("{}/stats", self.addr))
            .send()?;
        handle_response(response)
    }
}
