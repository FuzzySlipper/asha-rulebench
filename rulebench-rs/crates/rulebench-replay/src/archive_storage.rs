use std::collections::BTreeMap;

use crate::ReplayPackage;

pub const REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM: &str =
    "fnv1a64.rulebench-replay-archive.v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayArchiveMetadata {
    pub package_id: String,
    pub session_id: String,
    pub scenario_id: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayArchiveEntry {
    pub metadata: ReplayArchiveMetadata,
    pub package: ReplayPackage,
    pub payload_fingerprint_algorithm: String,
    pub payload_fingerprint: String,
}

impl ReplayArchiveEntry {
    pub fn new(package: ReplayPackage, completed_at: impl Into<String>) -> Self {
        let metadata = ReplayArchiveMetadata {
            package_id: package.id.clone(),
            session_id: package.initial_session.session.id.clone(),
            scenario_id: package.initial_session.scenario.metadata.id.clone(),
            ruleset_id: package.ruleset.ruleset_id.clone(),
            ruleset_version: package.ruleset.ruleset_version.clone(),
            completed_at: completed_at.into(),
        };
        let mut entry = Self {
            metadata,
            package,
            payload_fingerprint_algorithm: REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM.to_string(),
            payload_fingerprint: String::new(),
        };
        entry.payload_fingerprint = fingerprint_entry_payload(&entry);
        entry
    }

    pub fn is_self_consistent(&self) -> bool {
        self.payload_fingerprint_algorithm == REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM
            && self.payload_fingerprint == fingerprint_entry_payload(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayArchiveStorageError {
    WriteFailed { package_id: String },
    ReadFailed { package_id: String },
    ListFailed,
    ClearFailed,
}

pub trait ReplayArchiveStorage {
    fn write(&mut self, entry: ReplayArchiveEntry) -> Result<(), ReplayArchiveStorageError>;
    fn read(
        &self,
        package_id: &str,
    ) -> Result<Option<ReplayArchiveEntry>, ReplayArchiveStorageError>;
    fn list(&self) -> Result<Vec<ReplayArchiveMetadata>, ReplayArchiveStorageError>;
    fn clear(&mut self) -> Result<(), ReplayArchiveStorageError>;
}

impl<T> ReplayArchiveStorage for Box<T>
where
    T: ReplayArchiveStorage + ?Sized,
{
    fn write(&mut self, entry: ReplayArchiveEntry) -> Result<(), ReplayArchiveStorageError> {
        (**self).write(entry)
    }

    fn read(
        &self,
        package_id: &str,
    ) -> Result<Option<ReplayArchiveEntry>, ReplayArchiveStorageError> {
        (**self).read(package_id)
    }

    fn list(&self) -> Result<Vec<ReplayArchiveMetadata>, ReplayArchiveStorageError> {
        (**self).list()
    }

    fn clear(&mut self) -> Result<(), ReplayArchiveStorageError> {
        (**self).clear()
    }
}

#[derive(Debug, Default)]
pub struct InMemoryReplayArchiveStorage {
    entries: BTreeMap<String, ReplayArchiveEntry>,
    fail_next_write: bool,
}

impl InMemoryReplayArchiveStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_next_write(&mut self) {
        self.fail_next_write = true;
    }

    #[cfg(test)]
    pub(crate) fn replace_for_test(&mut self, entry: ReplayArchiveEntry) {
        self.entries
            .insert(entry.metadata.package_id.clone(), entry);
    }
}

impl ReplayArchiveStorage for InMemoryReplayArchiveStorage {
    fn write(&mut self, entry: ReplayArchiveEntry) -> Result<(), ReplayArchiveStorageError> {
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(ReplayArchiveStorageError::WriteFailed {
                package_id: entry.metadata.package_id,
            });
        }
        self.entries
            .insert(entry.metadata.package_id.clone(), entry);
        Ok(())
    }

    fn read(
        &self,
        package_id: &str,
    ) -> Result<Option<ReplayArchiveEntry>, ReplayArchiveStorageError> {
        Ok(self.entries.get(package_id).cloned())
    }

    fn list(&self) -> Result<Vec<ReplayArchiveMetadata>, ReplayArchiveStorageError> {
        Ok(self
            .entries
            .values()
            .map(|entry| entry.metadata.clone())
            .collect())
    }

    fn clear(&mut self) -> Result<(), ReplayArchiveStorageError> {
        self.entries.clear();
        Ok(())
    }
}

fn fingerprint_entry_payload(entry: &ReplayArchiveEntry) -> String {
    let payload = format!("{:?}|{:?}", entry.metadata, entry.package);
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in payload.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
