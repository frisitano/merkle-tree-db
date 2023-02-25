use super::{
    ChildSelector, DBValue, Hasher, KeyedTree, KeyedTreeMut, Node, NodeHash, Recorder, TreeDB,
    TreeDBBuilder, TreeDBMutBuilder,
};

use hash256_std_hasher::Hash256StdHasher;
use hash_db::{AsHashDB, Prefix, EMPTY_PREFIX};
use memory_db::{KeyFunction, MemoryDB};
use sha3::{Digest, Sha3_256};
use std::marker::PhantomData;

// MOCK
// ================================================================================================

/// Unit struct for Sha3.
#[derive(Debug)]
pub struct Sha3;

/// implementation of the Hasher trait for the Sha3 hasher
/// This is used for testing
impl Hasher for Sha3 {
    type Out = [u8; 32];

    type StdHasher = Hash256StdHasher;

    const LENGTH: usize = 32;

    fn hash(data: &[u8]) -> Self::Out {
        Sha3_256::digest(data).into()
    }
}

/// Unit struct for NoopKey
pub struct NoopKey<H: Hasher>(PhantomData<H>);

/// implementation of the KeyFunction trait for the NoopKey hasher. This is used for testing, the key is
/// the hash provided.  The prefix is ignored.
impl<H: Hasher> KeyFunction<H> for NoopKey<H> {
    type Key = Vec<u8>;

    fn key(hash: &H::Out, _prefix: Prefix) -> Vec<u8> {
        hash.as_ref().to_vec()
    }
}

/// Depth of tree
const TREE_DEPTH: usize = 1;

/// Test value
const TEST_VALUE: [u8; 4] = *b"test";

/// Creates mock data for testing
fn mock_data() -> (
    MemoryDB<Sha3, NoopKey<Sha3>, DBValue>,
    <Sha3 as Hasher>::Out,
) {
    let null_values = null_hashes::<Sha3>(TREE_DEPTH * 8);

    let mut db = MemoryDB::<Sha3, NoopKey<Sha3>, DBValue>::default();
    let hash_db = db.as_hash_db_mut();

    let mut current_node = Node::<Sha3>::new_value(&TEST_VALUE);

    hash_db.emplace(
        *current_node.hash(),
        EMPTY_PREFIX,
        current_node.clone().into(),
    );

    for i in 0..(TREE_DEPTH * 8) {
        current_node = Node::<Sha3>::new_inner(
            NodeHash::Database::<Sha3>(*current_node.hash()),
            NodeHash::Default::<Sha3>(null_values[i]),
        )
        .unwrap();
        hash_db.emplace(
            *current_node.hash(),
            EMPTY_PREFIX,
            current_node.clone().into(),
        )
    }

    let root = *current_node.hash();
    (db, root)
}

/// Return the null hashes of a tree of depth D
pub fn null_hashes<H: Hasher>(depth: usize) -> Vec<H::Out> {
    let mut hashes = Vec::with_capacity(depth);
    hashes.push(H::hash(&[]));
    for i in 1..depth {
        let hash = H::hash(&[hashes[i - 1].as_ref(), hashes[i - 1].as_ref()].concat());
        hashes.push(hash);
    }
    hashes
}

// TESTS
// ================================================================================================
#[test]
fn simple_inner_node_hash() {
    // hash flip and flop
    let flip = NodeHash::Database::<Sha3>(Sha3::hash(b"flip"));
    let flop = NodeHash::Database::<Sha3>(Sha3::hash(b"flop"));
    let node = Node::new_inner(flip, flop).unwrap();
    assert_eq!(
        node.hash().as_ref(),
        &Sha3::hash(&[Sha3::hash(b"flip"), Sha3::hash(b"flop")].concat())
    );
}

// test a simple node update
#[test]
fn simple_node_update() {
    // hash flip and flop
    let flip = NodeHash::Database::<Sha3>(Sha3::hash(b"flip"));
    let flop = NodeHash::Database::<Sha3>(Sha3::hash(b"flop"));
    let mut node = Node::new_inner(flip, flop).unwrap();
    // update the left child
    let new_hash = Sha3::hash(b"new");
    node.set_child_hash(&ChildSelector::Left, NodeHash::Database(new_hash))
        .unwrap();
    // verify that the hash of the node has changed
    assert_eq!(
        node.hash().as_ref(),
        &Sha3::hash(&[Sha3::hash(b"new"), Sha3::hash(b"flop")].concat())
    );
}

