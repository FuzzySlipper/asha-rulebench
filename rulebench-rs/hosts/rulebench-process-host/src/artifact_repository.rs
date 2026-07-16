use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rulebench_bridge::content_storage::ImportedContentPack;
use rulebench_bridge::replay_storage::{
    bind_authored_action, record_replay_package, AuthoredActionAbilityGrantReceipt,
    AuthoredActionBindingReceipt, AuthoredActionBindingRequest,
    CombatAutomationNoCandidateBehavior, CombatAutomationPolicySpec, CombatControlCommandSpec,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateSelectionSpec, CombatSessionCreateRequest,
    CombatSessionIntentCommandSpec, CommandRollMode, ContentFingerprint, ContentPackReference,
    ContentPackSetReference, EquipmentCommandKind, EquipmentCommandSpec, GridPosition,
    ReactionCommandSpec, ReactionResponseKind, ReplayArchiveEntry, ReplayArchiveMetadata,
    ReplayArchiveStorage, ReplayArchiveStorageError, ReplayCommand, ReplayCommandRecordingSpec,
    ReplayNarration, SessionRecoveryFrame, SessionRecoveryPackage, SessionRecoveryStorage,
    SessionRecoveryStorageError, StateFingerprint, UseActionIntent,
    REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION, REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM,
};
use rulebench_bridge::{prepare_replay_scenario, BridgeScenario};
use serde::{Deserialize, Serialize};

const REPLAY_STORAGE_FORMAT_VERSION: u32 = 2;
const LEGACY_REPLAY_STORAGE_FORMAT_VERSION: u32 = 1;
const REPLAY_INDEX_FORMAT_VERSION: u32 = 2;
const STORAGE_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-artifact-store.v1";
const LEGACY_REPLAY_ARCHIVE_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-replay-archive.v0";
const LEGACY_REPLAY_ARCHIVE_DEBUG_FINGERPRINT_ALGORITHM: &str =
    "fnv1a64.rulebench-replay-archive.v1";
const RECOVERY_STORAGE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRepositoryIssue {
    pub artifact_kind: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ReplayStorageOpenReport {
    pub storage: FileReplayArchiveStorage,
    pub issues: Vec<ArtifactRepositoryIssue>,
}

#[derive(Debug)]
pub struct FileReplayArchiveStorage {
    root: PathBuf,
    scenarios: BTreeMap<String, BridgeScenario>,
    binding_sources: BTreeMap<ContentPackReference, ImportedContentPack>,
    entries: BTreeMap<String, ReplayArchiveEntry>,
}

#[derive(Debug)]
pub struct SessionRecoveryOpenReport {
    pub storage: FileSessionRecoveryStorage,
    pub issues: Vec<ArtifactRepositoryIssue>,
}

#[derive(Debug)]
pub struct FileSessionRecoveryStorage {
    root: PathBuf,
    scenarios: BTreeMap<String, BridgeScenario>,
    binding_sources: BTreeMap<ContentPackReference, ImportedContentPack>,
    packages: BTreeMap<String, SessionRecoveryPackage>,
}

impl FileSessionRecoveryStorage {
    pub fn open(
        root: impl Into<PathBuf>,
        scenarios: impl IntoIterator<Item = BridgeScenario>,
    ) -> Result<SessionRecoveryOpenReport, ArtifactRepositoryIssue> {
        Self::open_with_content_bindings(root, scenarios, BTreeMap::new())
    }

    pub fn open_with_content_bindings(
        root: impl Into<PathBuf>,
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        binding_sources: BTreeMap<ContentPackReference, ImportedContentPack>,
    ) -> Result<SessionRecoveryOpenReport, ArtifactRepositoryIssue> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| {
            issue(
                "sessionRecovery",
                "createRecoveryDirectoryFailed",
                &root,
                "The session recovery directory could not be created.",
            )
        })?;
        let scenarios = scenarios
            .into_iter()
            .map(|scenario| (scenario.scenario.metadata.id.clone(), scenario))
            .collect();
        let mut storage = Self {
            root,
            scenarios,
            binding_sources,
            packages: BTreeMap::new(),
        };
        let issues = storage.load_packages()?;
        Ok(SessionRecoveryOpenReport { storage, issues })
    }

    fn load_packages(&mut self) -> Result<Vec<ArtifactRepositoryIssue>, ArtifactRepositoryIssue> {
        let directory = fs::read_dir(&self.root).map_err(|_| {
            issue(
                "sessionRecovery",
                "readRecoveryDirectoryFailed",
                &self.root,
                "The session recovery directory could not be read.",
            )
        })?;
        let mut paths = directory
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        let mut issues = Vec::new();
        for path in paths {
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if file_name.ends_with(".tmp") {
                issues.push(issue(
                    "sessionRecovery",
                    "partialRecoveryCommitQuarantined",
                    &path,
                    "An uncommitted session recovery temporary file was ignored.",
                ));
                continue;
            }
            if !file_name.ends_with(".recovery.json") {
                continue;
            }
            match self.decode_package(&path) {
                Ok(package) => {
                    let session_id = package.session_id().to_string();
                    if self.packages.insert(session_id.clone(), package).is_some() {
                        issues.push(issue(
                            "sessionRecovery",
                            "duplicateRecoveryIdentityQuarantined",
                            &path,
                            format!("A duplicate recovery identity was ignored: {session_id}."),
                        ));
                    }
                }
                Err(problem) => issues.push(problem),
            }
        }
        Ok(issues)
    }

    fn decode_package(
        &self,
        path: &Path,
    ) -> Result<SessionRecoveryPackage, ArtifactRepositoryIssue> {
        let bytes = fs::read(path).map_err(|_| {
            issue(
                "sessionRecovery",
                "readRecoveryFailed",
                path,
                "The session recovery artifact could not be read.",
            )
        })?;
        let envelope: StoredRecoveryEnvelope = serde_json::from_slice(&bytes).map_err(|error| {
            issue(
                "sessionRecovery",
                "corruptRecoveryEnvelopeQuarantined",
                path,
                format!("The recovery envelope was invalid and was ignored: {error}."),
            )
        })?;
        if envelope.format_version != RECOVERY_STORAGE_FORMAT_VERSION {
            return Err(issue(
                "sessionRecovery",
                "unsupportedRecoveryStorageVersionQuarantined",
                path,
                format!(
                    "Recovery storage version {} is not supported; expected {}.",
                    envelope.format_version, RECOVERY_STORAGE_FORMAT_VERSION
                ),
            ));
        }
        if envelope.fingerprint_algorithm != STORAGE_FINGERPRINT_ALGORITHM
            || envelope.payload_fingerprint != fingerprint_serializable(&envelope.payload)
        {
            return Err(issue(
                "sessionRecovery",
                "recoveryIntegrityMismatchQuarantined",
                path,
                "The recovery storage payload fingerprint did not match.",
            ));
        }
        let frame = envelope.frame.to_authority();
        let package = envelope
            .payload
            .to_recovery(
                envelope.generation,
                frame,
                &self.scenarios,
                &self.binding_sources,
            )
            .map_err(|message| {
                issue(
                    "sessionRecovery",
                    "incompatibleRecoveryPayloadQuarantined",
                    path,
                    message,
                )
            })?;
        package.restore().map_err(|error| {
            issue(
                "sessionRecovery",
                error.code(),
                path,
                format!("The recovery checkpoint could not be verified: {error:?}."),
            )
        })?;
        Ok(package)
    }

    fn package_path(&self, session_id: &str) -> PathBuf {
        self.root
            .join(format!("{}.recovery.json", hex_name(session_id)))
    }
}

impl SessionRecoveryStorage for FileSessionRecoveryStorage {
    fn write(
        &mut self,
        package: SessionRecoveryPackage,
    ) -> Result<(), SessionRecoveryStorageError> {
        let session_id = package.session_id().to_string();
        package
            .restore()
            .map_err(|_| SessionRecoveryStorageError::WriteFailed {
                session_id: session_id.clone(),
            })?;
        let envelope = StoredRecoveryEnvelope::from_package(&package);
        atomic_write_json(&self.package_path(&session_id), &envelope).map_err(|_| {
            SessionRecoveryStorageError::WriteFailed {
                session_id: session_id.clone(),
            }
        })?;
        self.packages.insert(session_id, package);
        Ok(())
    }

