import type {
  RulesetArtifactSummaryDto,
  RulesetDiagnosticDto,
  RulesetWorkspaceResponseDto,
} from '@asha-rulebench/protocol';

export interface RulesetSourceView {
  readonly identity: string;
  readonly fingerprint: string;
}

export interface RulesetLockView {
  readonly requester: string;
  readonly resolution: string;
  readonly importAs: string;
  readonly relationship: string;
  readonly fingerprint: string;
}

export interface RulesetDefinitionView {
  readonly id: string;
  readonly label: string;
  readonly contract: string;
  readonly owner: string;
  readonly source: string;
  readonly references: readonly string[];
}

export interface RulesetRelationshipView {
  readonly kind: string;
  readonly edge: string;
  readonly order: number;
}

export interface RulesetArtifactInspectionView {
  readonly artifactId: string;
  readonly schema: string;
  readonly composition: string;
  readonly language: string;
  readonly sources: readonly RulesetSourceView[];
  readonly lock: readonly RulesetLockView[];
  readonly operations: readonly string[];
  readonly capabilities: readonly string[];
  readonly exportedRoots: readonly string[];
  readonly definitions: readonly RulesetDefinitionView[];
  readonly policyBindingIds: readonly string[];
  readonly relationships: readonly RulesetRelationshipView[];
  readonly reservedSlots: string;
  readonly fingerprints: readonly {
    readonly plane: string;
    readonly value: string;
  }[];
}

export interface RulesetWorkspaceView {
  readonly phase: 'empty' | 'candidate' | 'active';
  readonly statusLabel: string;
  readonly headline: string;
  readonly summary: string;
  readonly activationRevision: number;
  readonly gameplayAvailable: boolean;
  readonly activeArtifactId: string | null;
  readonly artifact: RulesetArtifactInspectionView | null;
  readonly diagnostics: readonly RulesetDiagnosticDto[];
}

export function rulesetWorkspaceView(
  response: RulesetWorkspaceResponseDto,
): RulesetWorkspaceView {
  const inspectedArtifact =
    response.candidateArtifact ?? response.activeArtifact;
  const common = {
    activationRevision: response.activationRevision,
    gameplayAvailable: response.gameplayAvailable,
    activeArtifactId: response.activeArtifact?.artifactId ?? null,
    artifact:
      inspectedArtifact === null ? null : artifactInspection(inspectedArtifact),
    diagnostics: response.diagnostics,
  };

  if (response.status === 'compiledCandidate') {
    return {
      ...common,
      phase: 'candidate',
      statusLabel: 'Rust validation accepted',
      headline: 'Compiled candidate ready',
      summary:
        'The exact package lock and exported-root closure are compiled, but activation has not changed runtime truth.',
    };
  }
  if (response.status === 'active') {
    return {
      ...common,
      phase: 'active',
      statusLabel: `Activation revision ${response.activationRevision}`,
      headline: 'Compiled ruleset active',
      summary:
        'The complete accepted artifact replaced the active slot atomically. Gameplay remains outside this task.',
    };
  }
  return {
    ...common,
    phase: 'empty',
    statusLabel: 'Awaiting explicit compilation',
    headline: 'No compiled ruleset active',
    summary:
      'Files and imports do not activate content. Compile the explicit TypeScript composition, inspect the Rust artifact, then activate it.',
  };
}

function artifactInspection(
  artifact: RulesetArtifactSummaryDto,
): RulesetArtifactInspectionView {
  return {
    artifactId: artifact.artifactId,
    schema: `${artifact.schema.id}@${artifact.schema.version}`,
    composition: `${artifact.composition.id}@${artifact.composition.version}`,
    language: `${artifact.language.id}@${artifact.language.version}`,
    sources: artifact.sourcePackages.map((source) => ({
      identity: `${source.id}@${source.version}`,
      fingerprint: source.sourceFingerprint,
    })),
    lock: artifact.dependencyLock.map((entry) => ({
      requester: entry.requester,
      resolution: `${entry.packageId} ${entry.requestedVersion} → ${entry.resolvedVersion}`,
      importAs: entry.importAs,
      relationship: entry.relationship,
      fingerprint: entry.sourceFingerprint,
    })),
    operations: artifact.requiredOperations.map(
      (requirement) => `${requirement.id}@${requirement.version}`,
    ),
    capabilities: artifact.requiredCapabilities.map(
      (requirement) => `${requirement.id}@${requirement.version}`,
    ),
    exportedRoots: artifact.exportedRoots,
    definitions: artifact.definitions.map((definition) => ({
      id: definition.id,
      label: definition.label ?? definition.id,
      contract: `${definition.kind} · ${definition.visibility} · ${definition.extensionPolicy}`,
      owner: `${definition.packageId}@${definition.packageVersion}`,
      source: `${definition.sourceModule}#${definition.sourceDeclaration}`,
      references: definition.references,
    })),
    policyBindingIds: artifact.policyBindingIds,
    relationships: artifact.relationships.map((relationship) => ({
      kind: relationship.kind,
      edge: `${relationship.source} → ${relationship.target}`,
      order: relationship.order,
    })),
    reservedSlots: `${artifact.derivationSlots} derivation · ${artifact.overlaySlots} overlay`,
    fingerprints: [
      { plane: 'Source', value: artifact.fingerprints.source },
      { plane: 'Semantic', value: artifact.fingerprints.semantic },
      { plane: 'Presentation', value: artifact.fingerprints.presentation },
    ],
  };
}
