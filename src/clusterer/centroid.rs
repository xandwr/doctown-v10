pub fn compute_centroid(vectors: &[&[f32]]) -> Vec<f32> {
    let dim = vectors[0].len();
    let mut out = vec![0.0; dim];

    for v in vectors {
        for i in 0..dim {
            out[i] += v[i];
        }
    }

    let n = vectors.len() as f32;
    for i in 0..dim {
        out[i] /= n;
    }

    out
}
