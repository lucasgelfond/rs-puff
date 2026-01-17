use serde::Serialize;
use std::collections::HashMap;

use crate::{DistanceMetric, Filter, RankBy, VectorEncoding};

#[derive(Debug, Clone, Default, Serialize)]
pub struct WriteParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upsert_rows: Option<Vec<HashMap<String, serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub upsert_columns: Option<HashMap<String, Vec<serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_rows: Option<Vec<HashMap<String, serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_columns: Option<HashMap<String, Vec<serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletes: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_by_filter: Option<Filter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_by_filter: Option<PatchByFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub upsert_condition: Option<Filter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_condition: Option<Filter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_condition: Option<Filter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_metric: Option<DistanceMetric>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_by_filter_allow_partial: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_by_filter_allow_partial: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_backpressure: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_affected_ids: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_from_namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchByFilter {
    pub filters: Filter,
    pub patch: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_by: Option<RankBy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Filter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_attributes: Option<IncludeAttributes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_attributes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_encoding: Option<VectorEncoding>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_metric: Option<DistanceMetric>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<Consistency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate_by: Option<HashMap<String, AggregateBy>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum IncludeAttributes {
    All(bool),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize)]
pub struct Consistency {
    pub level: ConsistencyLevel,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyLevel {
    Strong,
    Eventual,
}

#[derive(Debug, Clone)]
pub enum AggregateBy {
    Count,
    Sum(String),
}

impl serde::Serialize for AggregateBy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        match self {
            AggregateBy::Count => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element("Count")?;
                seq.end()
            }
            AggregateBy::Sum(attr) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("Sum")?;
                seq.serialize_element(attr)?;
                seq.end()
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MultiQueryParams {
    pub queries: Vec<QueryParams>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_encoding: Option<VectorEncoding>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<Consistency>,
}
