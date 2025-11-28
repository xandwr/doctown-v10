use crate::chunker::ChunkId;

#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: u32,
    pub chunk_ids: Vec<ChunkId>,
    pub centroid: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct ClusterResult {
    pub clusters: Vec<Cluster>,
    pub iterations: usize,
}