    fn list(&self) -> Result<Vec<SessionRecoveryPackage>, SessionRecoveryStorageError> {
        Ok(self.packages.values().cloned().collect())
    }

    fn delete(&mut self, session_id: &str) -> Result<(), SessionRecoveryStorageError> {
        let path = self.package_path(session_id);
        if path.exists() {
            fs::remove_file(path).map_err(|_| SessionRecoveryStorageError::DeleteFailed {
                session_id: session_id.to_string(),
            })?;
        }
        self.packages.remove(session_id);
        Ok(())
    }
}

impl FileReplayArchiveStorage {
    pub fn open(
        root: impl Into<PathBuf>,
        scenarios: impl IntoIterator<Item = BridgeScenario>,
    ) -> Result<ReplayStorageOpenReport, ArtifactRepositoryIssue> {
        Self::open_with_content_bindings(root, scenarios, BTreeMap::new())
    }

    pub fn open_with_content_bindings(
        root: impl Into<PathBuf>,
        scenarios: impl IntoIterator<Item = BridgeScenario>,
        binding_sources: BTreeMap<ContentPackReference, ImportedContentPack>,
    ) -> Result<ReplayStorageOpenReport, ArtifactRepositoryIssue> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| {
            issue(
                "replay",
                "createRepositoryDirectoryFailed",
                &root,
                "The replay repository directory could not be created.",
            )
        })?;
        let scenarios = scenarios
            .into_iter()
            .map(|scenario| (scenario.scenario.metadata.id.clone(), scenario))
            .collect::<BTreeMap<_, _>>();
        let mut storage = Self {
            root,
            scenarios,
            binding_sources,
            entries: BTreeMap::new(),
        };
        let mut issues = storage.load_entries()?;
        if let Err(index_issue) = storage.validate_index() {
            issues.push(index_issue);
        }
        storage.write_index().map_err(|_| {
            issue(
                "replayIndex",
                "writeIndexFailed",
                storage.index_path(),
                "The deterministic replay index could not be committed.",
            )
        })?;
        Ok(ReplayStorageOpenReport { storage, issues })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn load_entries(&mut self) -> Result<Vec<ArtifactRepositoryIssue>, ArtifactRepositoryIssue> {
        let directory = fs::read_dir(&self.root).map_err(|_| {
            issue(
                "replay",
                "readRepositoryDirectoryFailed",
                &self.root,
                "The replay repository directory could not be read.",
            )
        })?;
        let mut paths = directory
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        let mut issues = Vec::new();
        for path in paths {
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if file_name.ends_with(".tmp") {
                issues.push(issue(
                    "replay",
                    "partialCommitQuarantined",
                    &path,
                    "An uncommitted replay temporary file was ignored.",
                ));
                continue;
            }
            if !file_name.ends_with(".replay.json") {
                continue;
            }
            match self.decode_entry(&path) {
                Ok(decoded) => {
                    let entry = decoded.entry;
                    if decoded.requires_migration
                        && atomic_write_json(&path, &StoredReplayEnvelope::from_entry(&entry))
                            .is_err()
                    {
                        issues.push(issue(
                            "replay",
                            "replayMigrationCommitFailedQuarantined",
                            &path,
                            "A legacy replay was authority-verified but its atomic canonical migration could not be committed.",
                        ));
                        continue;
                    }
                    let package_id = entry.metadata.package_id.clone();
                    if self.entries.contains_key(&package_id) {
                        issues.push(issue(
                            "replay",
                            "duplicateReplayIdentityQuarantined",
                            &path,
                            format!(
                                "A duplicate replay package identity was ignored: {package_id}."
                            ),
                        ));
                    } else {
                        self.entries.insert(package_id, entry);
                    }
                }
                Err(entry_issue) => issues.push(entry_issue),
            }
        }
        Ok(issues)
    }

    fn decode_entry(&self, path: &Path) -> Result<DecodedReplayEntry, ArtifactRepositoryIssue> {
        let bytes = fs::read(path).map_err(|_| {
            issue(
                "replay",
                "readReplayFailed",
                path,
                "The replay artifact could not be read.",
            )
        })?;
        let envelope = serde_json::from_slice::<StoredReplayEnvelope>(&bytes).map_err(|error| {
            issue(
                "replay",
                "corruptReplayEnvelopeQuarantined",
                path,
                format!("The replay envelope was invalid and was ignored: {error}."),
            )
        })?;
        if envelope.format_version != REPLAY_STORAGE_FORMAT_VERSION
            && envelope.format_version != LEGACY_REPLAY_STORAGE_FORMAT_VERSION
        {
            return Err(issue(
                "replay",
                "unsupportedReplayStorageVersionQuarantined",
                path,
                format!(
                    "Replay storage version {} is not supported; expected {} or migratable legacy {}.",
                    envelope.format_version,
                    REPLAY_STORAGE_FORMAT_VERSION,
                    LEGACY_REPLAY_STORAGE_FORMAT_VERSION
                ),
            ));
        }
        if envelope.fingerprint_algorithm != STORAGE_FINGERPRINT_ALGORITHM
            || envelope.payload_fingerprint != fingerprint_serializable(&envelope.payload)
        {
            return Err(issue(
                "replay",
                "replayIntegrityMismatchQuarantined",
                path,
                "The replay storage payload fingerprint did not match.",
            ));
        }
        let entry = envelope
            .payload
            .to_entry(&self.scenarios, &self.binding_sources)
            .map_err(|message| {
                issue(
                    "replay",
                    "incompatibleReplayPayloadQuarantined",
                    path,
                    message,
                )
            })?;
        let requires_migration = envelope.format_version == LEGACY_REPLAY_STORAGE_FORMAT_VERSION;
        if requires_migration {
            let recognized_legacy = matches!(
                envelope.archive_fingerprint_algorithm.as_str(),
                LEGACY_REPLAY_ARCHIVE_FINGERPRINT_ALGORITHM
                    | LEGACY_REPLAY_ARCHIVE_DEBUG_FINGERPRINT_ALGORITHM
            );
            if !recognized_legacy {
                return Err(issue(
                    "replay",
                    "unsupportedReplayArchiveIdentityVersionQuarantined",
                    path,
                    "The legacy replay archive identity version is not supported for migration.",
                ));
            }
        } else if envelope.archive_payload_encoding_version.as_deref()
            != Some(REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION)
            || envelope.archive_fingerprint_algorithm
                != REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM
            || entry.payload_encoding_version
                != envelope
                    .archive_payload_encoding_version
                    .as_deref()
                    .unwrap_or_default()
            || entry.payload_fingerprint_algorithm != envelope.archive_fingerprint_algorithm
            || entry.payload_fingerprint != envelope.archive_payload_fingerprint
        {
            return Err(issue(
                "replay",
                "replayCanonicalIdentityMismatchQuarantined",
                path,
                "The replay no longer reconstructs to its recorded canonical payload identity.",
            ));
        }
        Ok(DecodedReplayEntry {
            entry,
            requires_migration,
        })
    }

    fn validate_index(&self) -> Result<(), ArtifactRepositoryIssue> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(());
        }
        let bytes = fs::read(&path).map_err(|_| {
            issue(
                "replayIndex",
                "readIndexFailed",
                &path,
                "The replay index could not be read and will be rebuilt.",
            )
        })?;
        let index = serde_json::from_slice::<StoredReplayIndex>(&bytes).map_err(|error| {
            issue(
                "replayIndex",
                "corruptIndexRebuilt",
                &path,
                format!("The replay index was invalid and will be rebuilt: {error}."),
            )
        })?;
        let expected = StoredReplayIndex::from_entries(&self.entries);
        if index.format_version != REPLAY_INDEX_FORMAT_VERSION {
            return Err(issue(
                "replayIndex",
                "unsupportedIndexVersionRebuilt",
                &path,
                "The replay index format was unsupported and will be rebuilt.",
            ));
        }
        if index.fingerprint_algorithm != STORAGE_FINGERPRINT_ALGORITHM
            || index.entries_fingerprint != fingerprint_serializable(&index.entries)
            || index.entries != expected.entries
        {
            return Err(issue(
                "replayIndex",
                "indexIntegrityMismatchRebuilt",
                &path,
                "The replay index did not match committed artifacts and will be rebuilt.",
            ));
        }
        Ok(())
    }

    fn write_index(&self) -> Result<(), ()> {
        atomic_write_json(
            &self.index_path(),
            &StoredReplayIndex::from_entries(&self.entries),
        )
    }

    fn entry_path(&self, package_id: &str) -> PathBuf {
        self.root
            .join(format!("{}.replay.json", hex_name(package_id)))
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }
}