#[test]
fn tree_db_get_value() {
    let (db, root) = mock_data();
    let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&db, &root)
        .expect("valid tree depth provided")
        .build();

    assert_eq!(tree.value(&[0]).unwrap(), Some(TEST_VALUE.to_vec()));
    assert_eq!(tree.value(&[1]).unwrap(), None);
}

#[test]
fn tree_db_proof() {
    let (db, root) = mock_data();
    let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&db, &root)
        .expect("valid tree depth provided")
        .build();

    let proof = tree.proof(&[0]).unwrap().unwrap();

    assert_eq!(proof.len(), TREE_DEPTH * 8);

    assert!(TreeDB::<1, Sha3>::verify(&[0], &TEST_VALUE, &proof, &root).unwrap());
    assert!(!TreeDB::<1, Sha3>::verify(&[0], b"incorrect_value", &proof, &root).unwrap());

    let non_inclusion_proof = tree.proof(&[1]).unwrap().unwrap();
    assert!(TreeDB::<1, Sha3>::verify(&[1], &[], &non_inclusion_proof, &root).unwrap());
}

#[test]
fn tree_depth() {
    let (db, root) = mock_data();
    let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&db, &root)
        .expect("valid tree depth provided")
        .build();

    assert_eq!(tree.depth(), TREE_DEPTH * 8);
}

#[test]
fn tree_db_mut_insert() {
    let (mut db, mut root) = mock_data();
    let mut tree_db_mut = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
        .expect("valid tree depth provided")
        .build();

    let new_value = Sha3::hash(b"new").to_vec();

    let old_value = tree_db_mut.insert(&[0], new_value.clone()).unwrap();

    assert_eq!(tree_db_mut.value(&[0]).unwrap(), Some(new_value));
    assert_eq!(old_value, Some(TEST_VALUE.to_vec()));
}

#[test]
fn tree_db_mut_remove() {
    let (mut db, mut root) = mock_data();
    let mut tree_db_mut = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
        .expect("valid tree depth provided")
        .build();

    let old_value = tree_db_mut.remove(&[0]).unwrap();

    assert_eq!(tree_db_mut.value(&[0]).unwrap(), None);
    assert_eq!(old_value, Some(TEST_VALUE.to_vec()));

    let old_value = tree_db_mut.remove(&[122]).unwrap();
    assert_eq!(old_value, None);
}

#[test]
fn tree_db_mut_test_commit() {
    let (mut db, mut root) = mock_data();
    let mut tree_db_mut = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
        .expect("valid tree depth provided")
        .build();

    let new_value = b"new_value".to_vec();

    let old_value = tree_db_mut.insert(&[10], new_value.clone()).unwrap();

    assert_eq!(tree_db_mut.value(&[10]).unwrap(), Some(new_value));
    assert_eq!(old_value, None);

    let new_value = b"new1".to_vec();

    let old_value = tree_db_mut.insert(&[0], new_value.clone()).unwrap();

    assert_eq!(tree_db_mut.value(&[0]).unwrap(), Some(new_value));
    assert_eq!(old_value, Some(TEST_VALUE.to_vec()));

    tree_db_mut.commit();

    let value = tree_db_mut.value(&[10]).unwrap();

    assert_eq!(value, Some(b"new_value".to_vec()));
}

#[test]
fn tree_db_mut_test_proof() {
    let (mut db, mut root) = mock_data();
    let mut recorder = Recorder::<Sha3>::new();
    let tree_db_mut = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
        .expect("valid tree depth provided")
        .with_recorder(&mut recorder)
        .build();

    let value = tree_db_mut.value(&[0]).unwrap();
    assert_eq!(value, Some(TEST_VALUE.to_vec()));

    let storage_proof = recorder.drain_storage_proof();
    let mem_db = storage_proof.into_memory_db::<Sha3>();
    let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&mem_db, &root)
        .expect("valid tree depth provided")
        .build();
    assert_eq!(tree.value(&[0]).unwrap(), Some(TEST_VALUE.to_vec()));
}
