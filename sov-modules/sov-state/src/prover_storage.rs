use std::{fs, path::Path, sync::Arc};

use crate::{
    storage::{StorageKey, StorageValue},
    tree_db::TreeReadLogger,
    Storage, StorageSpec,
};
use first_read_last_write_cache::cache::CacheLog;
use jmt::{storage::TreeWriter, JellyfishMerkleTree, KeyHash, PhantomHasher, SimpleHasher};
use sovereign_db::state_db::StateDB;
use sovereign_sdk::core::traits::Witness;

pub struct ProverStorage<S: StorageSpec> {
    db: StateDB,
    _phantom_hasher: PhantomHasher<S::Hasher>,
}

impl<S: StorageSpec> Clone for ProverStorage<S> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            _phantom_hasher: Default::default(),
        }
    }
}

impl<S: StorageSpec> ProverStorage<S> {
    #[cfg(any(test, feature = "temp"))]
    pub fn temporary() -> Self {
        let db = StateDB::temporary();
        Self::with_db(db).unwrap()
    }

    pub fn with_path(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let db = StateDB::with_path(&path)?;
        Self::with_db(db)
    }

    fn with_db(db: StateDB) -> Result<Self, anyhow::Error> {
        Ok(Self {
            db,
            _phantom_hasher: Default::default(),
        })
    }

    fn read_value(&self, key: StorageKey) -> Option<StorageValue> {
        match self
            .db
            .get_value_option_by_key(self.db.get_next_version(), key.as_ref())
        {
            Ok(value) => value.map(StorageValue::new_from_bytes),
            // It is ok to panic here, we assume the db is available and consistent.
            Err(e) => panic!("Unable to read value from db: {e}"),
        }
    }
}

impl<S: StorageSpec> Storage for ProverStorage<S> {
    type Witness = S::Witness;
    fn get(&self, key: StorageKey, witness: &Self::Witness) -> Option<StorageValue> {
        let val = self.read_value(key);
        witness.add_hint(val.clone());
        val
    }

    fn validate_and_commit(
        &self,
        cache_log: CacheLog,
        witness: &Self::Witness,
    ) -> Result<[u8; 32], anyhow::Error> {
        let latest_version = self.db.get_next_version() - 1;
        witness.add_hint(latest_version);

        let (reads, writes) = cache_log.split();
        let read_logger = TreeReadLogger::with_db_and_witness(self.db.clone(), witness);
        let untracked_jmt = JellyfishMerkleTree::<_, S::Hasher>::new(&self.db);

        // For each value that's been read from the tree, read it from the logged JMT to populate hints
        for (key, read_value) in reads.into_iter() {
            let key_hash = KeyHash(S::Hasher::hash(key.key.as_ref()));
            // TODO: Switch to the batch read API once it becomes available
            let (result, proof) = untracked_jmt.get_with_proof(key_hash, latest_version)?;
            witness.add_hint(proof);
            anyhow::ensure!(
                result.as_ref() == read_value.as_ref().map(|f| f.value.as_ref()),
                "Bug! Incorrect value read from jmt"
            )
        }

        let tracked_jmt = JellyfishMerkleTree::<_, S::Hasher>::new(&read_logger);
        // Compute the jmt update from the write batch
        let batch = writes.into_iter().map(|(key, value)| {
            let key_hash = KeyHash(S::Hasher::hash(key.key.as_ref()));
            self.db
                .put_preimage(key_hash, key.key.as_ref())
                .expect("preimage must succeed");
            (
                key_hash,
                value.map(|v| Arc::try_unwrap(v.value).unwrap_or_else(|arc| (*arc).clone())),
            )
        });

        let next_version = self.db.get_next_version();

        let (new_root, tree_update) = tracked_jmt
            .put_value_set(batch, next_version)
            .expect("JMT update must succeed");

        self.db
            .write_node_batch(&tree_update.node_batch)
            .expect("db write must succeed");
        self.db.inc_next_version();
        Ok(new_root.0)
    }
}

pub fn delete_storage(path: impl AsRef<Path>) {
    fs::remove_dir_all(&path)
        .or_else(|_| fs::remove_file(&path))
        .unwrap();
}

#[cfg(test)]
mod test {
    use jmt::Version;

    use crate::{mocks::MockStorageSpec, WorkingSet};

    use super::*;

    #[derive(Clone)]
    struct TestCase {
        key: StorageKey,
        value: StorageValue,
        version: Version,
    }

    fn create_tests() -> Vec<TestCase> {
        vec![
            TestCase {
                key: StorageKey::from("key_0"),
                value: StorageValue::from("value_0"),
                version: 1,
            },
            TestCase {
                key: StorageKey::from("key_1"),
                value: StorageValue::from("value_1"),
                version: 2,
            },
            TestCase {
                key: StorageKey::from("key_2"),
                value: StorageValue::from("value_2"),
                version: 3,
            },
        ]
    }

    #[test]
    fn test_jmt_storage() {
        let path = schemadb::temppath::TempPath::new();
        let tests = create_tests();
        {
            for test in tests.clone() {
                let prover_storage = ProverStorage::<MockStorageSpec>::with_path(&path).unwrap();
                let mut storage = WorkingSet::new(prover_storage.clone());
                assert_eq!(prover_storage.db.get_next_version(), test.version);

                storage.set(test.key.clone(), test.value.clone());
                let (cache, witness) = storage.freeze();
                prover_storage
                    .validate_and_commit(cache, &witness)
                    .expect("storage is valid");

                assert_eq!(test.value, prover_storage.get(test.key, &witness).unwrap());
                assert_eq!(prover_storage.db.get_next_version(), test.version + 1)
            }
        }

        {
            let storage = ProverStorage::<MockStorageSpec>::with_path(&path).unwrap();
            assert_eq!(storage.db.get_next_version(), (tests.len() + 1) as u64);
            for test in tests {
                assert_eq!(
                    test.value,
                    storage.get(test.key, &Default::default()).unwrap()
                );
            }
        }
    }
}
