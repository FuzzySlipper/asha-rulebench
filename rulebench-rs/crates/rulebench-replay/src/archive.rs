use std::collections::BTreeMap;

use crate::{
    validate_replay_package, verify_replay_package, ReplayArchiveEntry, ReplayArchiveMetadata,
    ReplayArchiveStorage, ReplayArchiveStorageError, ReplayPackage, REPLAY_PACKAGE_VERSION,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayArchiveQuery {
    pub session_id: Option<String>,
    pub scenario_id: Option<String>,
    pub ruleset_id: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayArchiveError {
    InvalidPackage,
    CombatNotFinalized,
    Storage(ReplayArchiveStorageError),
    UnknownPackage { package_id: String },
    CorruptPackage { package_id: String },
    UnsupportedPackageVersion { version: String },
}

impl ReplayArchiveError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidPackage => "invalidReplayPackage",
            Self::CombatNotFinalized => "replayCombatNotFinalized",
            Self::Storage(_) => "replayArchiveStorageFailed",
            Self::UnknownPackage { .. } => "unknownReplayPackage",
            Self::CorruptPackage { .. } => "corruptReplayPackage",
            Self::UnsupportedPackageVersion { .. } => "unsupportedReplayPackageVersion",
        }
    }
}

#[derive(Debug)]
pub struct ReplayArchive<S: ReplayArchiveStorage> {
    storage: S,
    cache: BTreeMap<String, ReplayPackage>,
}

impl<S: ReplayArchiveStorage> ReplayArchive<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            cache: BTreeMap::new(),
        }
    }

    pub fn save(
        &mut self,
        package: ReplayPackage,
        completed_at: impl Into<String>,
    ) -> Result<ReplayArchiveMetadata, ReplayArchiveError> {
        let verification = verify_replay_package(&package);
        if !verification.accepted {
            return Err(ReplayArchiveError::InvalidPackage);
        }
        if !verification.finalized {
            return Err(ReplayArchiveError::CombatNotFinalized);
        }

        let entry = ReplayArchiveEntry::new(package.clone(), completed_at);
        let metadata = entry.metadata.clone();
        self.storage
            .write(entry)
            .map_err(ReplayArchiveError::Storage)?;
        self.cache.insert(package.id.clone(), package);
        Ok(metadata)
    }

    pub fn list(
        &self,
        query: &ReplayArchiveQuery,
    ) -> Result<Vec<ReplayArchiveMetadata>, ReplayArchiveError> {
        Ok(self
            .storage
            .list()
            .map_err(ReplayArchiveError::Storage)?
            .into_iter()
            .filter(|metadata| matches_query(metadata, query))
            .collect())
    }

    pub fn retrieve(&mut self, package_id: &str) -> Result<ReplayPackage, ReplayArchiveError> {
        if let Some(package) = self.cache.get(package_id) {
            return Ok(package.clone());
        }
        let entry = self
            .storage
            .read(package_id)
            .map_err(ReplayArchiveError::Storage)?
            .ok_or_else(|| ReplayArchiveError::UnknownPackage {
                package_id: package_id.to_string(),
            })?;
        if !entry.is_self_consistent() {
            return Err(ReplayArchiveError::CorruptPackage {
                package_id: package_id.to_string(),
            });
        }
        if entry.package.package_version != REPLAY_PACKAGE_VERSION {
            return Err(ReplayArchiveError::UnsupportedPackageVersion {
                version: entry.package.package_version,
            });
        }
        if !validate_replay_package(&entry.package).accepted {
            return Err(ReplayArchiveError::CorruptPackage {
                package_id: package_id.to_string(),
            });
        }
        self.cache
            .insert(package_id.to_string(), entry.package.clone());
        Ok(entry.package)
    }

    pub fn clear_runtime_cache(&mut self) {
        self.cache.clear();
    }

    pub fn clear_all(&mut self) -> Result<(), ReplayArchiveError> {
        self.cache.clear();
        self.storage.clear().map_err(ReplayArchiveError::Storage)
    }

    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

fn matches_query(metadata: &ReplayArchiveMetadata, query: &ReplayArchiveQuery) -> bool {
    query
        .session_id
        .as_ref()
        .is_none_or(|value| value == &metadata.session_id)
        && query
            .scenario_id
            .as_ref()
            .is_none_or(|value| value == &metadata.scenario_id)
        && query
            .ruleset_id
            .as_ref()
            .is_none_or(|value| value == &metadata.ruleset_id)
        && query
            .completed_at
            .as_ref()
            .is_none_or(|value| value == &metadata.completed_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::tests::recorded_control_package;
    use crate::InMemoryReplayArchiveStorage;

    #[test]
    fn save_clear_runtime_and_reload_preserves_package() {
        let package = recorded_control_package();
        let mut archive = ReplayArchive::new(InMemoryReplayArchiveStorage::new());

        archive
            .save(package.clone(), "test-completion-001")
            .expect("finalized replay saves");
        archive.clear_runtime_cache();

        assert_eq!(archive.retrieve(&package.id), Ok(package.clone()));
        assert_eq!(
            archive
                .list(&ReplayArchiveQuery {
                    scenario_id: Some(package.initial_session.scenario.metadata.id),
                    ..ReplayArchiveQuery::default()
                })
                .expect("storage lists")
                .len(),
            1
        );
    }

    #[test]
    fn failed_write_does_not_create_a_visible_package() {
        let package = recorded_control_package();
        let mut storage = InMemoryReplayArchiveStorage::new();
        storage.fail_next_write();
        let mut archive = ReplayArchive::new(storage);

        assert!(matches!(
            archive.save(package, "test-completion-002"),
            Err(ReplayArchiveError::Storage(_))
        ));
        assert!(archive
            .list(&ReplayArchiveQuery::default())
            .expect("storage lists")
            .is_empty());
    }

    #[test]
    fn reload_detects_corruption_and_version_mismatch() {
        let package = recorded_control_package();
        let mut archive = ReplayArchive::new(InMemoryReplayArchiveStorage::new());
        archive
            .save(package.clone(), "test-completion-003")
            .expect("package saves");
        archive.clear_runtime_cache();

        let mut corrupt = ReplayArchiveEntry::new(package.clone(), "test-completion-003");
        corrupt.package.id = "mutated".to_string();
        archive.storage_mut().replace_for_test(corrupt);
        assert!(matches!(
            archive.retrieve(&package.id),
            Err(ReplayArchiveError::CorruptPackage { .. })
        ));

        let mut newer = package.clone();
        newer.package_version = "2.0.0".to_string();
        let newer_entry = ReplayArchiveEntry::new(newer, "test-completion-003");
        archive.storage_mut().replace_for_test(newer_entry);
        assert!(matches!(
            archive.retrieve(&package.id),
            Err(ReplayArchiveError::UnsupportedPackageVersion { .. })
        ));
    }
}
