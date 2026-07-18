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
  readonly gameplay: GameplayWorkspaceView | null;
  readonly activeArtifactId: string | null;
  readonly artifact: RulesetArtifactInspectionView | null;
  readonly diagnostics: readonly RulesetDiagnosticDto[];
}

export interface GameplayActionView {
  readonly id: string;
  readonly name: string;
  readonly source: string;
  readonly candidateIds: readonly string[];
  readonly costs: readonly string[];
  readonly randomPlan: readonly string[];
  readonly preflight: readonly {
    readonly targetId: string;
    readonly available: boolean;
    readonly message: string;
  }[];
}

export interface GameplayEntityView {
  readonly id: string;
  readonly team: string;
  readonly position: string;
  readonly vitality: string;
  readonly stats: readonly string[];
  readonly defenses: readonly string[];
  readonly resources: readonly string[];
  readonly modifiers: readonly string[];
}

export interface GameplayWorkspaceView {
  readonly actorId: string;
  readonly stateRevision: number;
  readonly acceptedRandomValues: number;
  readonly actions: readonly GameplayActionView[];
  readonly entities: readonly GameplayEntityView[];
  readonly pendingReaction: {
    readonly reactionId: string;
    readonly actionId: string;
    readonly targetId: string;
    readonly options: readonly {
      readonly id: string;
      readonly label: string;
      readonly damageReduction: number;
    }[];
  } | null;
  readonly result: {
    readonly status: string;
    readonly code: string | null;
    readonly message: string;
    readonly randomConsumed: number;
    readonly randomRequest: string | null;
    readonly events: readonly string[];
    readonly trace: readonly string[];
  } | null;
}

export function rulesetWorkspaceView(
  response: RulesetWorkspaceResponseDto,
): RulesetWorkspaceView {
  const inspectedArtifact =
    response.candidateArtifact ?? response.activeArtifact;
  const common = {
    activationRevision: response.activationRevision,
    gameplayAvailable: response.gameplayAvailable,
    gameplay:
      response.gameplay === null ? null : gameplayView(response.gameplay),
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
        'The complete accepted artifact replaced the active slot atomically and opened one persistent Rust authority session.',
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

function gameplayView(
  gameplay: NonNullable<RulesetWorkspaceResponseDto['gameplay']>,
): GameplayWorkspaceView {
  return {
    actorId: gameplay.actorId,
    stateRevision: gameplay.stateRevision,
    acceptedRandomValues: gameplay.acceptedRandomValues,
    actions: gameplay.actions.map((action) => ({
      id: action.id,
      name: action.name,
      source: action.sourcePath,
      candidateIds: action.candidateIds,
      costs: action.costs.map((cost) => `${cost.amount} ${cost.resourceId}`),
      randomPlan: action.randomRequests.map(
        (request) => `${request.kind}: ${request.count}d${request.sides}`,
      ),
      preflight: gameplay.preflights
        .filter((preflight) => preflight.actionId === action.id)
        .map((preflight) => ({
          targetId: preflight.targetId,
          available: preflight.available,
          message: preflight.message,
        })),
    })),
    entities: gameplay.entities.map((entity) => ({
      id: entity.id,
      team: entity.team,
      position: `(${entity.x}, ${entity.y})`,
      vitality: `${entity.vitality.current}/${entity.vitality.maximum ?? 'unbounded'}`,
      stats: entity.stats.map((value) => `${value.id} ${value.current}`),
      defenses: entity.defenses.map((value) => `${value.id} ${value.current}`),
      resources: entity.resources.map(
        (value) =>
          `${value.id} ${value.current}/${value.maximum ?? 'unbounded'}`,
      ),
      modifiers: entity.modifiers.map(
        (modifier) =>
          `${modifier.id} ${modifier.value} (${modifier.remainingTurns} turns, ${modifier.stackingGroup})`,
      ),
    })),
    pendingReaction:
      gameplay.pendingReaction === null
        ? null
        : {
            reactionId: gameplay.pendingReaction.reactionId,
            actionId: gameplay.pendingReaction.actionId,
            targetId: gameplay.pendingReaction.targetId,
            options: gameplay.pendingReaction.options,
          },
    result:
      gameplay.lastResult === null
        ? null
        : {
            status: gameplay.lastResult.status,
            code: gameplay.lastResult.code,
            message: gameplay.lastResult.message,
            randomConsumed: gameplay.lastResult.randomConsumed,
            randomRequest:
              gameplay.lastResult.randomRequest === null
                ? null
                : `${gameplay.lastResult.randomRequest.count}d${gameplay.lastResult.randomRequest.sides} at ${gameplay.lastResult.randomRequest.path}`,
            events: gameplay.lastResult.events.map(
              (event) => `${event.kind}: ${event.summary}`,
            ),
            trace: gameplay.lastResult.trace.map(
              (trace) => `${trace.code} · ${trace.path} · ${trace.detail}`,
            ),
          },
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
