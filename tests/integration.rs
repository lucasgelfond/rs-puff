use rs_puff::{
    Client, DistanceMetric, Filter, IncludeAttributes, NamespacesParams, QueryParams, RankBy,
    WriteParams,
};
use serial_test::serial;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_prefix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("rust_sdk_{}_{}_", nonce, count)
}

fn setup() -> Client {
    dotenvy::dotenv().ok();
    Client::with_region(
        std::env::var("TURBOPUFFER_API_KEY").expect("TURBOPUFFER_API_KEY must be set"),
        "gcp-us-central1",
    )
}

fn row(id: i64, vector: Vec<f64>, attrs: Vec<(&str, serde_json::Value)>) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    map.insert("id".to_string(), serde_json::json!(id));
    map.insert("vector".to_string(), serde_json::json!(vector));
    for (k, v) in attrs {
        map.insert(k.to_string(), v);
    }
    map
}

#[tokio::test]
#[serial]
async fn test_sanity() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}sanity", prefix));

    // Clean up if exists
    let _ = ns.delete_all().await;

    // Write some rows
    let write_result = ns
        .write(WriteParams {
            upsert_rows: Some(vec![
                row(1, vec![1.0, 2.0], vec![
                    ("foo", serde_json::json!("bar")),
                    ("numbers", serde_json::json!([1, 2, 3])),
                    ("maybeNull", serde_json::Value::Null),
                    ("bool", serde_json::json!(true)),
                ]),
                row(2, vec![3.0, 4.0], vec![
                    ("foo", serde_json::json!("baz")),
                    ("numbers", serde_json::json!([2, 3, 4])),
                    ("maybeNull", serde_json::Value::Null),
                    ("bool", serde_json::json!(true)),
                ]),
                row(3, vec![3.0, 4.0], vec![
                    ("foo", serde_json::json!("baz")),
                    ("numbers", serde_json::json!([17])),
                    ("maybeNull", serde_json::json!("oh boy!")),
                    ("bool", serde_json::json!(true)),
                ]),
            ]),
            distance_metric: Some(DistanceMetric::CosineDistance),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(write_result.rows_affected, 3);

    // Query with vector search and filter
    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::vector("vector", vec![1.0, 1.0])),
            filters: Some(Filter::r#in("numbers", vec![2.into(), 4.into()])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 2);
    assert_eq!(results.rows[0].get("id").unwrap(), 2);
    assert_eq!(results.rows[1].get("id").unwrap(), 1);

    // Check performance info
    let perf = results.performance.unwrap();
    assert!(perf.approx_namespace_size.is_some());

    // Query with complex nested filters
    let results2 = ns
        .query(QueryParams {
            rank_by: Some(RankBy::vector("vector", vec![1.0, 1.0])),
            filters: Some(Filter::and(vec![
                Filter::or(vec![
                    Filter::r#in("numbers", vec![2.into(), 3.into()]),
                    Filter::r#in("numbers", vec![1.into(), 7.into()]),
                ]),
                Filter::or(vec![
                    Filter::eq("foo", "bar"),
                    Filter::r#in("numbers", vec![4.into()]),
                ]),
                Filter::not_eq("foo", serde_json::Value::Null),
                Filter::eq("maybeNull", serde_json::Value::Null),
                Filter::eq("bool", true),
            ])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results2.rows.len(), 2);
    assert_eq!(results2.rows[0].get("id").unwrap(), 2);
    assert_eq!(results2.rows[1].get("id").unwrap(), 1);

    // Delete one row
    ns.write(WriteParams {
        deletes: Some(vec![serde_json::json!(1)]),
        ..Default::default()
    })
    .await
    .unwrap();

    // Query again - should only get one result
    let results3 = ns
        .query(QueryParams {
            rank_by: Some(RankBy::vector("vector", vec![1.0, 1.0])),
            filters: Some(Filter::r#in("numbers", vec![2.into(), 4.into()])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results3.rows.len(), 1);
    assert_eq!(results3.rows[0].get("id").unwrap(), 2);

    // Clean up
    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_order_by_attribute() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}order_by_attribute", prefix));

    let _ = ns.delete_all().await;

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![("a", serde_json::json!("5"))]),
            row(2, vec![0.2, 0.2], vec![("a", serde_json::json!("4"))]),
            row(3, vec![0.3, 0.3], vec![("a", serde_json::json!("3"))]),
            row(4, vec![0.4, 0.4], vec![("a", serde_json::json!("2"))]),
            row(5, vec![0.5, 0.5], vec![("a", serde_json::json!("1"))]),
        ]),
        distance_metric: Some(DistanceMetric::EuclideanSquared),
        ..Default::default()
    })
    .await
    .unwrap();

    // Test ascending order
    let results_asc = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("a")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results_asc.rows.len(), 5);
    assert_eq!(results_asc.rows[0].get("id").unwrap(), 5);
    assert_eq!(results_asc.rows[1].get("id").unwrap(), 4);
    assert_eq!(results_asc.rows[2].get("id").unwrap(), 3);
    assert_eq!(results_asc.rows[3].get("id").unwrap(), 2);
    assert_eq!(results_asc.rows[4].get("id").unwrap(), 1);

    // Test descending order
    let results_desc = ns
        .query(QueryParams {
            rank_by: Some(RankBy::desc("a")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results_desc.rows.len(), 5);
    assert_eq!(results_desc.rows[0].get("id").unwrap(), 1);
    assert_eq!(results_desc.rows[1].get("id").unwrap(), 2);
    assert_eq!(results_desc.rows[2].get("id").unwrap(), 3);
    assert_eq!(results_desc.rows[3].get("id").unwrap(), 4);
    assert_eq!(results_desc.rows[4].get("id").unwrap(), 5);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_contains_and_contains_any() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}contains_and_contains_any", prefix));

    let _ = ns.delete_all().await;

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![
                ("tags", serde_json::json!(["python", "javascript", "rust"])),
                ("category", serde_json::json!("backend")),
            ]),
            row(2, vec![0.2, 0.2], vec![
                ("tags", serde_json::json!(["javascript", "typescript", "react"])),
                ("category", serde_json::json!("frontend")),
            ]),
            row(3, vec![0.3, 0.3], vec![
                ("tags", serde_json::json!(["rust", "go", "c++"])),
                ("category", serde_json::json!("systems")),
            ]),
            row(4, vec![0.4, 0.4], vec![
                ("tags", serde_json::json!(["python", "django", "flask"])),
                ("category", serde_json::json!("backend")),
            ]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    // Test Contains operator
    let contains_results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            filters: Some(Filter::contains("tags", "javascript")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut ids: Vec<i64> = contains_results
        .rows
        .iter()
        .map(|r| r.get("id").unwrap().as_i64().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);

    // Test ContainsAny operator
    let contains_any_results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            filters: Some(Filter::contains_any("tags", vec!["rust".into(), "typescript".into()])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut ids: Vec<i64> = contains_any_results
        .rows
        .iter()
        .map(|r| r.get("id").unwrap().as_i64().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2, 3]);

    // Test combined with And
    let combined = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            filters: Some(Filter::and(vec![
                Filter::contains("tags", "python"),
                Filter::eq("category", "backend"),
            ])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut ids: Vec<i64> = combined
        .rows
        .iter()
        .map(|r| r.get("id").unwrap().as_i64().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, vec![1, 4]);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_delete_by_filter() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}delete_by_filter", prefix));

    let _ = ns.delete_all().await;

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![1.0, 2.0], vec![("foo", serde_json::json!("bar"))]),
            row(2, vec![3.0, 4.0], vec![("foo", serde_json::json!("baz"))]),
            row(3, vec![3.0, 4.0], vec![("foo", serde_json::json!("baz"))]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 3);

    // Delete by filter
    let delete_result = ns
        .write(WriteParams {
            delete_by_filter: Some(Filter::eq("foo", "baz")),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(delete_result.rows_affected, 2);

    // Verify deletion
    let results2 = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(results2.rows.len(), 1);
    assert_eq!(results2.rows[0].get("id").unwrap(), 1);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_namespaces_listing() {
    let client = setup();
    let prefix = test_prefix();

    // Create a test namespace to ensure there's at least one to list
    let test_ns = client.namespace(format!("{}ns_listing_test", prefix));
    test_ns
        .write(WriteParams {
            upsert_rows: Some(vec![row(1, vec![0.1, 0.1], vec![])]),
            distance_metric: Some(DistanceMetric::CosineDistance),
            ..Default::default()
        })
        .await
        .unwrap();

    let namespaces0 = client
        .namespaces(NamespacesParams {
            page_size: Some(5),
            ..Default::default()
        })
        .await
        .unwrap();

    // Verify we got some namespaces (up to page_size)
    assert!(!namespaces0.namespaces.is_empty());
    assert!(namespaces0.namespaces.len() <= 5);

    // If there are more namespaces, test pagination
    if let Some(cursor0) = namespaces0.next_cursor.clone() {
        let namespaces1 = client
            .namespaces(NamespacesParams {
                cursor: Some(cursor0.clone()),
                page_size: Some(5),
                ..Default::default()
            })
            .await
            .unwrap();

        assert!(namespaces1.namespaces.len() <= 5);
        // Cursor should change between pages
        assert_ne!(Some(cursor0), namespaces1.next_cursor);
    }

    // Cleanup
    test_ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_hint_cache_warm() {
    let client = setup();
    let prefix = test_prefix();

    // Create a test namespace
    let ns = client.namespace(format!("{}hint_cache_warm", prefix));
    let _ = ns.delete_all().await;

    ns.write(WriteParams {
        upsert_rows: Some(vec![row(1, vec![0.1, 0.1], vec![])]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    let result = ns.hint_cache_warm().await.unwrap();
    assert!(result.status == "ACCEPTED" || result.status == "OK");

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_schema_and_metadata() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}schema", prefix));

    let _ = ns.delete_all().await;

    let mut schema = HashMap::new();
    schema.insert(
        "title".to_string(),
        serde_json::json!({
            "type": "string",
            "full_text_search": {
                "stemming": true,
                "remove_stopwords": true,
                "case_sensitive": false
            }
        }),
    );
    schema.insert(
        "vector".to_string(),
        serde_json::json!({
            "type": "[2]f16",
            "ann": true
        }),
    );

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![
                ("title", serde_json::json!("one")),
                ("private", serde_json::json!(true)),
            ]),
            row(2, vec![0.2, 0.2], vec![
                ("title", serde_json::Value::Null),
                ("private", serde_json::Value::Null),
            ]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        schema: Some(schema),
        ..Default::default()
    })
    .await
    .unwrap();

    let schema_resp = ns.schema().await.unwrap();
    assert!(schema_resp.0.contains_key("title"));
    assert!(schema_resp.0.contains_key("id"));

    let metadata = ns.metadata().await.unwrap();
    // Verify metadata structure - created_at should be present for existing namespaces
    assert!(
        metadata.created_at.is_some(),
        "Expected created_at to be present, got metadata: {:?}",
        metadata
    );

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_bm25_with_sum_query() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}bm25_sum", prefix));

    let _ = ns.delete_all().await;

    let mut schema = HashMap::new();
    schema.insert(
        "text".to_string(),
        serde_json::json!({
            "type": "string",
            "full_text_search": {
                "language": "english",
                "stemming": true,
                "case_sensitive": false,
                "remove_stopwords": true
            }
        }),
    );

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![
                ("text", serde_json::json!("Walruses are large marine mammals with long tusks and whiskers")),
            ]),
            row(2, vec![0.2, 0.2], vec![
                ("text", serde_json::json!("They primarily inhabit the cold Arctic regions")),
            ]),
            row(3, vec![0.3, 0.3], vec![
                ("text", serde_json::json!("Walruses use their tusks to help haul themselves onto ice")),
            ]),
            row(4, vec![0.4, 0.4], vec![
                ("text", serde_json::json!("Their diet mainly consists of mollusks and other sea creatures")),
            ]),
            row(5, vec![0.5, 0.5], vec![
                ("text", serde_json::json!("Walrus populations are affected by climate change and melting ice")),
            ]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        schema: Some(schema),
        ..Default::default()
    })
    .await
    .unwrap();

    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::sum(vec![
                RankBy::bm25("text", "large tusk"),
                RankBy::bm25("text", "mollusk diet"),
            ])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 3);
    assert_eq!(results.rows[0].get("id").unwrap(), 4);
    assert_eq!(results.rows[1].get("id").unwrap(), 1);
    assert_eq!(results.rows[2].get("id").unwrap(), 3);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_not_filter() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}not_filter", prefix));

    let _ = ns.delete_all().await;

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![("status", serde_json::json!("active"))]),
            row(2, vec![0.2, 0.2], vec![("status", serde_json::json!("deleted"))]),
            row(3, vec![0.3, 0.3], vec![("status", serde_json::json!("active"))]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    // Find all that are NOT deleted
    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            filters: Some(Filter::not(Filter::eq("status", "deleted"))),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 2);
    let mut ids: Vec<i64> = results
        .rows
        .iter()
        .map(|r| r.get("id").unwrap().as_i64().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, vec![1, 3]);

    // Double negation
    let results2 = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            filters: Some(Filter::not(Filter::not(Filter::eq("status", "deleted")))),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results2.rows.len(), 1);
    assert_eq!(results2.rows[0].get("id").unwrap(), 2);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_patch() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}patch", prefix));

    let _ = ns.delete_all().await;

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![1.0, 1.0], vec![]),
            row(2, vec![2.0, 2.0], vec![]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    // Patch with rows
    let mut patch1 = HashMap::new();
    patch1.insert("id".to_string(), serde_json::json!(1));
    patch1.insert("a".to_string(), serde_json::json!(1));

    let mut patch2 = HashMap::new();
    patch2.insert("id".to_string(), serde_json::json!(2));
    patch2.insert("b".to_string(), serde_json::json!(2));

    ns.write(WriteParams {
        patch_rows: Some(vec![patch1, patch2]),
        ..Default::default()
    })
    .await
    .unwrap();

    // Patch again
    let mut patch3 = HashMap::new();
    patch3.insert("id".to_string(), serde_json::json!(1));
    patch3.insert("b".to_string(), serde_json::json!(1));

    let mut patch4 = HashMap::new();
    patch4.insert("id".to_string(), serde_json::json!(2));
    patch4.insert("a".to_string(), serde_json::json!(2));

    ns.write(WriteParams {
        patch_rows: Some(vec![patch3, patch4]),
        ..Default::default()
    })
    .await
    .unwrap();

    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            include_attributes: Some(IncludeAttributes::List(vec![
                "id".to_string(),
                "a".to_string(),
                "b".to_string(),
            ])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 2);
    assert_eq!(results.rows[0].get("id").unwrap(), 1);
    assert_eq!(results.rows[0].get("a").unwrap(), 1);
    assert_eq!(results.rows[0].get("b").unwrap(), 1);
    assert_eq!(results.rows[1].get("id").unwrap(), 2);
    assert_eq!(results.rows[1].get("a").unwrap(), 2);
    assert_eq!(results.rows[1].get("b").unwrap(), 2);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_product_operator() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}product_operator", prefix));

    let _ = ns.delete_all().await;

    let mut schema = HashMap::new();
    schema.insert(
        "title".to_string(),
        serde_json::json!({
            "type": "string",
            "full_text_search": true
        }),
    );
    schema.insert(
        "content".to_string(),
        serde_json::json!({
            "type": "string",
            "full_text_search": true
        }),
    );

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![
                ("title", serde_json::json!("one")),
                ("content", serde_json::json!("foo bar baz")),
            ]),
            row(2, vec![0.2, 0.2], vec![
                ("title", serde_json::json!("two")),
                ("content", serde_json::json!("foo bar")),
            ]),
            row(3, vec![0.3, 0.3], vec![
                ("title", serde_json::json!("three")),
                ("content", serde_json::json!("bar baz")),
            ]),
        ]),
        distance_metric: Some(DistanceMetric::EuclideanSquared),
        schema: Some(schema),
        ..Default::default()
    })
    .await
    .unwrap();

    // Test Product with weight first
    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::product(2.0, RankBy::bm25("title", "one"))),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(!results.rows.is_empty());

    // Test Sum with Product
    let results2 = ns
        .query(QueryParams {
            rank_by: Some(RankBy::sum(vec![
                RankBy::product(2.0, RankBy::bm25("title", "one")),
                RankBy::bm25("content", "foo"),
            ])),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(!results2.rows.is_empty());

    // Test nested Product with Max
    let results3 = ns
        .query(QueryParams {
            rank_by: Some(RankBy::product(
                2.0,
                RankBy::max(vec![
                    RankBy::product(2.0, RankBy::bm25("title", "one")),
                    RankBy::bm25("content", "foo"),
                ]),
            )),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(!results3.rows.is_empty());

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_empty_namespace_query() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}empty_ns", prefix));

    // Write then delete
    ns.write(WriteParams {
        upsert_rows: Some(vec![row(1, vec![0.1, 0.1], vec![])]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    ns.write(WriteParams {
        deletes: Some(vec![serde_json::json!(1)]),
        ..Default::default()
    })
    .await
    .unwrap();

    // Query empty namespace - should not error
    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::asc("id")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 0);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_exists() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}exists", prefix));

    // Clean up if exists
    let _ = ns.delete_all().await;

    // Verify namespace doesn't exist yet
    assert!(!ns.exists().await.unwrap());

    // Create namespace
    ns.write(WriteParams {
        upsert_rows: Some(vec![row(1, vec![0.1, 0.1], vec![("private", serde_json::json!(true))])]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    // Verify namespace exists now
    assert!(ns.exists().await.unwrap());

    // Cleanup
    ns.delete_all().await.unwrap();

    // Verify namespace doesn't exist anymore
    assert!(!ns.exists().await.unwrap());
}

#[tokio::test]
#[serial]
async fn test_copy_from_namespace() {
    let client = setup();
    let prefix = test_prefix();
    let ns1 = client.namespace(format!("{}copy_from_1", prefix));
    let ns2 = client.namespace(format!("{}copy_from_2", prefix));

    let _ = ns1.delete_all().await;
    let _ = ns2.delete_all().await;

    // Create source namespace with data
    ns1.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![("tags", serde_json::json!(["a"]))]),
            row(2, vec![0.2, 0.2], vec![("tags", serde_json::json!(["b"]))]),
            row(3, vec![0.3, 0.3], vec![("tags", serde_json::json!(["c"]))]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        ..Default::default()
    })
    .await
    .unwrap();

    // Copy to second namespace
    ns2.write(WriteParams {
        copy_from_namespace: Some(ns1.name().to_string()),
        ..Default::default()
    })
    .await
    .unwrap();

    // Verify data was copied
    let results = ns2
        .query(QueryParams {
            rank_by: Some(RankBy::vector("vector", vec![0.1, 0.1])),
            include_attributes: Some(IncludeAttributes::All(true)),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 3);

    // Cleanup
    ns1.delete_all().await.unwrap();
    ns2.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_bm25_with_default_schema() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}bm25_default_schema", prefix));

    let _ = ns.delete_all().await;

    let mut schema = HashMap::new();
    schema.insert(
        "text".to_string(),
        serde_json::json!({
            "type": "string",
            "full_text_search": true
        }),
    );

    ns.write(WriteParams {
        upsert_rows: Some(vec![
            row(1, vec![0.1, 0.1], vec![
                ("text", serde_json::json!("Walruses can produce a variety of funny sounds, including whistles, grunts, and bell-like noises.")),
            ]),
            row(2, vec![0.2, 0.2], vec![
                ("text", serde_json::json!("They sometimes use their tusks as a tool to break through ice or to scratch their bodies.")),
            ]),
        ]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        schema: Some(schema),
        ..Default::default()
    })
    .await
    .unwrap();

    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::bm25("text", "scratch")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.rows.len(), 1);
    assert_eq!(results.rows[0].get("id").unwrap(), 2);

    ns.delete_all().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_contains_all_tokens() {
    let client = setup();
    let prefix = test_prefix();
    let ns = client.namespace(format!("{}contains_all_tokens", prefix));

    let _ = ns.delete_all().await;

    let mut schema = HashMap::new();
    schema.insert(
        "text".to_string(),
        serde_json::json!({
            "type": "string",
            "full_text_search": {
                "stemming": true
            }
        }),
    );

    ns.write(WriteParams {
        upsert_rows: Some(vec![row(
            1,
            vec![0.1, 0.1],
            vec![("text", serde_json::json!("Walruses are large marine mammals with long tusks and whiskers"))],
        )]),
        distance_metric: Some(DistanceMetric::CosineDistance),
        schema: Some(schema),
        ..Default::default()
    })
    .await
    .unwrap();

    // Should find the row
    let results = ns
        .query(QueryParams {
            rank_by: Some(RankBy::bm25("text", "walrus whisker")),
            filters: Some(Filter::contains_all_tokens("text", "marine mammals")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(results.rows.len(), 1);

    // Should not find the row - "short" is not in the text
    let missing = ns
        .query(QueryParams {
            rank_by: Some(RankBy::bm25("text", "walrus whisker")),
            filters: Some(Filter::contains_all_tokens("text", "marine mammals short")),
            top_k: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(missing.rows.len(), 0);

    ns.delete_all().await.unwrap();
}

/// Cleanup test that deletes all ephemeral test namespaces with the `rust_sdk_` prefix.
/// This helps clean up any orphaned namespaces from failed test runs.
/// Marked as serial to run after all other tests complete.
#[tokio::test]
#[serial]
async fn test_zz_cleanup_ephemeral_namespaces() {
    let client = setup();

    let mut cursor: Option<String> = None;
    let mut deleted_count = 0;

    loop {
        let namespaces = client
            .namespaces(NamespacesParams {
                prefix: Some("rust_sdk_".to_string()),
                cursor: cursor.clone(),
                page_size: Some(100),
            })
            .await
            .unwrap();

        for ns_summary in &namespaces.namespaces {
            if ns_summary.id.starts_with("rust_sdk_") {
                let ns = client.namespace(&ns_summary.id);
                if ns.delete_all().await.is_ok() {
                    deleted_count += 1;
                }
            }
        }

        cursor = namespaces.next_cursor;
        if cursor.is_none() || namespaces.namespaces.is_empty() {
            break;
        }
    }

    if deleted_count > 0 {
        println!("Cleaned up {} ephemeral test namespaces", deleted_count);
    }
}
