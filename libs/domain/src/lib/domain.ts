import type {
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
  readonly encounterSetupRequired: boolean;
  readonly hostRandomSource: {
    readonly policyId: string;
    readonly policyVersion: number;
    readonly sourceId: string;
    readonly sourceVersion: number;
  };
  readonly supportedRandomSources: readonly {
    readonly policyId: string;
    readonly policyVersion: number;
    readonly sourceId: string;
    readonly sourceVersion: number;
  }[];
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
  readonly available: boolean;
  readonly unavailable: string | null;
  readonly maximumTargets: number;
  readonly candidateIds: readonly string[];
  readonly cellIds: readonly string[];
  readonly areaIds: readonly string[];
}

export interface GameplayTurnControlView {
  readonly kind: string;
  readonly label: string;
  readonly available: boolean;
  readonly unavailable: string | null;
}

export interface GameplayEntityView {
  readonly id: string;
  readonly label: string;
  readonly teamId: string;
  readonly x: number;
  readonly y: number;
  readonly position: string;
  readonly vitality: string;
  readonly definitionIds: readonly string[];
  readonly stats: readonly string[];
  readonly defenses: readonly string[];
  readonly resources: readonly string[];
  readonly modifiers: readonly string[];
}

export interface GameplayWorkspaceView {
  readonly artifactId: string;
  readonly actorId: string;
  readonly stateRevision: number;
  readonly acceptedRandomValues: string;
  readonly board: {
    readonly width: number;
    readonly height: number;
    readonly cells: readonly {
      readonly id: string;
      readonly x: number;
      readonly y: number;
      readonly capabilities: readonly string[];
    }[];
  };
  readonly turn: {
    readonly initiativeOrder: readonly string[];
    readonly currentActorId: string;
    readonly round: number;
    readonly turn: number;
  };
  readonly actions: readonly GameplayActionView[];
  readonly controls: readonly GameplayTurnControlView[];
  readonly entities: readonly GameplayEntityView[];
  readonly log: readonly {
    readonly sequence: string;
    readonly stateRevision: string;
    readonly actorId: string;
    readonly actionId: string;
    readonly events: readonly string[];
  }[];
  readonly outcome: {
    readonly status: string;
    readonly winningTeamIds: readonly string[];
  };
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
    encounterSetupRequired: response.encounterSetupRequired,
    hostRandomSource: response.hostRandomSource,
    supportedRandomSources: response.supportedRandomSources,
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
      summary: response.encounterSetupRequired
        ? 'The complete accepted artifact replaced the active slot atomically. Create an explicit artifact-pinned encounter before play.'
        : 'The complete accepted artifact and explicit encounter setup are active in one persistent Rust authority session.',
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
    artifactId: gameplay.artifactId,
    actorId: gameplay.actorId,
    stateRevision: gameplay.stateRevision,
    acceptedRandomValues: gameplay.acceptedRandomValues,
    board: {
      width: gameplay.board.width,
      height: gameplay.board.height,
      cells: gameplay.board.cells.map((cell) => ({
        id: cell.id,
        x: cell.position.x,
        y: cell.position.y,
        capabilities: cell.capabilities.map(
          (capability) =>
            `${capability.id}@${capability.version} ${capability.value.kind}`,
        ),
      })),
    },
    turn: {
      initiativeOrder: gameplay.turn.initiativeOrder,
      currentActorId: gameplay.turn.currentActorId,
      round: gameplay.turn.round,
      turn: gameplay.turn.turn,
    },
    actions: gameplay.actions.map((action) => ({
      id: action.definitionId,
      name: action.label,
      available: action.available,
      unavailable:
        action.unavailable === null
          ? null
          : `${action.unavailable.code}: ${action.unavailable.message}`,
      maximumTargets: action.maximumTargets,
      candidateIds: action.options.participantIds,
      cellIds: action.options.cellIds,
      areaIds: action.options.areaIds,
    })),
    controls: gameplay.controls.map((control) => ({
      kind: control.kind,
      label: control.label,
      available: control.available,
      unavailable:
        control.unavailable === null
          ? null
          : `${control.unavailable.code}: ${control.unavailable.message}`,
    })),
    entities: gameplay.entities.map((entity) => ({
      id: entity.id,
      label: entity.label,
      teamId: entity.teamId,
      x: entity.x,
      y: entity.y,
      position: `(${entity.x}, ${entity.y})`,
      vitality: `${entity.vitality.current}/${entity.vitality.maximum ?? 'unbounded'}`,
      definitionIds: entity.definitionIds,
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
    log: gameplay.log.map((entry) => ({
      sequence: entry.sequence,
      stateRevision: entry.stateRevision,
      actorId: entry.actorId,
      actionId: entry.actionId,
      events: entry.events.map((event) => `${event.kind}: ${event.summary}`),
    })),
    outcome: gameplay.outcome,
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
