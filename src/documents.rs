use crate::Meilisearch;
use clap::Parser;
use miette::{bail, IntoDiagnostic, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;
use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

pub type DocId = String;

#[derive(Debug, Parser)]
pub enum Documents {
    /// Get one document. If no argument are specified it returns all documents.
    #[clap(aliases = &["g"])]
    Get {
        /// The id of the document you want to retrieve
        #[clap(long, conflicts_with = "filter")]
        id: Option<String>,
        /// The filter used to retrieve the document
        #[clap(long)]
        filter: Option<String>,
        /// Query parameters.
        #[clap(flatten)]
        params: GetDocumentParameter,
    },
    /// Add documents with the `post` verb
    /// You can pipe your documents in the command
    /// Will try to infer the content-type from the file extension if it fail
    /// it'll be set as json.
    #[clap(aliases = &["a"])]
    Add {
        /// Set the content-type of your file.
        #[clap(short)]
        content_type: Option<String>,
        /// The primary key
        #[clap(short, long)]
        primary: Option<String>,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Replace documents with the `put` verb
    /// You can pipe your documents in the command
    /// Will try to infer the content-type from the file extension if it fail
    /// it'll be set as json.
    #[clap(aliases = &["u"])]
    Update {
        /// Set the content-type of your file
        #[clap(short)]
        content_type: Option<String>,
        /// The primary key
        #[clap(short, long)]
        primary: Option<String>,
        /// The file you want to send
        file: Option<PathBuf>,
    },
    /// Delete documents. If no argument are specified all documents are deleted.
    #[clap(aliases = &["d"])]
    Delete {
        /// The list of document ids you want to delete
        document_ids: Vec<DocId>,
    },
}

#[derive(Debug, Parser, Serialize)]
pub struct GetDocumentParameter {
    /// Number of documents to return.
    #[clap(long, aliases = &["limits"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    /// Skip the n first documents.
    #[clap(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<usize>,
    /// Select fields from the documents.
    #[clap(long, aliases = &["field"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<String>,
}

impl Documents {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Documents::Get {
                params,
                id: None,
                filter: None,
            } => meili.get_all_documents(params),
            Documents::Get {
                params,
                id: Some(id),
                ..
            } => meili.get_one_document(params, id),
            Documents::Get {
                params,
                filter: Some(filter),
                ..
            } => meili.get_documents_by_filter(params, filter),
            Documents::Add {
                content_type,
                file,
                primary,
            } => meili.index_documents(file, primary, content_type, false),
            Documents::Update {
                content_type,
                file,
                primary,
            } => meili.index_documents(file, primary, content_type, true),
            Documents::Delete { document_ids } => match document_ids.as_slice() {
                [] => meili.delete_all(),
                [id] => meili.delete_one(id.clone()),
                ids => meili.delete_batch(ids),
            },
        }
    }
}

impl Meilisearch {
    fn get_one_document(&self, params: GetDocumentParameter, docid: DocId) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/documents/{}?{}",
                self.addr,
                self.index,
                docid,
                yaup::to_string(&params).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_all_documents(&self, params: GetDocumentParameter) -> Result<()> {
        let response = self
            .get(&format!(
                "{}/indexes/{}/documents?{}",
                self.addr,
                self.index,
                yaup::to_string(&params).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_documents_by_filter(&self, params: GetDocumentParameter, filter: String) -> Result<()> {
        let mut payload = serde_json::to_value(params).into_diagnostic()?;
        let payload = payload.as_object_mut().expect("impossiburuuu");
        payload.insert("filter".into(), filter.into());

        let response = self
            .post(&format!(
                "{}/indexes/{}/documents/fetch",
                self.addr, self.index,
            ))
            .json(payload)
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn index_documents(
        &self,
        filepath: Option<PathBuf>,
        primary_key: Option<String>,
        content_type: Option<String>,
        reindex: bool,
    ) -> Result<()> {
        let url = format!("{}/indexes/{}/documents", self.addr, self.index);
        let client = match reindex {
            false => self.post(url),
            true => self.put(url),
        };
        let client = if let Some(content_type) = content_type {
            client.header(CONTENT_TYPE, content_type)
        } else {
            match filepath
                .as_ref()
                .and_then(|filepath| filepath.extension())
                .and_then(|ext| ext.to_str())
            {
                Some("csv") => client.header(CONTENT_TYPE, "text/csv"),
                Some("jsonl") | Some("ndjson") | Some("jsonlines") => {
                    client.header(CONTENT_TYPE, "text/x-ndjson")
                }
                _ => client.header(CONTENT_TYPE, "application/json"),
            }
        };
        let client = if let Some(primary_key) = primary_key {
            client.query(&[("primaryKey", primary_key)])
        } else {
            client
        };

        let response = match filepath {
            Some(filepath) => {
                let file = File::open(filepath).into_diagnostic()?;
                client.body(file).send().into_diagnostic()?
            }
            None if atty::isnt(atty::Stream::Stdin) => {
                let mut buffer = Vec::new();
                stdin().read_to_end(&mut buffer).into_diagnostic()?;

                client.body(buffer).send().into_diagnostic()?
            }
            None => bail!("Did you forgot to pipe something in the command?"),
        };
        self.handle_response(response)
    }

    fn delete_all(&self) -> Result<()> {
        let response = self
            .delete(format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn delete_one(&self, docid: DocId) -> Result<()> {
        let response = self
            .delete(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn delete_batch(&self, docids: &[DocId]) -> Result<()> {
        let response = self
            .post(format!(
                "{}/indexes/{}/documents/delete-batch",
                self.addr, self.index
            ))
            .json(docids)
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
