use clap::Parser;
use futures::AsyncRead;
use meilisearch_sdk::{
    documents::{DocumentQuery, DocumentsQuery},
    indexes::Index,
};
use miette::{bail, IntoDiagnostic, Result};
use serde::Serialize;
use serde_json::Value;
use std::{path::PathBuf, pin::Pin};
use tokio::fs::File;
use tokio::io::stdin;
use tokio_util::compat::TokioAsyncReadCompatExt;

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
    #[clap(long, aliases = &["from"])]
    offset: Option<usize>,
    /// Select fields from the documents.
    #[clap(long, aliases = &["field"])]
    fields: Option<Vec<String>>,
}

impl Documents {
    pub async fn execute(self, index: Index) -> Result<()> {
        match self {
            Documents::Get {
                document_id: None,
                param,
            } => {
                let documents = index
                    .get_documents_with::<Value>(&DocumentsQuery {
                        index: &index,
                        offset: param.offset,
                        limit: param.limit,
                        fields: param
                            .fields
                            .as_ref()
                            .map(|fields| fields.iter().map(|field| field.as_str()).collect()),
                    })
                    .await
                    .unwrap();

                dbg!(documents.results);

                Ok(())
            }
            Documents::Get {
                document_id: Some(id),
                param,
            } => {
                let document = index
                    .get_document_with::<Value>(
                        &id,
                        &DocumentQuery {
                            index: &index,
                            fields: param
                                .fields
                                .as_ref()
                                .map(|fields| fields.iter().map(|field| field.as_str()).collect()),
                        },
                    )
                    .await
                    .unwrap();
                dbg!(document);
                Ok(())
            }
            Documents::Add {
                content_type,
                file,
                primary,
            } => index_documents(index, file, primary, content_type, false).await,
            Documents::Update {
                content_type,
                file,
                primary,
            } => index_documents(index, file, primary, content_type, true).await,
            Documents::Delete { document_ids } => match document_ids.as_slice() {
                [] => {
                    index.delete_all_documents().await.into_diagnostic()?;
                    Ok(())
                }
                [id] => {
                    index.delete_document(&id).await.into_diagnostic()?;
                    Ok(())
                }
                ids => {
                    index.delete_documents(ids).await.into_diagnostic()?;
                    Ok(())
                }
            },
        }
    }
}

async fn index_documents(
    index: Index,
    filepath: Option<PathBuf>,
    primary_key: Option<String>,
    content_type: Option<String>,
    reindex: bool,
) -> Result<()> {
    let content_type = if let Some(ref content_type) = content_type {
        content_type.as_str()
    } else {
        match filepath
            .as_ref()
            .and_then(|filepath| filepath.extension())
            .and_then(|ext| ext.to_str())
        {
            Some("csv") => "text/csv",
            Some("jsonl") | Some("ndjson") | Some("jsonlines") => "text/x-ndjson",
            _ => "application/json",
        }
    };

    let reader = match filepath {
        Some(filepath) => {
            let file = File::open(filepath).await.into_diagnostic()?;
            Box::pin(file.compat()) as Pin<Box<dyn AsyncRead + Sync + Send>>
        }
        None if atty::isnt(atty::Stream::Stdin) => {
            Box::pin(stdin().compat()) as Pin<Box<dyn AsyncRead + Sync + Send>>
        }
        None => bail!("Did you forgot to pipe something in the command?"),
    };

    if reindex {
        index
            .add_or_replace_unchecked_payload(reader, content_type, primary_key.as_deref())
            .await
            .into_diagnostic()?;
    } else {
        index
            .add_or_update_unchecked_payload(reader, content_type, primary_key.as_deref())
            .await
            .into_diagnostic()?;
    }

    // self.handle_response(response)
    Ok(())
}