impl ReplayArchiveStorage for FileReplayArchiveStorage {
    fn write(&mut self, entry: ReplayArchiveEntry) -> Result<(), ReplayArchiveStorageError> {
        let package_id = entry.metadata.package_id.clone();
        let envelope = StoredReplayEnvelope::from_entry(&entry);
        let reconstructs_exactly = envelope.payload.authored_action_binding.is_some()
            || envelope
                .payload
                .to_entry(&self.scenarios, &self.binding_sources)
                .is_ok_and(|rebuilt| rebuilt == entry);
        if !reconstructs_exactly {
            return Err(ReplayArchiveStorageError::WriteFailed { package_id });
        }
        let bytes = serde_json::to_vec_pretty(&envelope).map_err(|_| {
            ReplayArchiveStorageError::WriteFailed {
                package_id: package_id.clone(),
            }
        })?;
        let path = self.entry_path(&package_id);
        let previous = self.entries.get(&package_id).cloned();
        if let Some(existing) = &previous {
            if existing == &entry {
                return Ok(());
            }
        }
        let backup_path = previous.as_ref().map(|_| replacement_backup_path(&path));
        if let Some(backup_path) = &backup_path {
            fs::copy(&path, backup_path).map_err(|_| ReplayArchiveStorageError::WriteFailed {
                package_id: package_id.clone(),
            })?;
        }
        if atomic_write(&path, &bytes).is_err() {
            if let Some(backup_path) = &backup_path {
                let _ = fs::remove_file(backup_path);
            }
            return Err(ReplayArchiveStorageError::WriteFailed { package_id });
        }
        self.entries.insert(package_id.clone(), entry);
        if self.write_index().is_err() {
            match (&previous, &backup_path) {
                (Some(previous), Some(backup_path)) => {
                    self.entries.insert(package_id.clone(), previous.clone());
                    let _ = fs::rename(backup_path, &path);
                }
                _ => {
                    self.entries.remove(&package_id);
                    let _ = fs::remove_file(&path);
                }
            }
            return Err(ReplayArchiveStorageError::WriteFailed { package_id });
        }
        if let Some(backup_path) = backup_path {
            let _ = fs::remove_file(backup_path);
        }
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
        let package_ids = self.entries.keys().cloned().collect::<Vec<_>>();
        for package_id in &package_ids {
            fs::remove_file(self.entry_path(package_id))
                .map_err(|_| ReplayArchiveStorageError::ClearFailed)?;
        }
        self.entries.clear();
        self.write_index()
            .map_err(|_| ReplayArchiveStorageError::ClearFailed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredReplayEnvelope {
    format_version: u32,
    fingerprint_algorithm: String,
    payload_fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_payload_encoding_version: Option<String>,
    archive_fingerprint_algorithm: String,
    archive_payload_fingerprint: String,
    payload: StoredReplayPayload,
}

impl StoredReplayEnvelope {
    fn from_entry(entry: &ReplayArchiveEntry) -> Self {
        let payload = StoredReplayPayload::from_entry(entry);
        Self {
            format_version: REPLAY_STORAGE_FORMAT_VERSION,
            fingerprint_algorithm: STORAGE_FINGERPRINT_ALGORITHM.to_string(),
            payload_fingerprint: fingerprint_serializable(&payload),
            archive_payload_encoding_version: Some(entry.payload_encoding_version.clone()),
            archive_fingerprint_algorithm: entry.payload_fingerprint_algorithm.clone(),
            archive_payload_fingerprint: entry.payload_fingerprint.clone(),
            payload,
        }
    }
}

struct DecodedReplayEntry {
    entry: ReplayArchiveEntry,
    requires_migration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredReplayPayload {
    package_id: String,
    package_version: String,
    session_id: String,
    scenario_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content_pack_set: Option<StoredContentPackSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authored_action_binding: Option<StoredAuthoredActionBindingReceipt>,
    participants: Vec<StoredParticipant>,
    ruleset: StoredRulesetProvenance,
    completed_at: String,
    commands: Vec<StoredCommandRecord>,
    narration: Option<StoredNarration>,
}

impl StoredReplayPayload {
    fn from_entry(entry: &ReplayArchiveEntry) -> Self {
        let package = &entry.package;
        Self {
            package_id: package.id.clone(),
            package_version: package.package_version.clone(),
            session_id: package.initial_session.session.id.clone(),
            scenario_id: package.initial_session.scenario.metadata.id.clone(),
            content_pack_set: package
                .initial_session
                .scenario
                .content_pack_set
                .as_ref()
                .map(StoredContentPackSet::from_authority),
            authored_action_binding: package
                .initial_session
                .scenario
                .authored_action_binding
                .as_ref()
                .map(StoredAuthoredActionBindingReceipt::from_authority),
            participants: package
                .initial_session
                .scenario
                .combatants
                .iter()
                .map(|combatant| StoredParticipant {
                    id: combatant.id.clone(),
                    initiative: combatant.initiative,
                })
                .collect(),
            ruleset: StoredRulesetProvenance {
                ruleset_id: package.ruleset.ruleset_id.clone(),
                ruleset_version: package.ruleset.ruleset_version.clone(),
                modules: package
                    .ruleset
                    .module_versions
                    .iter()
                    .map(|module| StoredModuleProvenance {
                        module: module.module.code().to_string(),
                        version: module.version.clone(),
                    })
                    .collect(),
                effect_operation_vocabulary_version: package
                    .ruleset
                    .effect_operation_vocabulary_version
                    .clone(),
            },
            completed_at: entry.metadata.completed_at.clone(),
            commands: package
                .commands
                .iter()
                .map(StoredCommandRecord::from_authority)
                .collect(),
            narration: package.narration.as_ref().map(StoredNarration::from),
        }
    }

    fn to_entry(
        &self,
        scenarios: &BTreeMap<String, BridgeScenario>,
        binding_sources: &BTreeMap<ContentPackReference, ImportedContentPack>,
    ) -> Result<ReplayArchiveEntry, String> {
        let bridge_scenario = scenarios.get(&self.scenario_id).ok_or_else(|| {
            format!(
                "The recorded scenario is not registered: {}.",
                self.scenario_id
            )
        })?;
        let mut scenario = bridge_scenario.scenario.clone();
        apply_participant_configuration(&mut scenario.combatants, &self.participants)?;
        let mut scenario = prepare_replay_scenario(scenario);
        if let Some(stored_binding) = &self.authored_action_binding {
            let expected_binding = stored_binding.to_authority()?;
            let imported = binding_sources
                .get(&expected_binding.content_pack_set.root)
                .ok_or_else(|| {
                    format!(
                        "Authored action content is unavailable for exact root {}@{}.",
                        expected_binding.content_pack_set.root.id,
                        expected_binding.content_pack_set.root.version
                    )
                })?;
            scenario = bind_authored_action(
                scenario,
                imported,
                &AuthoredActionBindingRequest {
                    content_pack: expected_binding.content_pack_set.root.clone(),
                    action_id: expected_binding.action_id.clone(),
                    actor_id: expected_binding.actor_id.clone(),
                },
            )
            .map_err(|error| {
                format!(
                    "Authored action rebinding failed with {}: {}",
                    error.code, error.message
                )
            })?;
            if scenario.authored_action_binding.as_ref() != Some(&expected_binding) {
                return Err(
                    "Authored action binding receipt no longer reconstructs exactly.".to_string(),
                );
            }
        } else if let Some(content_pack_set) = &self.content_pack_set {
            scenario.content_pack_set = Some(content_pack_set.to_authority()?);
        }
        let ruleset = scenario
            .selected_ruleset()
            .ok_or_else(|| "The recorded scenario no longer selects a ruleset.".to_string())?
            .artifact_provenance();
        self.ruleset.verify(&ruleset)?;
        let commands = self
            .commands
            .iter()
            .map(StoredCommandRecord::to_authority)
            .collect::<Result<Vec<_>, _>>()?;
        let mut package = record_replay_package(
            &self.package_id,
            CombatSessionCreateRequest::new(&self.session_id, scenario),
            ruleset,
            commands,
        );
        if package.package_version != self.package_version {
            return Err(format!(
                "Replay package version {} is not supported; current version is {}.",
                self.package_version, package.package_version
            ));
        }
        package.narration = self.narration.as_ref().map(StoredNarration::to_authority);
        Ok(ReplayArchiveEntry::new(package, &self.completed_at))
    }

    fn from_recovery(package: &SessionRecoveryPackage) -> Self {
        let commands = package
            .commands
            .iter()
            .map(|command| ReplayCommandRecordingSpec::new(&command.id, command.command.clone()))
            .collect();
        let replay = record_replay_package(
            format!("recovery:{}", package.session_id()),
            package.initial_session.clone(),
            package.ruleset.clone(),
            commands,
        );
        Self::from_entry(&ReplayArchiveEntry::new(replay, "active-session"))
    }

    fn to_recovery(
        &self,
        generation: u64,
        expected_frame: SessionRecoveryFrame,
        scenarios: &BTreeMap<String, BridgeScenario>,
        binding_sources: &BTreeMap<ContentPackReference, ImportedContentPack>,
    ) -> Result<SessionRecoveryPackage, String> {
        let entry = self.to_entry(scenarios, binding_sources)?;
        let commands = entry
            .package
            .commands
            .into_iter()
            .map(|command| ReplayCommandRecordingSpec::new(command.id, command.command))
            .collect();
        let package = SessionRecoveryPackage::record(
            generation,
            entry.package.initial_session,
            entry.package.ruleset,
            commands,
        )
        .map_err(|error| format!("Recovery package reconstruction failed: {error:?}."))?;
        if package.frame != expected_frame {
            return Err(
                "Recovery frame identity did not match reconstructed authority.".to_string(),
            );
        }
        Ok(package)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredRecoveryEnvelope {
    format_version: u32,
    fingerprint_algorithm: String,
    payload_fingerprint: String,
    generation: u64,
    frame: StoredRecoveryFrame,
    payload: StoredReplayPayload,
}

impl StoredRecoveryEnvelope {
    fn from_package(package: &SessionRecoveryPackage) -> Self {
        let payload = StoredReplayPayload::from_recovery(package);
        Self {
            format_version: RECOVERY_STORAGE_FORMAT_VERSION,
            fingerprint_algorithm: STORAGE_FINGERPRINT_ALGORITHM.to_string(),
            payload_fingerprint: fingerprint_serializable(&payload),
            generation: package.frame.generation,
            frame: StoredRecoveryFrame::from_authority(&package.frame),
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredRecoveryFrame {
    generation: u64,
    command_sequence: Option<u32>,
    command_id: Option<String>,
    state_fingerprint_algorithm: String,
    state_fingerprint_value: String,
    gameplay_module_state_hash: String,
    pending_reaction_window_id: Option<String>,
}

impl StoredRecoveryFrame {
    fn from_authority(value: &SessionRecoveryFrame) -> Self {
        Self {
            generation: value.generation,
            command_sequence: value.command_sequence,
            command_id: value.command_id.clone(),
            state_fingerprint_algorithm: value.state_fingerprint.algorithm.clone(),
            state_fingerprint_value: value.state_fingerprint.value.clone(),
            gameplay_module_state_hash: value.gameplay_module_state_hash.clone(),
            pending_reaction_window_id: value.pending_reaction_window_id.clone(),
        }
    }

    fn to_authority(&self) -> SessionRecoveryFrame {
        SessionRecoveryFrame {
            generation: self.generation,
            command_sequence: self.command_sequence,
            command_id: self.command_id.clone(),
            state_fingerprint: StateFingerprint {
                algorithm: self.state_fingerprint_algorithm.clone(),
                value: self.state_fingerprint_value.clone(),
            },
            gameplay_module_state_hash: self.gameplay_module_state_hash.clone(),
            pending_reaction_window_id: self.pending_reaction_window_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredContentPackSet {
    root: StoredContentPackReference,
    packs: Vec<StoredContentPackReference>,
    fingerprint: StoredContentFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredAuthoredActionBindingReceipt {
    binding_version: String,
    content_pack_set: StoredContentPackSet,
    action_id: String,
    action_definition_fingerprint: StoredContentFingerprint,
    ability_id: String,
    scenario_id: String,
    actor_id: String,
    grant_actor_id: String,
    grant_ability_id: String,
    targeting_operation_vocabulary_version: String,
    check_vocabulary_version: String,
    effect_operation_vocabulary_version: String,
}

impl StoredAuthoredActionBindingReceipt {
    fn from_authority(value: &AuthoredActionBindingReceipt) -> Self {
        Self {
            binding_version: value.binding_version.clone(),
            content_pack_set: StoredContentPackSet::from_authority(&value.content_pack_set),
            action_id: value.action_id.clone(),
            action_definition_fingerprint: StoredContentFingerprint::from_authority(
                &value.action_definition_fingerprint,
            ),
            ability_id: value.ability_id.clone(),
            scenario_id: value.scenario_id.clone(),
            actor_id: value.actor_id.clone(),
            grant_actor_id: value.grant.actor_id.clone(),
            grant_ability_id: value.grant.ability_id.clone(),
            targeting_operation_vocabulary_version: value
                .targeting_operation_vocabulary_version
                .clone(),
            check_vocabulary_version: value.check_vocabulary_version.clone(),
            effect_operation_vocabulary_version: value.effect_operation_vocabulary_version.clone(),
        }
    }

    fn to_authority(&self) -> Result<AuthoredActionBindingReceipt, String> {
        Ok(AuthoredActionBindingReceipt {
            binding_version: self.binding_version.clone(),
            content_pack_set: self.content_pack_set.to_authority()?,
            action_id: self.action_id.clone(),
            action_definition_fingerprint: self.action_definition_fingerprint.to_authority(),
            ability_id: self.ability_id.clone(),
            scenario_id: self.scenario_id.clone(),
            actor_id: self.actor_id.clone(),
            grant: AuthoredActionAbilityGrantReceipt {
                actor_id: self.grant_actor_id.clone(),
                ability_id: self.grant_ability_id.clone(),
            },
            targeting_operation_vocabulary_version: self
                .targeting_operation_vocabulary_version
                .clone(),
            check_vocabulary_version: self.check_vocabulary_version.clone(),
            effect_operation_vocabulary_version: self.effect_operation_vocabulary_version.clone(),
        })
    }
}

impl StoredContentPackSet {
    fn from_authority(value: &ContentPackSetReference) -> Self {
        Self {
            root: StoredContentPackReference::from_authority(&value.root),
            packs: value
                .packs
                .iter()
                .map(StoredContentPackReference::from_authority)
                .collect(),
            fingerprint: StoredContentFingerprint::from_authority(&value.fingerprint),
        }
    }

    fn to_authority(&self) -> Result<ContentPackSetReference, String> {
        let reference = ContentPackSetReference {
            root: self.root.to_authority(),
            packs: self
                .packs
                .iter()
                .map(StoredContentPackReference::to_authority)
                .collect(),
            fingerprint: self.fingerprint.to_authority(),
        };
        if !reference.is_self_consistent() {
            return Err("Stored replay content pack set is not self-consistent.".to_string());
        }
        Ok(reference)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredContentPackReference {
    id: String,
    version: String,
    fingerprint: StoredContentFingerprint,
}

impl StoredContentPackReference {
    fn from_authority(value: &ContentPackReference) -> Self {
        Self {
            id: value.id.clone(),
            version: value.version.clone(),
            fingerprint: StoredContentFingerprint::from_authority(&value.fingerprint),
        }
    }

    fn to_authority(&self) -> ContentPackReference {
        ContentPackReference {
            id: self.id.clone(),
            version: self.version.clone(),
            fingerprint: self.fingerprint.to_authority(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredContentFingerprint {
    algorithm: String,
    value: String,
}

impl StoredContentFingerprint {
    fn from_authority(value: &ContentFingerprint) -> Self {
        Self {
            algorithm: value.algorithm.clone(),
            value: value.value.clone(),
        }
    }

    fn to_authority(&self) -> ContentFingerprint {
        ContentFingerprint {
            algorithm: self.algorithm.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredParticipant {
    id: String,
    initiative: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredRulesetProvenance {
    ruleset_id: String,
    ruleset_version: String,
    modules: Vec<StoredModuleProvenance>,
    effect_operation_vocabulary_version: String,
}

impl StoredRulesetProvenance {
    fn verify(
        &self,
        actual: &rulebench_bridge::replay_storage::RulesetArtifactProvenance,
    ) -> Result<(), String> {
        let actual_modules = actual
            .module_versions
            .iter()
            .map(|module| StoredModuleProvenance {
                module: module.module.code().to_string(),
                version: module.version.clone(),
            })
            .collect::<Vec<_>>();
        if self.ruleset_id != actual.ruleset_id
            || self.ruleset_version != actual.ruleset_version
            || self.modules != actual_modules
            || self.effect_operation_vocabulary_version
                != actual.effect_operation_vocabulary_version
        {
            return Err(
                "The recorded ruleset provenance is not compatible with the registered scenario."
                    .to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredModuleProvenance {
    module: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredCommandRecord {
    id: String,
    command: StoredCommand,
}

impl StoredCommandRecord {
    fn from_authority(value: &rulebench_bridge::replay_storage::ReplayCommandRecord) -> Self {
        Self {
            id: value.id.clone(),
            command: StoredCommand::from_authority(&value.command),
        }
    }

    fn to_authority(&self) -> Result<ReplayCommandRecordingSpec, String> {
        Ok(ReplayCommandRecordingSpec::new(
            &self.id,
            self.command.to_authority()?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum StoredCommand {
    Intent {
        id: String,
        title: String,
        summary: String,
        intent: StoredIntent,
        roll_stream: Vec<i32>,
        roll_mode: StoredRollMode,
    },
    Control {
        command_kind: String,
    },
    SelectedCandidate {
        id: String,
        title: String,
        summary: String,
        action_id: String,
        target_id: String,
        roll_stream: Vec<i32>,
        roll_mode: StoredRollMode,
    },
    AutomaticStep {
        id: String,
        title: String,
        summary: String,
        roll_stream: Vec<i32>,
        roll_mode: StoredRollMode,
        policy: StoredPolicy,
    },
    AutomaticRun {
        id: String,
        title: String,
        summary: String,
        max_steps: u32,
        roll_stream: Vec<i32>,
        roll_mode: StoredRollMode,
        policy: StoredPolicy,
    },
    Equipment {
        command_kind: String,
        combatant_id: String,
        item_id: String,
    },
    Reaction {
        window_id: String,
        reactor_id: String,
        response_kind: String,
        option_id: Option<String>,
    },
}

impl StoredCommand {
    fn from_authority(value: &ReplayCommand) -> Self {
        match value {
            ReplayCommand::Intent(spec) => Self::Intent {
                id: spec.id.clone(),
                title: spec.title.clone(),
                summary: spec.summary.clone(),
                intent: StoredIntent::from(&spec.intent),
                roll_stream: spec.roll_stream.clone(),
                roll_mode: StoredRollMode::from(spec.roll_mode),
            },
            ReplayCommand::Control(spec) => Self::Control {
                command_kind: spec.kind.code().to_string(),
            },
            ReplayCommand::SelectedCandidate(spec) => Self::SelectedCandidate {
                id: spec.id.clone(),
                title: spec.title.clone(),
                summary: spec.summary.clone(),
                action_id: spec.action_id.clone(),
                target_id: spec.target_id.clone(),
                roll_stream: spec.roll_stream.clone(),
                roll_mode: StoredRollMode::from(spec.roll_mode),
            },
            ReplayCommand::AutomaticStep(spec) => Self::AutomaticStep {
                id: spec.id.clone(),
                title: spec.title.clone(),
                summary: spec.summary.clone(),
                roll_stream: spec.roll_stream.clone(),
                roll_mode: StoredRollMode::from(spec.roll_mode),
                policy: StoredPolicy::from(&spec.policy),
            },
            ReplayCommand::AutomaticRun(spec) => Self::AutomaticRun {
                id: spec.id.clone(),
                title: spec.title.clone(),
                summary: spec.summary.clone(),
                max_steps: spec.max_steps,
                roll_stream: spec.roll_stream.clone(),
                roll_mode: StoredRollMode::from(spec.roll_mode),
                policy: StoredPolicy::from(&spec.policy),
            },
            ReplayCommand::Equipment(spec) => Self::Equipment {
                command_kind: spec.kind.code().to_string(),
                combatant_id: spec.combatant_id.clone(),
                item_id: spec.item_id.clone(),
            },
            ReplayCommand::Reaction(spec) => Self::Reaction {
                window_id: spec.window_id.clone(),
                reactor_id: spec.reactor_id.clone(),
                response_kind: spec.response_kind.code().to_string(),
                option_id: spec.option_id.clone(),
            },
        }
    }

    fn to_authority(&self) -> Result<ReplayCommand, String> {
        match self {
            Self::Intent {
                id,
                title,
                summary,
                intent,
                roll_stream,
                roll_mode,
            } => Ok(ReplayCommand::Intent(CombatSessionIntentCommandSpec {
                id: id.clone(),
                title: title.clone(),
                summary: summary.clone(),
                intent: intent.to_authority(),
                roll_stream: roll_stream.clone(),
                roll_mode: roll_mode.to_authority(),
            })),
            Self::Control { command_kind } => {
                Ok(ReplayCommand::Control(match command_kind.as_str() {
                    "explicitStart" => CombatControlCommandSpec::explicit_start(),
                    "explicitEnd" => CombatControlCommandSpec::explicit_end(),
                    "advanceTurn" => CombatControlCommandSpec::advance_turn(),
                    "endIfConditionMet" => CombatControlCommandSpec::end_if_condition_met(),
                    _ => return Err(format!("Unknown stored control command: {command_kind}.")),
                }))
            }
            Self::SelectedCandidate {
                id,
                title,
                summary,
                action_id,
                target_id,
                roll_stream,
                roll_mode,
            } => Ok(ReplayCommand::SelectedCandidate(
                CombatSessionCandidateSelectionSpec {
                    id: id.clone(),
                    title: title.clone(),
                    summary: summary.clone(),
                    action_id: action_id.clone(),
                    target_id: target_id.clone(),
                    roll_stream: roll_stream.clone(),
                    roll_mode: roll_mode.to_authority(),
                },
            )),
            Self::AutomaticStep {
                id,
                title,
                summary,
                roll_stream,
                roll_mode,
                policy,
            } => Ok(ReplayCommand::AutomaticStep(
                CombatSessionAutomaticStepSpec {
                    id: id.clone(),
                    title: title.clone(),
                    summary: summary.clone(),
                    roll_stream: roll_stream.clone(),
                    policy: policy.to_authority()?,
                    roll_mode: roll_mode.to_authority(),
                },
            )),
            Self::AutomaticRun {
                id,
                title,
                summary,
                max_steps,
                roll_stream,
                roll_mode,
                policy,
            } => Ok(ReplayCommand::AutomaticRun(CombatSessionAutomaticRunSpec {
                id: id.clone(),
                title: title.clone(),
                summary: summary.clone(),
                max_steps: *max_steps,
                roll_stream: roll_stream.clone(),
                policy: policy.to_authority()?,
                roll_mode: roll_mode.to_authority(),
            })),
            Self::Equipment {
                command_kind,
                combatant_id,
                item_id,
            } => {
                let kind = match command_kind.as_str() {
                    "equip" => EquipmentCommandKind::Equip,
                    "unequip" => EquipmentCommandKind::Unequip,
                    _ => return Err(format!("Unknown stored equipment command: {command_kind}.")),
                };
                Ok(ReplayCommand::Equipment(EquipmentCommandSpec {
                    kind,
                    combatant_id: combatant_id.clone(),
                    item_id: item_id.clone(),
                }))
            }
            Self::Reaction {
                window_id,
                reactor_id,
                response_kind,
                option_id,
            } => {
                let response_kind = match response_kind.as_str() {
                    "pass" => ReactionResponseKind::Pass,
                    "accept" => ReactionResponseKind::Accept,
                    _ => {
                        return Err(format!(
                            "Unknown stored reaction response: {response_kind}."
                        ))
                    }
                };
                Ok(ReplayCommand::Reaction(ReactionCommandSpec {
                    window_id: window_id.clone(),
                    reactor_id: reactor_id.clone(),
                    response_kind,
                    option_id: option_id.clone(),
                }))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum StoredRollMode {
    Supplied,
    AuthorityGenerated { seed: u64 },
    RecordedGenerated { seed: u64 },
}

impl StoredRollMode {
    fn to_authority(self) -> CommandRollMode {
        match self {
            Self::Supplied => CommandRollMode::Supplied,
            Self::AuthorityGenerated { seed } => CommandRollMode::AuthorityGenerated { seed },
            Self::RecordedGenerated { seed } => CommandRollMode::RecordedGenerated { seed },
        }
    }
}

impl From<CommandRollMode> for StoredRollMode {
    fn from(value: CommandRollMode) -> Self {
        match value {
            CommandRollMode::Supplied => Self::Supplied,
            CommandRollMode::AuthorityGenerated { seed } => Self::AuthorityGenerated { seed },
            CommandRollMode::RecordedGenerated { seed } => Self::RecordedGenerated { seed },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredPolicy {
    id: String,
    version: u32,
    no_candidate_behavior: String,
}

impl StoredPolicy {
    fn to_authority(&self) -> Result<CombatAutomationPolicySpec, String> {
        let no_candidate_behavior = match self.no_candidate_behavior.as_str() {
            "advanceTurn" => CombatAutomationNoCandidateBehavior::AdvanceTurn,
            "stopRun" => CombatAutomationNoCandidateBehavior::StopRun,
            unknown => return Err(format!("Unknown stored no-candidate behavior: {unknown}.")),
        };
        Ok(CombatAutomationPolicySpec {
            id: self.id.clone(),
            version: self.version,
            no_candidate_behavior,
        })
    }
}

impl From<&CombatAutomationPolicySpec> for StoredPolicy {
    fn from(value: &CombatAutomationPolicySpec) -> Self {
        Self {
            id: value.id.clone(),
            version: value.version,
            no_candidate_behavior: value.no_candidate_behavior.code().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredIntent {
    actor_id: String,
    action_id: String,
    target_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    target_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_cell: Option<StoredGridPosition>,
    destination_cell: Option<StoredGridPosition>,
    observed_origin: Option<StoredGridPosition>,
}

impl StoredIntent {
    fn to_authority(&self) -> UseActionIntent {
        UseActionIntent {
            actor_id: self.actor_id.clone(),
            action_id: self.action_id.clone(),
            target_id: self.target_id.clone(),
            target_ids: self.target_ids.clone(),
            target_cell: self
                .target_cell
                .as_ref()
                .map(StoredGridPosition::to_authority),
            destination_cell: self
                .destination_cell
                .as_ref()
                .map(StoredGridPosition::to_authority),
            observed_origin: self
                .observed_origin
                .as_ref()
                .map(StoredGridPosition::to_authority),
        }
    }
}

impl From<&UseActionIntent> for StoredIntent {
    fn from(value: &UseActionIntent) -> Self {
        Self {
            actor_id: value.actor_id.clone(),
            action_id: value.action_id.clone(),
            target_id: value.target_id.clone(),
            target_ids: value.target_ids.clone(),
            target_cell: value.target_cell.map(StoredGridPosition::from),
            destination_cell: value.destination_cell.map(StoredGridPosition::from),
            observed_origin: value.observed_origin.map(StoredGridPosition::from),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredGridPosition {
    x: u32,
    y: u32,
}

impl StoredGridPosition {
    fn to_authority(&self) -> GridPosition {
        GridPosition {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<GridPosition> for StoredGridPosition {
    fn from(value: GridPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredNarration {
    title: String,
    summary: String,
    command_summaries: Vec<String>,
}

impl StoredNarration {
    fn to_authority(&self) -> ReplayNarration {
        ReplayNarration {
            title: self.title.clone(),
            summary: self.summary.clone(),
            command_summaries: self.command_summaries.clone(),
        }
    }
}

impl From<&ReplayNarration> for StoredNarration {
    fn from(value: &ReplayNarration) -> Self {
        Self {
            title: value.title.clone(),
            summary: value.summary.clone(),
            command_summaries: value.command_summaries.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredReplayIndex {
    format_version: u32,
    fingerprint_algorithm: String,
    entries_fingerprint: String,
    entries: Vec<StoredReplayIndexEntry>,
}

impl StoredReplayIndex {
    fn from_entries(entries: &BTreeMap<String, ReplayArchiveEntry>) -> Self {
        let entries = entries
            .values()
            .map(|entry| StoredReplayIndexEntry {
                package_id: entry.metadata.package_id.clone(),
                session_id: entry.metadata.session_id.clone(),
                scenario_id: entry.metadata.scenario_id.clone(),
                ruleset_id: entry.metadata.ruleset_id.clone(),
                ruleset_version: entry.metadata.ruleset_version.clone(),
                completed_at: entry.metadata.completed_at.clone(),
                archive_payload_fingerprint: entry.payload_fingerprint.clone(),
                archive_payload_encoding_version: entry.payload_encoding_version.clone(),
            })
            .collect::<Vec<_>>();
        Self {
            format_version: REPLAY_INDEX_FORMAT_VERSION,
            fingerprint_algorithm: STORAGE_FINGERPRINT_ALGORITHM.to_string(),
            entries_fingerprint: fingerprint_serializable(&entries),
            entries,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredReplayIndexEntry {
    package_id: String,
    session_id: String,
    scenario_id: String,
    ruleset_id: String,
    ruleset_version: String,
    completed_at: String,
    archive_payload_fingerprint: String,
    archive_payload_encoding_version: String,
}

fn apply_participant_configuration<T>(
    combatants: &mut [T],
    participants: &[StoredParticipant],
) -> Result<(), String>
where
    T: ParticipantConfiguration,
{
    let mut configured = BTreeMap::new();
    for participant in participants {
        if configured
            .insert(participant.id.clone(), participant.initiative)
            .is_some()
        {
            return Err(format!("Duplicate stored participant: {}.", participant.id));
        }
    }
    if combatants.len() != configured.len() {
        return Err(
            "The stored participant set does not match the registered scenario.".to_string(),
        );
    }
    for combatant in combatants {
        let initiative = configured
            .remove(combatant.participant_id())
            .ok_or_else(|| {
                format!(
                    "The stored participant is not registered: {}.",
                    combatant.participant_id()
                )
            })?;
        combatant.set_initiative(initiative);
    }
    Ok(())
}

trait ParticipantConfiguration {
    fn participant_id(&self) -> &str;
    fn set_initiative(&mut self, initiative: i32);
}

impl ParticipantConfiguration for rulebench_bridge::replay_storage::Combatant {
    fn participant_id(&self) -> &str {
        &self.id
    }

    fn set_initiative(&mut self, initiative: i32) {
        self.initiative = initiative;
    }
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| ())?;
    atomic_write(path, &bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ()> {
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact")
    ));
    fs::write(&temporary, bytes).map_err(|_| ())?;
    if fs::rename(&temporary, path).is_err() {
        let _ = fs::remove_file(&temporary);
        return Err(());
    }
    Ok(())
}

fn fingerprint_serializable<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("repository values have deterministic JSON");
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn hex_name(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn replacement_backup_path(path: &Path) -> PathBuf {
    path.with_extension(format!(
        "{}.replace-backup.tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact")
    ))
}

fn issue(
    artifact_kind: impl Into<String>,
    code: impl Into<String>,
    path: impl AsRef<Path>,
    message: impl Into<String>,
) -> ArtifactRepositoryIssue {
    ArtifactRepositoryIssue {
        artifact_kind: artifact_kind.into(),
        code: code.into(),
        path: path.as_ref().display().to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicU64, Ordering};

    use rulebench_fixtures::{aggregated_scenario_catalog_cases, replay_review_packages};

    use super::*;

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    const REPLAY_STORAGE_V1_FIXTURE: &str = include_str!("fixtures/replay-storage-v1.json");

    #[test]
    fn committed_v1_replay_storage_fixture_remains_readable() {
        let directory = test_directory("v1-compatibility");
        fs::create_dir_all(&directory).expect("fixture repository creates");
        let path = directory.join(format!("{}.replay.json", hex_name("hexing-bolt-replay")));
        fs::write(&path, REPLAY_STORAGE_V1_FIXTURE).expect("v1 fixture copies");

        let report =
            FileReplayArchiveStorage::open(&directory, scenarios()).expect("v1 repository opens");
        assert!(report.issues.is_empty(), "v1 issues: {:?}", report.issues);
        let entry = report
            .storage
            .read("hexing-bolt-replay")
            .expect("v1 repository reads")
            .expect("v1 fixture is visible");
        assert_eq!(entry.package.package_version, "1.0.0");
        assert_eq!(entry.metadata.completed_at, "v1-compatibility-fixture");
        assert_eq!(
            entry.payload_fingerprint_algorithm,
            REPLAY_ARCHIVE_PAYLOAD_FINGERPRINT_ALGORITHM
        );
        assert_eq!(
            entry.payload_encoding_version,
            REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION
        );
        assert!(entry.is_self_consistent());
        let migrated = serde_json::from_slice::<serde_json::Value>(
            &fs::read(&path).expect("migrated fixture reads"),
        )
        .expect("migrated fixture remains JSON");
        assert_eq!(migrated["formatVersion"], REPLAY_STORAGE_FORMAT_VERSION);
        assert_eq!(
            migrated["archivePayloadEncodingVersion"],
            REPLAY_ARCHIVE_PAYLOAD_ENCODING_VERSION
        );
        drop(report.storage);

        let reopened =
            FileReplayArchiveStorage::open(&directory, scenarios()).expect("v2 repository reopens");
        assert!(
            reopened.issues.is_empty(),
            "v2 issues: {:?}",
            reopened.issues
        );
        assert!(reopened
            .storage
            .read("hexing-bolt-replay")
            .expect("migrated repository reads")
            .expect("migrated fixture remains visible")
            .is_self_consistent());
        cleanup(&directory);
    }

    #[test]
    fn legacy_migration_is_atomic_and_quarantines_an_uncommitted_rewrite() {
        let directory = test_directory("migration-commit-failure");
        fs::create_dir_all(&directory).expect("fixture repository creates");
        let path = directory.join(format!("{}.replay.json", hex_name("hexing-bolt-replay")));
        fs::write(&path, REPLAY_STORAGE_V1_FIXTURE).expect("v1 fixture copies");
        let temporary = path.with_extension("json.tmp");
        fs::create_dir(&temporary).expect("migration fault fixture creates");

        let report = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens with migration issue");
        assert_eq!(
            report
                .issues
                .iter()
                .map(|issue| issue.code.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "partialCommitQuarantined",
                "replayMigrationCommitFailedQuarantined",
            ])
        );
        assert!(report.storage.list().expect("repository lists").is_empty());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &fs::read(&path).expect("legacy file remains readable")
            )
            .expect("legacy file remains JSON")["formatVersion"],
            LEGACY_REPLAY_STORAGE_FORMAT_VERSION
        );
        cleanup(&directory);
    }

    #[test]
    fn replay_repository_round_trips_across_restart_with_deterministic_index() {
        let directory = test_directory("restart");
        let package = replay_review_packages().remove(0);
        let expected = ReplayArchiveEntry::new(package, "restart-proof");
        let scenario_map = scenarios()
            .into_iter()
            .map(|scenario| (scenario.scenario.metadata.id.clone(), scenario))
            .collect::<BTreeMap<_, _>>();
        let rebuilt = StoredReplayPayload::from_entry(&expected)
            .to_entry(&scenario_map, &BTreeMap::new())
            .expect("fixture reconstructs");
        assert_eq!(
            rebuilt.package.initial_session,
            expected.package.initial_session
        );
        assert_eq!(rebuilt.package.ruleset, expected.package.ruleset);
        assert_eq!(rebuilt.package.commands, expected.package.commands);
        assert_eq!(rebuilt.package.evidence, expected.package.evidence);
        assert_eq!(
            rebuilt.package.final_state_fingerprint,
            expected.package.final_state_fingerprint
        );
        assert_eq!(rebuilt.package.narration, expected.package.narration);
        {
            let report =
                FileReplayArchiveStorage::open(&directory, scenarios()).expect("repository opens");
            assert!(report.issues.is_empty());
            let mut storage = report.storage;
            storage.write(expected.clone()).expect("replay commits");
            assert!(storage.index_path().exists());
        }

        let reopened =
            FileReplayArchiveStorage::open(&directory, scenarios()).expect("repository reopens");
        assert!(reopened.issues.is_empty());
        assert_eq!(
            reopened.storage.read(&expected.metadata.package_id),
            Ok(Some(expected))
        );
        cleanup(&directory);
    }

    #[test]
    fn replay_repository_replaces_explicit_identity_and_preserves_it_across_restart() {
        let directory = test_directory("replacement");
        let package = replay_review_packages().remove(0);
        let expected = ReplayArchiveEntry::new(package.clone(), "replacement-proof");
        let mut storage = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens")
            .storage;
        storage.write(expected.clone()).expect("original commits");
        let mut changed = package;
        changed.narration = Some(ReplayNarration {
            title: "Changed".to_string(),
            summary: "A conflicting replacement".to_string(),
            command_summaries: Vec::new(),
        });
        let replacement = ReplayArchiveEntry::new(changed, "replacement-proof");
        storage
            .write(replacement.clone())
            .expect("replacement commits");
        drop(storage);

        let reopened =
            FileReplayArchiveStorage::open(&directory, scenarios()).expect("repository reopens");
        assert_eq!(
            reopened.storage.read(&expected.metadata.package_id),
            Ok(Some(replacement))
        );
        cleanup(&directory);
    }

    #[test]
    fn failed_replay_replacement_restores_the_last_good_entry() {
        let directory = test_directory("replacement-rollback");
        let package = replay_review_packages().remove(0);
        let expected = ReplayArchiveEntry::new(package.clone(), "replacement-rollback-proof");
        let mut storage = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens")
            .storage;
        storage.write(expected.clone()).expect("original commits");
        fs::remove_file(storage.index_path()).expect("index removes for fault fixture");
        fs::create_dir(storage.index_path()).expect("index fault fixture creates");
        let mut changed = package;
        changed.narration = Some(ReplayNarration {
            title: "Changed".to_string(),
            summary: "A replacement that must roll back".to_string(),
            command_summaries: Vec::new(),
        });

        assert!(matches!(
            storage.write(ReplayArchiveEntry::new(
                changed,
                "replacement-rollback-proof"
            )),
            Err(ReplayArchiveStorageError::WriteFailed { .. })
        ));
        assert_eq!(
            storage.read(&expected.metadata.package_id),
            Ok(Some(expected.clone()))
        );
        fs::remove_dir(storage.index_path()).expect("index fault fixture removes");
        drop(storage);

        let reopened = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository reopens after rollback");
        assert_eq!(
            reopened.storage.read(&expected.metadata.package_id),
            Ok(Some(expected))
        );
        cleanup(&directory);
    }

    #[test]
    fn replay_repository_classifies_corruption_partial_commits_and_unknown_versions() {
        let directory = test_directory("classification");
        let package = replay_review_packages().remove(0);
        let package_id = package.id.clone();
        let mut storage = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens")
            .storage;
        storage
            .write(ReplayArchiveEntry::new(package, "classification-proof"))
            .expect("replay commits");
        let path = storage.entry_path(&package_id);
        let mut envelope =
            serde_json::from_slice::<serde_json::Value>(&fs::read(&path).expect("artifact reads"))
                .expect("artifact is JSON");
        envelope["formatVersion"] = serde_json::Value::from(999_u32);
        fs::write(
            &path,
            serde_json::to_vec_pretty(&envelope).expect("modified artifact serializes"),
        )
        .expect("version fixture writes");
        fs::write(directory.join("interrupted.replay.json.tmp"), b"partial")
            .expect("partial fixture writes");
        fs::write(storage.index_path(), b"not-json").expect("index corruption writes");
        drop(storage);

        let reopened = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens with classified issues");
        let codes = reopened
            .issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("unsupportedReplayStorageVersionQuarantined"));
        assert!(codes.contains("partialCommitQuarantined"));
        assert!(codes.contains("corruptIndexRebuilt"));
        assert!(reopened
            .storage
            .list()
            .expect("repository lists")
            .is_empty());
        cleanup(&directory);
    }

    #[test]
    fn replay_repository_classifies_unknown_identity_partial_migration_and_payload_mismatch() {
        let directory = test_directory("canonical-classification");
        let package = replay_review_packages().remove(0);

        let unknown_path = directory.join("unknown.replay.json");
        let mut legacy = serde_json::from_str::<serde_json::Value>(REPLAY_STORAGE_V1_FIXTURE)
            .expect("legacy fixture parses");
        legacy["archiveFingerprintAlgorithm"] = serde_json::Value::from("unknown.v9");
        fs::create_dir_all(&directory).expect("repository directory creates");
        fs::write(
            &unknown_path,
            serde_json::to_vec_pretty(&legacy).expect("legacy fixture serializes"),
        )
        .expect("unknown identity fixture writes");

        let mut storage = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens with unknown identity")
            .storage;
        storage
            .write(ReplayArchiveEntry::new(
                package.clone(),
                "partial-migration",
            ))
            .expect("partial migration source writes");
        let partial_path = storage.entry_path(&package.id);
        let mut partial = serde_json::from_slice::<serde_json::Value>(
            &fs::read(&partial_path).expect("partial source reads"),
        )
        .expect("partial source parses");
        partial
            .as_object_mut()
            .expect("envelope is object")
            .remove("archivePayloadEncodingVersion");
        fs::write(
            &partial_path,
            serde_json::to_vec_pretty(&partial).expect("partial fixture serializes"),
        )
        .expect("partial fixture writes");

        let mismatch = ReplayArchiveEntry::new(package, "payload-mismatch");
        let mismatch_path = directory.join("mismatch.replay.json");
        let mut mismatch_envelope = StoredReplayEnvelope::from_entry(&mismatch);
        mismatch_envelope.archive_payload_fingerprint = "changed".to_string();
        fs::write(
            &mismatch_path,
            serde_json::to_vec_pretty(&mismatch_envelope).expect("mismatch fixture serializes"),
        )
        .expect("mismatch fixture writes");
        drop(storage);

        let reopened = FileReplayArchiveStorage::open(&directory, scenarios())
            .expect("repository opens with canonical issues");
        let codes = reopened
            .issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("unsupportedReplayArchiveIdentityVersionQuarantined"));
        assert!(codes.contains("replayCanonicalIdentityMismatchQuarantined"));
        assert!(reopened
            .storage
            .list()
            .expect("repository lists")
            .is_empty());
        cleanup(&directory);
    }

    #[test]
    fn session_recovery_round_trips_and_quarantines_corrupt_or_stale_frames() {
        let directory = test_directory("session-recovery");
        let replay = replay_review_packages()
            .into_iter()
            .next()
            .expect("replay fixture exists");
        let command = replay.commands[0].clone();
        let package = SessionRecoveryPackage::record(
            1,
            replay.initial_session,
            replay.ruleset,
            vec![ReplayCommandRecordingSpec::new(command.id, command.command)],
        )
        .expect("recovery package records");
        let session_id = package.session_id().to_string();
        let mut report =
            FileSessionRecoveryStorage::open(&directory, scenarios()).expect("storage opens");
        report
            .storage
            .write(package.clone())
            .expect("checkpoint commits");
        drop(report.storage);

        let reopened =
            FileSessionRecoveryStorage::open(&directory, scenarios()).expect("storage reopens");
        assert!(reopened.issues.is_empty(), "issues={:?}", reopened.issues);
        assert_eq!(
            reopened.storage.list().expect("storage lists"),
            vec![package]
        );
        drop(reopened.storage);

        let path = directory.join(format!("{}.recovery.json", hex_name(&session_id)));
        let mut stale: StoredRecoveryEnvelope =
            serde_json::from_slice(&fs::read(&path).expect("checkpoint reads"))
                .expect("checkpoint decodes");
        stale.frame.state_fingerprint_value.push_str("-stale");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&stale).expect("stale fixture serializes"),
        )
        .expect("stale fixture writes");
        fs::write(directory.join("partial.recovery.json.tmp"), b"partial")
            .expect("partial fixture writes");

        let quarantined =
            FileSessionRecoveryStorage::open(&directory, scenarios()).expect("storage quarantines");
        let codes = quarantined
            .issues
            .iter()
            .map(|problem| problem.code.as_str())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("incompatibleRecoveryPayloadQuarantined"));
        assert!(codes.contains("partialRecoveryCommitQuarantined"));
        assert!(quarantined
            .storage
            .list()
            .expect("quarantined storage lists")
            .is_empty());
        cleanup(&directory);
    }

    fn scenarios() -> Vec<BridgeScenario> {
        let mut scenarios = aggregated_scenario_catalog_cases()
            .into_iter()
            .map(|case| {
                BridgeScenario::new(
                    case.summary.id,
                    case.summary.title,
                    case.summary.summary,
                    case.scenario,
                )
            })
            .collect::<Vec<_>>();
        for (index, package) in replay_review_packages().into_iter().enumerate() {
            let scenario = package.initial_session.scenario;
            if !scenarios
                .iter()
                .any(|registered| registered.scenario.metadata.id == scenario.metadata.id)
            {
                let title = scenario.metadata.title.clone();
                let summary = scenario.metadata.summary.clone();
                scenarios.push(BridgeScenario::new(
                    format!("replay-storage-test-{index:04}"),
                    title,
                    summary,
                    scenario,
                ));
            }
        }
        scenarios
    }

    fn test_directory(label: &str) -> PathBuf {
        let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "asha-rulebench-replay-storage-{}-{label}-{sequence}",
            std::process::id()
        ))
    }

    fn cleanup(directory: &Path) {
        fs::remove_dir_all(directory).expect("test repository should clean up");
    }
}
