import type {
  GameplayRandomPlanConditionDto,
  GameplayRandomPlanEntryDto,
  RulesetPatchChangeDto,
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
  readonly fingerprint: string;
  readonly label: string;
  readonly contract: string;
  readonly owner: string;
  readonly source: string;
  readonly references: readonly string[];
}

export interface RulesetPatchChangeView {
  readonly plane: string;
  readonly path: string;
  readonly transition: string;
  readonly effective: boolean;
}

export interface RulesetDerivationView {
  readonly definitionId: string;
  readonly owner: string;
  readonly base: string;
  readonly baseFingerprint: string;
  readonly mixins: readonly {
    readonly identity: string;
    readonly fingerprint: string;
    readonly parameters: string;
    readonly order: number;
  }[];
  readonly localPatchFingerprint: string;
  readonly materializedFingerprint: string;
  readonly changes: readonly RulesetPatchChangeView[];
}

export interface RulesetOverlayView {
  readonly overlay: string;
  readonly target: string;
  readonly impact: string;
  readonly expectedFingerprint: string;
  readonly beforeFingerprint: string;
  readonly afterFingerprint: string;
  readonly patchFingerprint: string;
  readonly order: number;
  readonly changes: readonly RulesetPatchChangeView[];
}

export interface RulesetUpgradeImpactView {
  readonly transition: string;
  readonly sourceChanges: readonly string[];
  readonly definitions: readonly {
    readonly definitionId: string;
    readonly status: string;
    readonly causes: readonly string[];
    readonly fields: readonly {
      readonly plane: string;
      readonly path: string;
      readonly transition: string;
    }[];
  }[];
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
  readonly derivations: readonly RulesetDerivationView[];
  readonly overlays: readonly RulesetOverlayView[];
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
  readonly upgradeImpact: RulesetUpgradeImpactView | null;
  readonly diagnostics: readonly RulesetDiagnosticDto[];
}

export interface GameplayActionView {
  readonly id: string;
  readonly name: string;
  readonly source: string;
  readonly team: string;
  readonly maximumRange: number;
  readonly maximumTargets: number;
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
  readonly x: number;
  readonly y: number;
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
  readonly acceptedRandomValues: string;
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
    readonly randomConsumed: string;
    readonly randomRequest: string | null;
    readonly randomEvidence: readonly {
      readonly kind: string;
      readonly dice: string;
      readonly values: readonly number[];
      readonly path: string;
    }[];
    readonly events: readonly string[];
    readonly trace: readonly string[];
  } | null;
  readonly archive: {
    readonly checkpointSchema: string;
    readonly replaySchemaVersion: number;
    readonly eventSchemaVersion: number;
    readonly artifactId: string;
    readonly artifactSchema: string;
    readonly composition: string;
    readonly language: string;
    readonly operationSchemas: readonly string[];
    readonly capabilitySchemas: readonly string[];
    readonly sourcePackages: readonly string[];
    readonly dependencyLock: readonly string[];
    readonly fingerprints: readonly string[];
    readonly definitionFingerprints: readonly string[];
    readonly stateRevision: string;
    readonly acceptedRandomPosition: string;
    readonly phase: string;
    readonly stateHash: string;
    readonly checkpointBytes: number;
    readonly verificationStatus: string;
    readonly verificationMessage: string;
    readonly replayEntries: readonly {
      readonly sequence: number;
      readonly operation: string;
      readonly outcome: string;
      readonly transition: string;
      readonly randomEvidence: readonly string[];
      readonly events: readonly string[];
    }[];
  };
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
    upgradeImpact:
      response.upgradeImpact === null
        ? null
        : {
            transition: `${response.upgradeImpact.fromArtifactId} → ${response.upgradeImpact.toArtifactId}`,
            sourceChanges: response.upgradeImpact.sourceChanges,
            definitions: response.upgradeImpact.definitions.map(
              (definition) => ({
                definitionId: definition.definitionId,
                status: definition.descendant
                  ? `${definition.change} derived descendant`
                  : definition.change,
                causes: definition.causes,
                fields: definition.fields.map((field) => ({
                  plane: field.plane,
                  path: field.path,
                  transition: `${field.before} → ${field.after}`,
                })),
              }),
            ),
          },
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
      team: action.team,
      maximumRange: action.maximumRange,
      maximumTargets: action.maximumTargets,
      candidateIds: action.candidateIds,
      costs: action.costs.map((cost) => `${cost.amount} ${cost.resourceId}`),
      randomPlan: action.randomPlan.map(randomPlanLabel),
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
      x: entity.x,
      y: entity.y,
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
            randomEvidence: gameplay.lastResult.randomEvidence.map(
              (evidence) => ({
                kind: evidence.kind,
                dice: `${evidence.count}d${evidence.sides}`,
                values: evidence.values,
                path: evidence.path,
              }),
            ),
            events: gameplay.lastResult.events.map(
              (event) => `${event.kind}: ${event.summary}`,
            ),
            trace: gameplay.lastResult.trace.map(
              (trace) => `${trace.code} · ${trace.path} · ${trace.detail}`,
            ),
          },
    archive: {
      checkpointSchema: gameplay.archive.checkpointSchema,
      replaySchemaVersion: gameplay.archive.replaySchemaVersion,
      eventSchemaVersion: gameplay.archive.eventSchemaVersion,
      artifactId: gameplay.archive.artifactId,
      artifactSchema: gameplay.archive.artifactSchema,
      composition: gameplay.archive.composition,
      language: gameplay.archive.language,
      operationSchemas: gameplay.archive.operationSchemas,
      capabilitySchemas: gameplay.archive.capabilitySchemas,
      sourcePackages: gameplay.archive.sourcePackages,
      dependencyLock: gameplay.archive.dependencyLock,
      fingerprints: [
        `source ${gameplay.archive.fingerprints.source}`,
        `semantic ${gameplay.archive.fingerprints.semantic}`,
        `presentation ${gameplay.archive.fingerprints.presentation}`,
      ],
      definitionFingerprints: gameplay.archive.definitionFingerprints,
      stateRevision: gameplay.archive.stateRevision,
      acceptedRandomPosition: gameplay.archive.acceptedRandomPosition,
      phase: gameplay.archive.phase,
      stateHash: gameplay.archive.stateHash,
      checkpointBytes: gameplay.archive.checkpointBytes,
      verificationStatus: gameplay.archive.verificationStatus,
      verificationMessage: gameplay.archive.verificationMessage,
      replayEntries: gameplay.archive.replayEntries.map((entry) => ({
        sequence: entry.sequence,
        operation: entry.operation,
        outcome: entry.outcome,
        transition: `${entry.before.phase} r${entry.before.revision} ${entry.before.stateHash} → ${entry.after.phase} r${entry.after.revision} ${entry.after.stateHash}`,
        randomEvidence: entry.randomEvidence,
        events: entry.events,
      })),
    },
  };
}

