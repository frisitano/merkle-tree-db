use hash256_std_hasher::Hash256StdHasher;
use hash_db::Prefix;
use memory_db::{KeyFunction, MemoryDB};
use merkle_tree_db::{Hasher, IndexTree, IndexTreeDBBuilder, IndexTreeDBMutBuilder, IndexTreeMut};
use sha3::{Digest, Sha3_256};
use std::marker::PhantomData;

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

fn main() {
    // create an empty in memory database
    let mut memory_db = MemoryDB::<Sha3, NoopKey<_>, Vec<u8>>::default();

    // specify the tree depth - the actual depth will be 8 * TREE_DEPTH
    const TREE_DEPTH: usize = 1;

    // create a new default root
    let mut root = Default::default();

    // create a new mutable keyed tree with the specified depth
    let mut tree = IndexTreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut memory_db, &mut root)
        .expect("failed to create tree")
        .build();

    // define some dummy data
    let data = vec![
        (0u64, b"flip".to_vec()),
        (2u64, b"flop".to_vec()),
        (8u64, b"flap".to_vec()),
        (9u64, b"flup".to_vec()),
    ];

    // insert the data into the tree
    for (key, value) in data {
        tree.insert(&key, value).expect("failed to insert data");
    }

    // commit the changes to the database
    tree.commit();

    // print the root hash
    println!("root hash: {:?}", tree.root());

    // delete some data from the tree
    tree.remove(&0).expect("failed to delete data");
    tree.remove(&9).expect("failed to delete data");

    // commit the changes to the database
    tree.commit();

    // print the root hash
    println!("root hash: {:?}", tree.root());

    // lets now create an immutable keyed tree using the same database and root
    let tree = IndexTreeDBBuilder::<TREE_DEPTH, Sha3>::new(&memory_db, &root)
        .expect("failed to create tree")
        .build();

    // lets now get the data we inserted
    let data_at_key_0 = tree.value(&0).expect("failed to get data");
    let data_at_key_2 = tree.value(&2).expect("failed to get data");
    let data_at_key_8 = tree.value(&8).expect("failed to get data");
    let data_at_key_9 = tree.value(&9).expect("failed to get data");

    // define a utility function to print the data
    fn print_data(data: Option<Vec<u8>>) {
        match data {
            Some(data) => println!("data: {:?}", std::str::from_utf8(&data).unwrap()),
            None => println!("data: None"),
        }
    }

    // print the data
    print_data(data_at_key_0);
    print_data(data_at_key_2);
    print_data(data_at_key_8);
    print_data(data_at_key_9);
}
