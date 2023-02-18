use crate::treedb::TreeDBBuilder;

use super::{ChildSelector, DBValue, Hasher, Key, Node, NodeHash, Tree, TreeDB};

use hash256_std_hasher::Hash256StdHasher;
use hash_db::{AsHashDB, Prefix, EMPTY_PREFIX};
use memory_db::{KeyFunction, MemoryDB};
use sha3::{Digest, Sha3_256};
use std::marker::PhantomData;

// MOCK
// ================================================================================================

/// Unit struct for Sha3.
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

// TESTS
// ================================================================================================
#[test]
fn simple_inner_node_hash() {
    // hash flip and flop
    let flip = NodeHash::Hash::<Sha3>(Sha3::hash(b"flip"));
    let flop = NodeHash::Hash::<Sha3>(Sha3::hash(b"flop"));
    let node = Node::new_inner(flip, flop).unwrap();
    assert_eq!(
        node.hash(),
        &Sha3::hash(&[Sha3::hash(b"flip"), Sha3::hash(b"flop")].concat())
    );
}

// test a simple node update
#[test]
fn simple_node_update() {
    // hash flip and flop
    let flip = NodeHash::Hash::<Sha3>(Sha3::hash(b"flip"));
    let flop = NodeHash::Hash::<Sha3>(Sha3::hash(b"flop"));
    let mut node = Node::new_inner(flip, flop).unwrap();
    // update the left child
    let new_hash = Sha3::hash(b"new");
    node.set_child_hash(ChildSelector::Left, NodeHash::Hash(new_hash))
        .unwrap();
    // verify that the hash of the node has changed
    assert_eq!(
        node.hash(),
        &Sha3::hash(&[Sha3::hash(b"new"), Sha3::hash(b"flop")].concat())
    );
}

#[test]
fn tree_db_get_value() {
    const BYTE_DEPTH: usize = 1;
    let null_values = compute_null_hashes::<Sha3>(BYTE_DEPTH * 8);

    let mut db = MemoryDB::<Sha3, NoopKey<Sha3>, DBValue>::default();
    let hash_db = db.as_hash_db_mut();

    let value = b"test".to_vec();
    let mut current_node = Node::<Sha3>::new_value(&value);

    hash_db.emplace(
        *current_node.hash(),
        EMPTY_PREFIX,
        current_node.clone().into(),
    );

    for i in 0..(BYTE_DEPTH * 8) {
        current_node = Node::<Sha3>::new_inner(
            NodeHash::Hash::<Sha3>(*current_node.hash()),
            NodeHash::Default::<Sha3>(null_values[i]),
        )
        .unwrap();
        hash_db.emplace(
            *current_node.hash(),
            EMPTY_PREFIX,
            current_node.clone().into(),
        )
    }

    let tree = TreeDBBuilder::<BYTE_DEPTH, Sha3>::new(&db, current_node.hash()).build();

    let key = Key::<BYTE_DEPTH>::new(&[0]);
    assert_eq!(tree.get_value(&key).unwrap(), Some(value));

    let key = Key::<BYTE_DEPTH>::new(&[1]);
    assert_eq!(tree.get_value(&key).unwrap(), None);
}

pub fn compute_null_hashes<H: Hasher>(depth: usize) -> Vec<H::Out> {
    (0..depth + 1)
        .scan(H::hash(&[]), |null_hash, _| {
            let value = *null_hash;
            *null_hash = H::hash(&[null_hash.as_ref(), null_hash.as_ref()].concat());
            Some(value)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