function randomPlanLabel(entry: GameplayRandomPlanEntryDto): string {
  const request = `${entry.request.kind} ${entry.request.count}d${entry.request.sides}`;
  if (entry.conditions.length === 0) {
    return `always: ${request}`;
  }
  const conditions = entry.conditions.map(randomConditionLabel).join(' and ');
  return `if ${conditions}: ${request}`;
}

function randomConditionLabel(
  condition: GameplayRandomPlanConditionDto,
): string {
  switch (condition.kind) {
    case 'whenThen':
      return 'predicate true';
    case 'whenOtherwise':
      return 'predicate false';
    case 'checkHit':
      return 'check hit';
    case 'checkMiss':
      return 'check miss';
    case 'checkSaved':
      return 'check saved';
    case 'checkFailed':
      return 'check failed';
    case 'checkNoRoll':
      return 'no-roll branch';
    case 'allPreviousTrue':
      return 'all prior predicates true';
    case 'anyPreviousFalse':
      return 'all prior predicates false';
    default:
      return condition.kind;
  }
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
      fingerprint: definition.fingerprint,
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
    derivations: artifact.derivations.map((derivation) => ({
      definitionId: derivation.definitionId,
      owner: derivation.owner,
      base: derivation.base,
      baseFingerprint: derivation.baseFingerprint,
      mixins: derivation.mixins.map((mixin) => ({
        identity: mixin.identity,
        fingerprint: mixin.fingerprint,
        parameters:
          mixin.parameters.length === 0
            ? 'no parameters'
            : mixin.parameters.join(', '),
        order: mixin.order,
      })),
      localPatchFingerprint: derivation.localPatchFingerprint,
      materializedFingerprint: derivation.materializedFingerprint,
      changes: derivation.changes.map(patchChangeView),
    })),
    overlays: artifact.overlays.map((overlay) => ({
      overlay: overlay.overlay,
      target: overlay.target,
      impact: `${overlay.plane} · ${overlay.conflictPolicy}`,
      expectedFingerprint: overlay.expectedFingerprint,
      beforeFingerprint: overlay.beforeFingerprint,
      afterFingerprint: overlay.afterFingerprint,
      patchFingerprint: overlay.patchFingerprint,
      order: overlay.order,
      changes: overlay.changes.map(patchChangeView),
    })),
    reservedSlots: `${artifact.derivationSlots} derivation · ${artifact.overlaySlots} overlay`,
    fingerprints: [
      { plane: 'Source', value: artifact.fingerprints.source },
      { plane: 'Semantic', value: artifact.fingerprints.semantic },
      { plane: 'Presentation', value: artifact.fingerprints.presentation },
    ],
  };
}

function patchChangeView(
  change: RulesetPatchChangeDto,
): RulesetPatchChangeView {
  return {
    plane: change.plane,
    path: change.path,
    transition: `${change.before} → ${change.after}`,
    effective: change.effective,
  };
}
