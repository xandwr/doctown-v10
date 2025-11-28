use crate::clusterer::{
    centroid::compute_centroid,
    similarity::cosine_distance,
    types::{Cluster, ClusterResult},
};

pub fn kmeans(embeddings: &[Vec<f32>], k: usize, max_iters: usize, seed: u64) -> ClusterResult {
    use rand::{SeedableRng, seq::SliceRandom};
    use rand_chacha::ChaCha8Rng;

    let n = embeddings.len();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // 1. Pick random initial centers
    let mut centroids: Vec<Vec<f32>> = embeddings.choose_multiple(&mut rng, k).cloned().collect();

    let mut assignments = vec![0usize; n];
    let mut iterations = 0;

    for _ in 0..max_iters {
        iterations += 1;

        // 2. Assign each vector to nearest centroid
        let mut changed = false;
        for i in 0..n {
            let best = centroids
                .iter()
                .enumerate()
                .map(|(c, center)| (c, cosine_distance(&embeddings[i], center)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap()
                .0;

            if assignments[i] != best {
                changed = true;
                assignments[i] = best;
            }
        }

        if !changed {
            break; // converged
        }

        // 3. Recompute centroids
        for c in 0..k {
            let members: Vec<&[f32]> = embeddings
                .iter()
                .zip(assignments.iter())
                .filter(|&(_, a)| *a == c)
                .map(|(e, _)| &e[..])
                .collect();

            if !members.is_empty() {
                centroids[c] = compute_centroid(&members);
            }
        }
    }

    // 4. Build result clusters
    let mut clusters = vec![
        Cluster {
            id: 0,
            chunk_ids: vec![],
            centroid: vec![]
        };
        k
    ];

    for i in 0..k {
        clusters[i].id = i as u32;
        clusters[i].centroid = centroids[i].clone();
    }

    for (chunk_id, &cluster_idx) in assignments.iter().enumerate() {
        clusters[cluster_idx].chunk_ids.push(chunk_id as u32);
    }

    ClusterResult {
        clusters,
        iterations,
    }
}
