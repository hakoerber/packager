use base64::Engine as _;
use sha2::{Digest, Sha256};

pub struct QueryClassification<Component>
where
    Component: ToString,
{
    pub query_type: crate::QueryType,
    pub component: Component,
}

pub fn track_query<Component>(
    classification: &QueryClassification<Component>,
    query: &str,
    labels: &[(&'static str, String)],
) where
    Component: ToString,
{
    let query_id = {
        let mut hasher = Sha256::new();
        hasher.update(query);
        hasher.finalize()
    };

    // 9 bytes is enough to be unique
    // If this is divisible by 3, it means that we can base64-encode it without
    // any "=" padding
    //
    // cannot panic, as the output for sha256 will always be bit
    let query_id = &query_id[..9];

    let query_id = base64::engine::general_purpose::STANDARD.encode(query_id);
    let mut labels = Vec::from(labels);
    labels.extend_from_slice(&[
        ("query_id", query_id),
        ("query_type", classification.query_type.to_string()),
        ("query_component", classification.component.to_string()),
    ]);
    metrics::counter!("packager_database_queries_total", &labels).increment(1);
}

pub fn track_query_file<Component>(
    classification: &QueryClassification<Component>,
    path: &str,
    labels: &[(&'static str, String)],
) where
    Component: ToString,
{
    let query_id = {
        let mut hasher = Sha256::new();
        hasher.update(path);
        hasher.finalize()
    };

    // 9 bytes is enough to be unique
    // If this is divisible by 3, it means that we can base64-encode it without
    // any "=" padding
    //
    // cannot panic, as the output for sha256 will always be bit
    let query_id = &query_id[..9];

    let query_id = base64::engine::general_purpose::STANDARD.encode(query_id);
    let mut labels = Vec::from(labels);
    labels.extend_from_slice(&[
        ("query_id", query_id),
        ("query_type", classification.query_type.to_string()),
        ("query_component", classification.component.to_string()),
    ]);
    metrics::counter!("packager_database_queries_total", &labels).increment(1);
}
