use hash_db::{HashDBRef, Hasher, EMPTY_PREFIX};

use super::{
    ChildSelector, DBValue, DataError, HashMap, Key, Node, NodeHash, NodeStorage, TreeDB,
    TreeDBMut, TreeError, TreeRecorder,
};

// Lookup
// ================================================================================================

/// Lookup to search for Node in the database.
pub struct Lookup<'db, const D: usize, H: Hasher> {
    storage: Option<&'db NodeStorage<H>>,
    db: &'db dyn HashDBRef<H, DBValue>,
    root: &'db NodeHash<H>,
    null_nodes: &'db HashMap<H::Out, Node<H>>,
    recorder: Option<core::cell::RefCell<&'db mut dyn TreeRecorder<H>>>,
}

impl<'db, const D: usize, H: Hasher> Lookup<'db, D, H> {
    fn lookup(&self, node_hash: &NodeHash<H>) -> Result<Node<H>, TreeError> {
        let node = match node_hash {
            NodeHash::InMemory(_) if self.storage.is_none() => {
                return Err(TreeError::DataError(DataError::InMemoryNotSupported))
            }
            NodeHash::InMemory(hash) => self
                .storage
                .expect("The storage field should be present for lookup")
                .get(hash)
                .cloned()
                .ok_or(TreeError::DataError(DataError::InMemoryDataNotFound(
                    hash.as_ref().to_vec(),
                ))),
            NodeHash::Database(hash) => {
                let data = self.db.get(hash, EMPTY_PREFIX).ok_or(TreeError::DataError(
                    DataError::DatabaseDataNotFound(hash.as_ref().to_vec()),
                ))?;
                let node: Node<H> = data.try_into().map_err(TreeError::NodeError)?;

                if let Some(recorder) = self.recorder.as_ref() {
                    recorder.borrow_mut().record(&node);
                }

                Ok(node)
            }
            NodeHash::Default(hash) => {
                self.null_nodes
                    .get(hash)
                    .cloned()
                    .ok_or(TreeError::DataError(DataError::NullNodeDataNotFound(
                        hash.as_ref().to_vec(),
                    )))
            }
        }?;

        Ok(node)
    }

    fn lookup_leaf_node(
        &self,
        key: &Key<D>,
        proof: &mut Option<Vec<DBValue>>,
    ) -> Result<Option<Node<H>>, TreeError> {
        let mut current_node = self.lookup(&self.root)?;

        for bit in key.iter() {
            let child_selector = ChildSelector::new(bit);
            let child_hash = current_node
                .child_hash(&child_selector)
                .map_err(TreeError::NodeError)?;
            if child_hash.is_default() && proof.is_none() {
                return Ok(None);
            }

            // store the sibling hash in the proof
            if let Some(proof) = proof.as_mut() {
                let sibling_hash: H::Out = **current_node
                    .child_hash(&child_selector.sibling())
                    .map_err(TreeError::NodeError)?;
                proof.push(sibling_hash.as_ref().to_vec());
            }

            current_node = self.lookup(child_hash)?;
        }

        Ok(Some(current_node))
    }
}

//TODO: implement From<&TreeDB<'db, D, H>> and From<&TreeDBMut<'db, D, H>> for Lookup.

// impl<'db, const D: usize, H: Hasher> From<&TreeDB<'db, D, H>> for Lookup<'db, D, H> {
//     fn from(tree_db: &TreeDB<'db, D, H>) -> Self {
//         let TreeDB {
//             db,
//             root,
//             null_nodes,
//             recorder,
//         } = tree_db;

//         Self {
//             storage: None,
//             db,
//             root,
//             null_nodes,
//             recorder,
//         }
//     }
// }

// impl<'db, const D: usize, H: Hasher> From<&TreeDBMut<'db, D, H>> for Lookup<'db, D, H> {
//     fn from(tree_db_mut: &TreeDBMut<'db, D, H>) -> Self {
//         let TreeDBMut::<'db, D, H> {
//             storage,
//             db,
//             root_handle,
//             null_nodes,
//             recorder,
//             ..
//         } = tree_db_mut;

//         Self {
//             storage: Some(storage),
//             db: db_ref_immut,
//             root: root_handle,
//             null_nodes,
//             recorder,
//         }
//     }
// }
