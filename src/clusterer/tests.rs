#[test]
fn test_simple_kmeans() {
    let v1 = vec![1.0, 0.0];
    let v2 = vec![0.9, 0.1];
    let v3 = vec![0.0, 1.0];
    let v4 = vec![0.1, 0.9];

    let res = kmeans(&vec![v1, v2, v3, v4], 2, 20, 42);

    assert_eq!(res.clusters.len(), 2);
}
