# Merkle Tree DB

merkle-tree-db is a highly flexible library for working with merkle trees. It supports persistance 
over any key-value database backend. The merkle tree data structures are stored as sparse merkle trees
allowing for efficient lookups, updates and persistence. The library is generic over the hasher used 
and the depth of the tree. Sparse merkle trees that leverage circuit friendly hash functions 
(e.g [Poseidon](https://eprint.iacr.org/2019/458.pdf), [Rescue-Prime](https://eprint.iacr.org/2020/1143))
are performant in a ZKP setting and as such this library can serve this purpose. This library supports
both indexed merkle trees (max depth 64) and keyed (addressable) merkle trees (max depth `usize::MAX`).

## Tree interfaces

### Keyed Merkle Tree

The library supports two interfaces for keyed merkle trees, one mutable and the other immutable.  The 
interfaces are shown below.

```rust
/// A immutable key-value datastore implemented as a database-backed sparse merkle tree.
pub trait KeyedTree<H: Hasher, const D: usize> {
    /// Returns the root of the tree.
    fn root(&self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        D * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, key: &[u8]) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, key: &[u8]) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.  
    fn proof(&self, key: &[u8]) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        key: &[u8],
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}

/// A mutable key-value datastore implemented as a database-backed sparse merkle tree.
pub trait KeyedTreeMut<H: Hasher, const D: usize> {
    /// Returns the root of the tree.
    fn root(&mut self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        D * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, key: &[u8]) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, key: &[u8]) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.
    fn proof(&self, key: &[u8]) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Inserts a value at the provided key.
    fn insert(&mut self, key: &[u8], value: DBValue) -> Result<Option<DBValue>, TreeError>;

    /// Removes a value at the provided key.
    fn remove(&mut self, key: &[u8]) -> Result<Option<DBValue>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        key: &[u8],
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}
```

### Indexed Merkle Tree

The library supports two interfaces for indexed merkle trees, one mutable and the other immutable. 
These are implemented as wrappers around the keyed variants. The index (u64) is converted to a byte
slice before being passed to the keyed interface. The interfaces are shown below.

```rust
/// A immutable index-value datastore implemented as a database-backed sparse merkle tree.
pub trait IndexTree<H: Hasher, const D: usize> {
    /// Returns the root of the tree.
    fn root(&self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        D * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, index: &u64) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, index: &u64) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.  
    fn proof(&self, index: &u64) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        index: &u64,
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}

/// A mutable index-value datastore implemented as a database-backed sparse merkle tree.
pub trait IndexTreeMut<H: Hasher, const D: usize> {
    /// Returns the root of the tree.
    fn root(&mut self) -> &H::Out;

    /// Returns the depth of the tree.
    fn depth(&self) -> usize {
        D * 8
    }

    /// Returns the value at the provided key.
    fn value(&self, index: &u64) -> Result<Option<DBValue>, TreeError>;

    /// Returns the leaf at the provided key.
    fn leaf(&self, index: &u64) -> Result<Option<H::Out>, TreeError>;

    /// Returns an inclusion proof of a value a the specified key.
    fn proof(&self, index: &u64) -> Result<Option<Vec<DBValue>>, TreeError>;

    /// Inserts a value at the provided key.
    fn insert(&mut self, index: &u64, value: DBValue) -> Result<Option<DBValue>, TreeError>;

    /// Removes a value at the provided key.
    fn remove(&mut self, index: &u64) -> Result<Option<DBValue>, TreeError>;

    /// Verifies an inclusion proof of a value at the specified key.
    fn verify(
        index: &u64,
        value: &[u8],
        proof: &[DBValue],
        root: &H::Out,
    ) -> Result<bool, TreeError>;
}
```

## Recorder and Storage proofs

This library provides a `Recorder` which can record database reads across transactions.  The recorder
can then be converted into a `StorageProof`.  The `StorageProof` can be sent to a client who can use
it to reconstruct a database and re-execute transactions against the data.

## User Guide

### Database Persistance

This library is generic over the database backed and hasher. This is achieved by being leveraging
the following traits provided by the [`hash_db`](https://github.com/paritytech/trie/tree/master/hash-db) 
library:

- [`HashDB`](https://github.com/paritytech/trie/blob/master/hash-db/src/lib.rs#L127-L147) - A mutable key-value database trait
- [`HashDBRef`](https://github.com/paritytech/trie/blob/master/hash-db/src/lib.rs#L150-L157) - An immutable key-value database trait
- [`Hasher`](https://github.com/paritytech/trie/blob/master/hash-db/src/lib.rs#L53-L73) - A hasher trait

The user is free to implement these traits for any database backend and hasher of their choosing. The traits
are re-exported in this library.

For the purpose of this user guide we will use a simple in-memory database `MemoryDB` which implements both
`HashDB` and `HashDBRef`.

### Implementing a Hasher

Here we provide an example of implementing the `Hasher` trait for `Sha3`. We will use this hasher for the
rest of the guide.

```rust
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
```

### Keyed Merkle Tree

Here we provide an example of constructing a `MemoryDB` which implements `HashDB` and `HashDBRef` traits.
We then use this to construct a new `TreeDBMut` of depth 8. We perform a number of insertions and deletions.
Finally we commit the changes. We then use the `MemoryDB` and updated root to construct a `TreeDB` and read 
the data we have just committed.

The following example is provided in the examples folder `examples/keyed_tree.rs` and it can be run using:
```bash
cargo run --example keyed_tree --features executable
```

```rust
// create an empty in memory database
let mut memory_db = MemoryDB::<Sha3, NoopKey<_>, Vec<u8>>::default();

// specify the tree depth - the actual depth will be 8 * TREE_DEPTH
const TREE_DEPTH: usize = 1;

// create a new default root
let mut root = Default::default();

// create a new mutable keyed tree with the specified depth
let mut tree = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut memory_db, &mut root)
    .expect("failed to create tree")
    .build();

// define some dummy data
let data = vec![
    ([0], b"flip".to_vec()),
    ([2], b"flop".to_vec()),
    ([8], b"flap".to_vec()),
    ([9], b"flup".to_vec()),
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
tree.remove(&[0]).expect("failed to delete data");
tree.remove(&[9]).expect("failed to delete data");

// commit the changes to the database
tree.commit();

// print the root hash
println!("root hash: {:?}", tree.root());

// lets now create an immutable keyed tree using the same database and root
let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&memory_db, &root)
    .expect("failed to create tree")
    .build();

// lets now get the data we inserted
let data_at_key_0 = tree.value(&[0]).expect("failed to get data");
let data_at_key_2 = tree.value(&[2]).expect("failed to get data");
let data_at_key_8 = tree.value(&[8]).expect("failed to get data");
let data_at_key_9 = tree.value(&[9]).expect("failed to get data");

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
```

### Indexed Merkle Tree

Here we provide an example of constructing a `MemoryDB` which implements `HashDB` and `HashDBRef` traits.
We then use this to construct a new `IndexTreeDBMut` of depth 8. We perform a number of insertions and deletions.
Finally we commit the changes. We then use the `MemoryDB` and updated root to construct a `IndexTreeDB` and read 
the data we have just committed.

The following example is provided in the examples folder `examples/index_tree.rs` and it can be run using:
```bash
cargo run --example index_tree --features executable
```

```rust
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
```

### Recorder and Storage Proof

Below we see an example of constructing a mutable tree, inserting some data into it and then commit
it to the database. We then construct a recorder and pass it to the immutable tree we construct.  We
then perform some value lookups in the tree and the recorder records the nodes it passes as it the
tree lookups are performed. The recorder is then converted into a storage proof and subsequently the
storage proof is converted into a database. We then use the database to construct a new tree and 
confirm that we can access the nodes that were recorded by the tree.

The following example is provided in the examples folder `examples/recorder.rs` and it can be run using:
```bash
cargo run --example recorder --features executable
```

```rust
// create an empty in memory database
let mut memory_db = MemoryDB::<Sha3, NoopKey<_>, Vec<u8>>::default();

// specify the tree depth - the actual depth will be 8 * TREE_DEPTH
const TREE_DEPTH: usize = 1;

// create a new default root
let mut root = Default::default();

// create a new mutable keyed tree with the specified depth
let mut tree = TreeDBMutBuilder::<TREE_DEPTH, Sha3>::new(&mut memory_db, &mut root)
    .expect("failed to create tree")
    .build();

// define some dummy data
let data = vec![
    ([0], b"flip".to_vec()),
    ([2], b"flop".to_vec()),
    ([8], b"flap".to_vec()),
    ([9], b"flup".to_vec()),
];

// insert the data into the tree
for (key, value) in data {
    tree.insert(&key, value).expect("failed to insert data");
}

// commit the changes to the database
tree.commit();

// lets construct a recorder
let mut recorder = Recorder::<Sha3>::new();

// lets now create an immutable keyed tree using the same database and root
let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&memory_db, &root)
    .expect("failed to create tree")
    .with_recorder(&mut recorder)
    .build();

// lets now get the data we inserted
tree.value(&[0]).expect("failed to get data");
tree.value(&[2]).expect("failed to get data");
tree.value(&[8]).expect("failed to get data");
tree.value(&[9]).expect("failed to get data");

// now lets generate a storage proof which will have recorded the tree nodes associated with the value lookups
let storage_proof = recorder.drain_storage_proof();

// now lets convert this to an in memory DB
let memory_db = storage_proof.into_memory_db::<Sha3>();

// now lets create a tree from this memory DB
let tree = TreeDBBuilder::<TREE_DEPTH, Sha3>::new(&memory_db, &root)
    .expect("failed to create tree")
    .build();

// now lets get the data again
let data_at_0 = tree.value(&[0]).expect("failed to get data");
let data_at_2 = tree.value(&[2]).expect("failed to get data");
let data_at_8 = tree.value(&[8]).expect("failed to get data");
let data_at_9 = tree.value(&[9]).expect("failed to get data");

// define a utility function to print the data
fn print_data(data: Option<Vec<u8>>) {
    match data {
        Some(data) => println!("data: {:?}", std::str::from_utf8(&data).unwrap()),
        None => println!("data: None"),
    }
}

// print the data
print_data(data_at_0);
print_data(data_at_2);
print_data(data_at_8);
print_data(data_at_9);
```

## Testing
The library tests can be run using the command:
```bash
cargo test
```

Alternatively one can execute the tests using the Dockerfile found in the root of the repo via the command:
```bash
docker run --rm -it $(docker build -q .)
```

## License
This project is [MIT licensed](./LICENSE.md).