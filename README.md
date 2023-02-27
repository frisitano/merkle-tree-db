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