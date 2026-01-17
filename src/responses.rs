use serde::Deserialize;
use std::collections::HashMap;

use crate::Row;

#[derive(Debug, Clone, Deserialize)]
pub struct WriteResponse {
    pub rows_affected: u64,

    #[serde(default)]
    pub rows_upserted: Option<u64>,

    #[serde(default)]
    pub rows_patched: Option<u64>,

    #[serde(default)]
    pub rows_deleted: Option<u64>,

    #[serde(default)]
    pub rows_remaining: Option<bool>,

    #[serde(default)]
    pub upserted_ids: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub patched_ids: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub deleted_ids: Option<Vec<serde_json::Value>>,

    #[serde(default)]
    pub billing: Option<WriteBilling>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteBilling {
    pub billable_logical_bytes_written: u64,

    #[serde(default)]
    pub query: Option<QueryBillingInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryBillingInfo {
    pub billable_logical_bytes_queried: u64,
    pub billable_logical_bytes_returned: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryResponse {
    #[serde(default)]
    pub rows: Vec<Row>,

    #[serde(default)]
    pub aggregations: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub aggregation_groups: Option<Vec<HashMap<String, serde_json::Value>>>,

    #[serde(default)]
    pub billing: Option<QueryBilling>,

    #[serde(default)]
    pub performance: Option<QueryPerformance>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryBilling {
    pub billable_logical_bytes_queried: u64,
    pub billable_logical_bytes_returned: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryPerformance {
    #[serde(default)]
    pub cache_hit_ratio: Option<f64>,

    #[serde(default)]
    pub cache_temperature: Option<String>,

    #[serde(default)]
    pub server_total_ms: Option<u64>,

    #[serde(default)]
    pub query_execution_ms: Option<u64>,

    #[serde(default)]
    pub exhaustive_search_count: Option<u64>,

    #[serde(default)]
    pub approx_namespace_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultiQueryResponse {
    pub results: Vec<QueryResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteAllResponse {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceMetadata {
    #[serde(default)]
    pub approx_count: Option<u64>,

    #[serde(default)]
    pub dimensions: Option<u32>,

    #[serde(default)]
    pub created_at: Option<String>,

    #[serde(default)]
    pub unindexed_bytes: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchemaResponse(pub HashMap<String, serde_json::Value>);

#[derive(Debug, Clone, Deserialize)]
pub struct HintCacheWarmResponse {
    pub status: String,

    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceSummary {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespacesResponse {
    pub namespaces: Vec<NamespaceSummary>,

    #[serde(default)]
    pub next_cursor: Option<String>,
}
