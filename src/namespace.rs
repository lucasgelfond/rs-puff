use reqwest::Method;

use crate::{
    Client, Error, Result,
    params::{MultiQueryParams, QueryParams, WriteParams},
    responses::{
        DeleteAllResponse, HintCacheWarmResponse, MultiQueryResponse, NamespaceMetadata,
        QueryResponse, SchemaResponse, WriteResponse,
    },
};

pub struct Namespace<'a> {
    client: &'a Client,
    name: String,
}

impl<'a> Namespace<'a> {
    pub(crate) fn new(client: &'a Client, name: String) -> Self {
        Self { client, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn v1_path(&self, suffix: &str) -> String {
        format!("/v1/namespaces/{}{}", self.name, suffix)
    }

    fn v2_path(&self, suffix: &str) -> String {
        format!("/v2/namespaces/{}{}", self.name, suffix)
    }

    pub async fn write(&self, params: WriteParams) -> Result<WriteResponse> {
        self.client
            .request(Method::POST, &self.v2_path(""), Some(&params))
            .await
    }

    pub async fn query(&self, params: QueryParams) -> Result<QueryResponse> {
        self.client
            .request(Method::POST, &self.v2_path("/query"), Some(&params))
            .await
    }

    pub async fn multi_query(&self, params: MultiQueryParams) -> Result<MultiQueryResponse> {
        self.client
            .request(Method::POST, &self.v2_path("/query"), Some(&params))
            .await
    }

    pub async fn delete_all(&self) -> Result<DeleteAllResponse> {
        self.client
            .request_no_body(Method::DELETE, &self.v2_path(""))
            .await
    }

    pub async fn metadata(&self) -> Result<NamespaceMetadata> {
        self.client
            .request_no_body(Method::GET, &self.v1_path("/metadata"))
            .await
    }

    pub async fn schema(&self) -> Result<SchemaResponse> {
        self.client
            .request_no_body(Method::GET, &self.v1_path("/schema"))
            .await
    }

    pub async fn hint_cache_warm(&self) -> Result<HintCacheWarmResponse> {
        self.client
            .request_no_body(Method::GET, &self.v1_path("/hint_cache_warm"))
            .await
    }

    /// Check if the namespace exists.
    ///
    /// Returns `true` if the namespace exists, `false` if it does not (404 error).
    /// Other errors are propagated.
    pub async fn exists(&self) -> Result<bool> {
        match self.metadata().await {
            Ok(_) => Ok(true),
            Err(Error::Api { status: 404, .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
