use crate::Meilisearch;
use clap::Parser;
use miette::{bail, IntoDiagnostic, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;
use serde_json::json;
use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

pub type DocId = String;

#[derive(Debug, Parser)]
pub enum DocumentsCommand {
    /// Get one document. If no argument are specified it returns all documents.
    #[clap(aliases = &["g"])]
    Get {
        /// The id of the document you want to retrieve
        #[clap(long)]
        id: Option<String>,
        /// Use the `POST /documents/fetch` route with the payload as json instead of the `GET /documents` with query parameters
        #[clap(long, default_value = "false", aliases = &["byPost", "post", "fetch"])]
        by_post: bool,
        // These parameters are available both for get and post
        #[clap(flatten)]
        base_params: GetDocumentBaseParameter,
        #[clap(flatten)]
        extra_params: GetDocumentExtraParameter,
    },
    /// Add documents with the `post` verb
    /// You can pipe your documents in the command
    /// Will try to infer the content-type from the file extension if it fail
    /// it'll be set as json.
    #[clap(aliases = &["a"])]
    Add(AddOrUpdate),
    /// Replace documents with the `put` verb
    /// You can pipe your documents in the command
    /// Will try to infer the content-type from the file extension if it fail
    /// it'll be set as json.
    #[clap(aliases = &["u"])]
    Update(AddOrUpdate),
    /// Update documents with function
    /// The payload must be sent through stdin
    #[clap(aliases = &["e"])]
    Edit,
    /// Delete documents. If no argument are specified all documents are deleted.
    #[clap(aliases = &["d"])]
    Delete {
        /// The ids of the documents you want to delete
        #[clap(long, conflicts_with = "filter")]
        ids: Option<Vec<DocId>>,
        /// The filter used to delete the documents
        #[clap(long)]
        filter: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub struct AddOrUpdate {
    /// Set the content-type of your file. It should be either `application/json`, `application/x-ndjson`, `text/csv`.
    #[clap(short)]
    content_type: Option<String>,
    /// The primary key
    #[clap(short, long, aliases = &["primary-key", "primary_key", "primaryKey", "pk"])]
    primary: Option<String>,
    /// Configure the character separating CSV fields. Must be a string containing one ASCII character.
    #[clap(long)]
    csv_delimiter: Option<String>,
    /// The file you want to send
    file: Option<PathBuf>,
}

#[derive(Default, PartialEq, Eq, Debug, Parser, Serialize)]
pub struct GetDocumentBaseParameter {
    #[clap(long, aliases = &["field"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<String>,
    /// Return document vector data with search result
    #[clap(long, aliases = &["vector", "vectors", "retrieve_vector", "retrieveVectors", "retrieveVector"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    retrieve_vectors: Option<String>,
}

#[derive(Default, PartialEq, Eq, Debug, Parser, Serialize)]
pub struct GetDocumentExtraParameter {
    /// Number of documents to skip
    #[clap(long, aliases = &["from"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<usize>,
    /// Number of documents to return
    #[clap(long, aliases = &["limits"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    /// Refine results based on attributes in the `filterableAttributes` list
    #[clap(long, aliases = &["filters"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,
}

impl DocumentsCommand {
    pub fn execute(self, meili: Meilisearch) -> Result<()> {
        match self {
            DocumentsCommand::Get {
                extra_params,
                base_params,
                id: None,
                by_post,
            } => meili.get_all_documents(base_params, extra_params, by_post),
            DocumentsCommand::Get {
                base_params,
                extra_params,
                by_post,
                id: Some(id),
            } => {
                if GetDocumentExtraParameter::default() != extra_params {
                    log::warn!("extra parameters have been specified while retrieving a document by id. The following parameters will be ignored: {}", serde_json::to_string(&extra_params).unwrap());
                } else if by_post {
                    log::warn!("--by-post have been specified while retrieving a document by id. That's not possible and will be ignored");
                }
                meili.get_one_document(base_params, id)
            }
            DocumentsCommand::Add(params) => meili.index_documents(params, false),
            DocumentsCommand::Update(params) => meili.index_documents(params, true),
            DocumentsCommand::Edit => meili.edit_documents(),
            DocumentsCommand::Delete {
                ids: None,
                filter: None,
            } => meili.delete_all(),
            DocumentsCommand::Delete { ids: Some(ids), .. } => match ids.as_slice() {
                [] => meili.delete_all(),
                [id] => meili.delete_one(id.clone()),
                ids => meili.delete_batch(ids),
            },
            DocumentsCommand::Delete {
                filter: Some(filter),
                ..
            } => meili.delete_documents_by_filter(filter),
        }
    }
}

impl Meilisearch {
    fn get_one_document(&self, params: GetDocumentBaseParameter, docid: DocId) -> Result<()> {
        let response = self
            .get(format!(
                "{}/indexes/{}/documents/{}{}",
                self.addr,
                self.index,
                docid,
                yaup::to_string(&params).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }

    fn get_all_documents(
        &self,
        base_params: GetDocumentBaseParameter,
        extra_params: GetDocumentExtraParameter,
        by_post: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Params {
            #[serde(flatten)]
            base_params: GetDocumentBaseParameter,
            #[serde(flatten)]
            extra_params: GetDocumentExtraParameter,
        }
        let params = Params {
            base_params,
            extra_params,
        };
        let response = if by_post {
            self.post(format!(
                "{}/indexes/{}/documents/fetch",
                self.addr, self.index,
            ))
            .json(&params)
            .send()
            .into_diagnostic()?
        } else {
            self.get(format!(
                "{}/indexes/{}/documents{}",
                self.addr,
                self.index,
                yaup::to_string(&params).into_diagnostic()?
            ))
            .send()
            .into_diagnostic()?
        };
        self.handle_response(response)
    }

    fn index_documents(&self, params: AddOrUpdate, reindex: bool) -> Result<()> {
        let url = format!("{}/indexes/{}/documents", self.addr, self.index);
        let client = match reindex {
            false => self.post(url),
            true => self.put(url),
        };
        let client = if let Some(content_type) = params.content_type {
            client.header(CONTENT_TYPE, content_type)
        } else {
            match params
                .file
                .as_ref()
                .and_then(|filepath| filepath.extension())
                .and_then(|ext| ext.to_str())
            {
                Some("csv") => client.header(CONTENT_TYPE, "text/csv"),
                Some("jsonl") | Some("ndjson") | Some("jsonlines") => {
                    client.header(CONTENT_TYPE, "application/x-ndjson")
                }
                _ => client.header(CONTENT_TYPE, "application/json"),
            }
        };
        let client = if let Some(primary_key) = params.primary {
            client.query(&[("primaryKey", primary_key)])
        } else {
            client
        };

        let response = match params.file {
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

    pub fn edit_documents(&self) -> std::result::Result<(), miette::Error> {
        let value: serde_json::Value = if atty::isnt(atty::Stream::Stdin) {
            serde_json::from_reader(stdin()).into_diagnostic()?
        } else {
            bail!("The payload must be sent through stdin with the edit document by filter route. See the documentation at https://www.meilisearch.com/docs/reference/api/documents#update-documents-with-function");
        };

        let response = self
            .post(format!("{}/indexes/{}/edit", self.addr, self.index))
            .header(CONTENT_TYPE, "application/json")
            .json(&value)
            .send()
            .into_diagnostic()?;

        self.handle_response(response)
    }

    fn delete_documents_by_filter(&self, filter: String) -> Result<()> {
        let response = self
            .post(format!(
                "{}/indexes/{}/documents/delete",
                self.addr, self.index
            ))
            .json(&json!({ "filter": filter }))
            .send()
            .into_diagnostic()?;
        self.handle_response(response)
    }
}
