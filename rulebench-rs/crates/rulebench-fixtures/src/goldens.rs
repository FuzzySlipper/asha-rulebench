/// Package-owned regression manifest.
///
/// The expectation remains Rust data; generated TypeScript is a projection of
/// that evidence and is checked separately through the named command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureGoldenManifest {
    pub package_id: String,
    pub artifacts: Vec<FixtureGoldenArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureGoldenArtifact {
    pub id: String,
    pub kind: FixtureGoldenArtifactKind,
    pub check_command: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureGoldenArtifactKind {
    Receipt,
    ScenarioCatalog,
    SessionTranscript,
    ControlHistory,
    ScriptReadout,
    AutomaticRun,
    ReplayVerification,
}
