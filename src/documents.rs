use crate::Meilisearch;
use clap::Parser;
use meilisearch_sdk::{documents::DocumentsQuery, indexes::Index, Client};
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
        document_id: Option<DocId>,
        /// Query parameters.
        #[clap(flatten)]
        param: GetDocumentParameter,
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
    limit: Option<usize>,
    /// Skip the n first documents.
    #[clap(long)]
    from: Option<usize>,
    /// Select fields from the documents.
    #[clap(long, aliases = &["field"])]
    fields: Option<String>,
}

impl Documents {
    pub async fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            Documents::Get {
                document_id: None,
                param,
            } => meili.get_all_documents(param).await,
            Documents::Get {
                document_id: Some(id),
                ..
            } => meili.get_one_document(id).await,
            Documents::Add {
                content_type,
                file,
                primary,
            } => {
                meili
                    .index_documents(file, primary, content_type, false)
                    .await
            }
            Documents::Update {
                content_type,
                file,
                primary,
            } => {
                meili
                    .index_documents(file, primary, content_type, true)
                    .await
            }
            Documents::Delete { document_ids } => match document_ids.as_slice() {
                [] => meili.delete_all().await,
                [id] => meili.delete_one(id.clone()).await,
                ids => meili.delete_batch(ids).await,
            },
        }
    }
}

impl Meilisearch {
    async fn get_one_document(&self, docid: DocId) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    async fn get_all_documents(&self, params: GetDocumentParameter) -> Result<()> {
        let client = Client::new(&self.addr, self.key.clone().unwrap_or_default());
        let index = client.index(&self.index);

        let response = index
            .get_documents_with(&DocumentsQuery {
                index: &index,
                offset: params.from,
                limit: params.limit,
                fields: params.fields.map(|fields| vec![fields.as_ref()]),
            })
            .await
            .into_diagnostic()?;

        // let response = self
        //     .get(&format!(
        //         "{}/indexes/{}/documents?{}",
        //         self.addr,
        //         self.index,
        //         yaup::to_string(&params).into_diagnostic()?
        //     ))
        //     .send()
        //     .into_diagnostic()?;

        self.handle_response(response)
    }

    async fn index_documents(
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

    async fn delete_all(&self) -> Result<()> {
        let response = self
            .delete(format!("{}/indexes/{}/documents", self.addr, self.index))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    async fn delete_one(&self, docid: DocId) -> Result<()> {
        let response = self
            .delete(format!(
                "{}/indexes/{}/documents/{}",
                self.addr, self.index, docid
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    async fn delete_batch(&self, docids: &[DocId]) -> Result<()> {
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
