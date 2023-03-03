use crate::{IndexTreeDB, IndexTreeDBBuilder, TreeDBMut};

use super::{
    DBValue, Hasher, IndexTree, IndexTreeDBMut, IndexTreeDBMutBuilder, IndexTreeMut, KeyedTree,
    KeyedTreeMut, TreeDB, TreeDBBuilder, TreeDBMutBuilder,
};

use hash256_std_hasher::Hash256StdHasher;
use hash_db::Prefix;
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
const TREE_DEPTH: usize = 2;

/// Test Data
const TEST_DATA: [(u64, &[u8], &[u8]); 4] = [
    (0, &[0, 0], b"value1"),
    (100, &[0, 100], b"value2"),
    (200, &[0, 200], b"value3"),
    (300, &[1, 44], b"value4"),
];

/// Creates mock data for testing
fn mock_data() -> (
    MemoryDB<Sha3, NoopKey<Sha3>, DBValue>,
    <Sha3 as Hasher>::Out,
) {
    let mut root = Default::default();
    let mut db = MemoryDB::<Sha3, NoopKey<Sha3>, DBValue>::default();
    let mut tree = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
        .expect("failed to construct tree buidler")
        .build();

    for (_index, path, value) in TEST_DATA.iter() {
        tree.insert(path, value.to_vec()).unwrap();
    }

    tree.commit();

    (db, root)
}

// TEST ROOT
// ================================================================================================

macro_rules! test_root {
    ($name:ident, mut $tree:ident) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let mut tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();
            let actual_root = tree.root().clone();
            assert_eq!(&actual_root, &root);
        }
    };
    ($name:ident, $tree:ident) => {
        #[test]
        fn $name() {
            let (db, root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&db, &root).unwrap().build();
            assert_eq!(tree.root(), &root);
        }
    };
}

test_root!(test_root_tree_db, TreeDBBuilder);
test_root!(test_root_index_db, IndexTreeDBBuilder);
test_root!(test_root_tree_db_mut, mut TreeDBMutBuilder);
test_root!(test_root_index_db_mut, mut IndexTreeDBMutBuilder);

// TEST DEPTH
// ================================================================================================

macro_rules! test_depth {
    ($name:ident, mut $tree:ident) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();
            assert_eq!(tree.depth(), TREE_DEPTH * 8);
        }
    };
    ($name:ident, $tree:ident) => {
        #[test]
        fn $name() {
            let (db, root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&db, &root).unwrap().build();
            assert_eq!(tree.depth(), TREE_DEPTH * 8);
        }
    };
}

test_depth!(test_depth_tree_db, TreeDBBuilder);
test_depth!(test_depth_index_db, IndexTreeDBBuilder);
test_depth!(test_depth_tree_db_mut, mut TreeDBMutBuilder);
test_depth!(test_depth_index_db_mut, mut IndexTreeDBMutBuilder);

// TEST VALUE
// ================================================================================================
macro_rules! test_value {
    ($name:ident, mut $tree:ident, $selector:tt) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();

            for data in TEST_DATA.iter() {
                let actual_value = tree.value(&data.$selector).unwrap();
                assert_eq!(actual_value, Some(data.2.to_vec()));
            }
        }
    };
    ($name:ident, $tree:ident, $selector:tt) => {
        #[test]
        fn $name() {
            let (db, root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&db, &root).unwrap().build();

            for data in TEST_DATA.iter() {
                let actual_value = tree.value(&data.$selector).unwrap();
                assert_eq!(actual_value, Some(data.2.to_vec()));
            }
        }
    };
}

test_value!(test_value_tree_db, TreeDBBuilder, 1);
test_value!(test_value_index_db, IndexTreeDBBuilder, 0);
test_value!(test_value_tree_db_mut, mut TreeDBMutBuilder, 1);
test_value!(test_value_index_db_mut, mut IndexTreeDBMutBuilder, 0);

// TEST LEAF
// ================================================================================================
macro_rules! test_leaf {
    ($name:ident, mut $tree:ident, $selector:tt) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();

            for data in TEST_DATA.iter() {
                let actual_leaf = tree.leaf(&data.$selector).unwrap();
                assert_eq!(actual_leaf, Sha3::hash(&data.2).into());
            }
        }
    };
    ($name:ident, $tree:ident, $selector:tt) => {
        #[test]
        fn $name() {
            let (db, root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&db, &root).unwrap().build();

            for data in TEST_DATA.iter() {
                let actual_leaf = tree.leaf(&data.$selector).unwrap();
                assert_eq!(actual_leaf, Sha3::hash(&data.2).into());
            }
        }
    };
}

