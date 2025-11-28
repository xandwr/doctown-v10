use super::*;

#[test]
fn test_batching_small() {
    let batcher = Batcher::new(3);
    let items = vec![
        "chunk1".to_string(),
        "chunk2".to_string(),
        "chunk3".to_string(),
        "chunk4".to_string(),
        "chunk5".to_string(),
    ];

    let batches = batcher.split(&items);
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].len(), 3);
    assert_eq!(batches[1].len(), 2);
}

#[test]
fn test_batching_exact_size() {
    let batcher = Batcher::new(5);
    let items = vec!["a".to_string(); 10];

    let batches = batcher.split(&items);
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].len(), 5);
    assert_eq!(batches[1].len(), 5);
}

#[test]
fn test_batching_empty() {
    let batcher = Batcher::new(100);
    let items: Vec<String> = vec![];

    let batches = batcher.split(&items);
    assert_eq!(batches.len(), 0);
}

#[test]
fn test_model_info_default() {
    let model = EmbeddingModelInfo::default();
    assert_eq!(model.name, "google/embeddinggemma-300m");
    assert_eq!(model.dim, 768);
    assert_eq!(model.max_batch, 32);
}

#[test]
fn test_model_info_custom() {
    let model = EmbeddingModelInfo::new("custom-model", 384, 64);
    assert_eq!(model.name, "custom-model");
    assert_eq!(model.dim, 384);
    assert_eq!(model.max_batch, 64);
}

#[tokio::test]
async fn test_client_empty_input() {
    let client = EmbeddingClient::new("http://localhost:18115");
    let result = client.embed(vec![]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

// Integration test - requires Python server running
#[tokio::test]
#[ignore]
async fn test_client_round_trip() {
    let client = EmbeddingClient::new("http://localhost:18115");
    let texts = vec![
        "This is a test sentence.".to_string(),
        "Another test sentence here.".to_string(),
    ];

    let result = client.embed(texts.clone()).await;
    assert!(result.is_ok());

    let embeddings = result.unwrap();
    assert_eq!(embeddings.len(), 2);

    // Each embedding should have 768 dimensions (for gemma-300m)
    assert_eq!(embeddings[0].len(), 768);
    assert_eq!(embeddings[1].len(), 768);

    // Vectors should be normalized (L2 norm ~= 1.0)
    let norm: f32 = embeddings[0].iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01, "Vector should be normalized");
}

// Integration test - test batching with client
#[tokio::test]
#[ignore]
async fn test_batched_embedding() {
    let client = EmbeddingClient::new("http://localhost:18115");
    let batcher = Batcher::new(2);

    let texts: Vec<String> = (0..5)
        .map(|i| format!("Test sentence number {}", i))
        .collect();

    let mut all_embeddings = Vec::new();

    for batch in batcher.split(&texts) {
        let batch_texts = batch.iter().map(|s| s.to_string()).collect();
        let embeddings = client.embed(batch_texts).await.unwrap();
        all_embeddings.extend(embeddings);
    }

    assert_eq!(all_embeddings.len(), 5);
    assert_eq!(all_embeddings[0].len(), 768);
}