use buscador_archivos::algorithms::avl::{AvlNode, AvlSearcher};
use buscador_archivos::algorithms::Searcher;
use buscador_archivos::models::{FileEntry, QueryType};

fn verify_avl_invariants(node_opt: &Option<Box<AvlNode>>) -> (bool, i32) {
    match node_opt {
        None => (true, 0),
        Some(node) => {
            let (left_valid, left_h) = verify_avl_invariants(&node.left);
            let (right_valid, right_h) = verify_avl_invariants(&node.right);

            let bf = left_h - right_h;
            let h = 1 + left_h.max(right_h);

            let is_balanced = bf.abs() <= 1;
            let height_correct = node.height == h;

            (left_valid && right_valid && is_balanced && height_correct, h)
        }
    }
}

#[test]
fn test_avl_rotations_ll() {
    let mut root = None;
    root = Some(AvlNode::insert(root, "c".to_string(), 0));
    root = Some(AvlNode::insert(root, "b".to_string(), 1));
    root = Some(AvlNode::insert(root, "a".to_string(), 2));

    let (valid, height) = verify_avl_invariants(&root);
    assert!(valid);
    assert_eq!(height, 2);
    let r = root.unwrap();
    assert_eq!(r.key, "b");
    assert_eq!(r.left.as_ref().unwrap().key, "a");
    assert_eq!(r.right.as_ref().unwrap().key, "c");
}

#[test]
fn test_avl_rotations_rr() {
    let mut root = None;
    root = Some(AvlNode::insert(root, "a".to_string(), 0));
    root = Some(AvlNode::insert(root, "b".to_string(), 1));
    root = Some(AvlNode::insert(root, "c".to_string(), 2));

    let (valid, height) = verify_avl_invariants(&root);
    assert!(valid);
    assert_eq!(height, 2);
    let r = root.unwrap();
    assert_eq!(r.key, "b");
}

#[test]
fn test_avl_rotations_lr() {
    let mut root = None;
    root = Some(AvlNode::insert(root, "c".to_string(), 0));
    root = Some(AvlNode::insert(root, "a".to_string(), 1));
    root = Some(AvlNode::insert(root, "b".to_string(), 2));

    let (valid, height) = verify_avl_invariants(&root);
    assert!(valid);
    assert_eq!(height, 2);
    let r = root.unwrap();
    assert_eq!(r.key, "b");
}

#[test]
fn test_avl_rotations_rl() {
    let mut root = None;
    root = Some(AvlNode::insert(root, "a".to_string(), 0));
    root = Some(AvlNode::insert(root, "c".to_string(), 1));
    root = Some(AvlNode::insert(root, "b".to_string(), 2));

    let (valid, height) = verify_avl_invariants(&root);
    assert!(valid);
    assert_eq!(height, 2);
    let r = root.unwrap();
    assert_eq!(r.key, "b");
}

#[test]
fn test_avl_bulk_invariants() {
    let mut searcher = AvlSearcher::new();
    let words = vec![
        "main.rs", "lib.rs", "config.rs", "models.rs", "avl.rs", "bfs.rs", "dfs.rs",
        "linear.rs", "binary.rs", "hash.rs", "cargo.toml", "readme.md",
    ];

    let files: Vec<FileEntry> = words
        .iter()
        .enumerate()
        .map(|(i, &w)| FileEntry {
            id: i,
            name: w.to_string(),
            name_lower: w.to_string(),
            path: format!("/project/{}", w),
            size: 100,
            modified: 1000,
            is_dir: false,
        })
        .collect();

    searcher.build(&files);

    let (valid, _) = verify_avl_invariants(&searcher.root);
    assert!(valid);

    // Búsqueda exacta
    let exact_res = searcher.search("avl.rs", QueryType::Exact, &files, 10);
    assert_eq!(exact_res.len(), 1);
    assert_eq!(exact_res[0].name, "avl.rs");

    // Búsqueda prefijo
    let prefix_res = searcher.search("b", QueryType::Prefix, &files, 10);
    assert!(prefix_res.iter().any(|r| r.name == "bfs.rs"));
    assert!(prefix_res.iter().any(|r| r.name == "binary.rs"));
}