test_leaf!(test_leaf_tree_db, TreeDBBuilder, 1);
test_leaf!(test_leaf_index_db, IndexTreeDBBuilder, 0);
test_leaf!(test_leaf_tree_db_mut, mut TreeDBMutBuilder, 1);
test_leaf!(test_leaf_index_db_mut, mut IndexTreeDBMutBuilder, 0);

// TEST PROOF AND VERIFY
// ================================================================================================
macro_rules! test_proof {
    ($name:ident, mut $tree:ident, $selector:tt, $tree_interface:ident) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let mut tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();

            for data in TEST_DATA.iter() {
                let proof = tree.proof(&data.$selector).unwrap();
                let value = tree.value(&data.$selector).unwrap();

                let root = tree.root().clone();
                assert_eq!(
                    $tree_interface::<TREE_DEPTH, Sha3>::verify(
                        &data.$selector,
                        &value.unwrap(),
                        &proof.unwrap(),
                        &root
                    ),
                    Ok(true)
                );
            }
        }
    };
    ($name:ident, $tree:ident, $selector:tt, $tree_interface:ident) => {
        #[test]
        fn $name() {
            let (db, root) = mock_data();
            let tree = $tree::<TREE_DEPTH, Sha3>::new(&db, &root).unwrap().build();

            for data in TEST_DATA.iter() {
                let proof = tree.proof(&data.$selector).unwrap();
                let value = tree.value(&data.$selector).unwrap();

                assert_eq!(
                    $tree_interface::<TREE_DEPTH, Sha3>::verify(
                        &data.$selector,
                        &value.unwrap(),
                        &proof.unwrap(),
                        &root
                    ),
                    Ok(true)
                );
            }
        }
    };
}

test_proof!(test_proof_tree_db, TreeDBBuilder, 1, TreeDB);
test_proof!(test_proof_index_db, IndexTreeDBBuilder, 0, IndexTreeDB);
test_proof!(test_proof_tree_db_mut, mut TreeDBMutBuilder, 1, TreeDBMut);
test_proof!(
    test_proof_index_db_mut,
    mut IndexTreeDBMutBuilder,
    0,
    IndexTreeDBMut
);

// TEST INSERT
// ================================================================================================
macro_rules! test_insert {
    ($name:ident, mut $tree:ident, $selector:tt) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let mut tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();

            let new_value = b"new value";
            let new_leaf = Sha3::hash(new_value).into();
            let old_value = tree
                .insert(&TEST_DATA[0].$selector, new_value.to_vec())
                .unwrap();

            assert_eq!(old_value, Some(TEST_DATA[0].2.to_vec()));

            let actual_value = tree.value(&TEST_DATA[0].$selector).unwrap();
            assert_eq!(actual_value, Some(new_value.to_vec()));

            let actual_leaf = tree.leaf(&TEST_DATA[0].$selector).unwrap();
            assert_eq!(actual_leaf, new_leaf);
        }
    };
}

test_insert!(test_insert_tree_db_mut, mut TreeDBMutBuilder, 1);
test_insert!(test_insert_index_db_mut, mut IndexTreeDBMutBuilder, 0);

// TEST REMOVE
// ================================================================================================
macro_rules! test_remove {
    ($name:ident, mut $tree:ident, $selector:tt) => {
        #[test]
        fn $name() {
            let (mut db, mut root) = mock_data();
            let mut tree = $tree::<TREE_DEPTH, Sha3>::new(&mut db, &mut root)
                .unwrap()
                .build();

            let old_value = tree.remove(&TEST_DATA[0].$selector).unwrap();
            assert_eq!(old_value, Some(TEST_DATA[0].2.to_vec()));

            let actual_value = tree.value(&TEST_DATA[0].$selector).unwrap();
            assert_eq!(actual_value, None);

            let actual_leaf = tree.leaf(&TEST_DATA[0].$selector).unwrap();
            assert_eq!(actual_leaf, None);
        }
    };
}

test_remove!(test_remove_tree_db_mut, mut TreeDBMutBuilder, 1);
test_remove!(test_remove_index_db_mut, mut IndexTreeDBMutBuilder, 0);
