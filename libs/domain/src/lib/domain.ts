import type {
  ContentPatchChangeDto,
  GameplayItemBindingDto,
  GameplayItemInstanceDto,
  ItemDefinitionDto,
  PlayBundleArtifactSummaryDto,
  PlayDiagnosticDto,
  PlayWorkspaceResponseDto,
  ScenarioEquipmentSlotDto,
  ScenarioInitialCapabilityDto,
  ScenarioItemInstanceDto,
} from '@asha-rulebench/protocol';

export interface ContentPackSourceView {
  readonly identity: string;
  readonly fingerprint: string;
}

export interface ContentPackLockView {
  readonly requester: string;
  readonly resolution: string;
  readonly importAs: string;
  readonly relationship: string;
  readonly fingerprint: string;
}

export interface ContentDefinitionView {
  readonly id: string;
  readonly fingerprint: string;
  readonly label: string;
  readonly description: string | null;
  readonly tags: readonly string[];
  readonly catalog: string | null;
  readonly catalogId: string | null;
  readonly contract: string;
  readonly owner: string;
  readonly source: string;
  readonly references: readonly string[];
}

export interface RulesetValueView {
  readonly kind: string;
  readonly id: string;
  readonly label: string;
  readonly numericDomainId: string;
}

export interface ParticipantProfileView {
  readonly definitionId: string;
  readonly profileId: string;
  readonly label: string;
  readonly description: string | null;
  readonly role: string;
  readonly definitionIds: readonly string[];
  readonly classDefinitionId: string | null;
  readonly featureDefinitionIds: readonly string[];
  readonly items: readonly ScenarioItemInstanceDto[];
  readonly equipment: readonly ScenarioEquipmentSlotDto[];
  readonly capabilities: readonly ScenarioInitialCapabilityDto[];
}

export interface ContentPatchChangeView {
  readonly plane: string;
  readonly path: string;
  readonly transition: string;
  readonly effective: boolean;
}

export interface ContentDerivationView {
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
  readonly changes: readonly ContentPatchChangeView[];
}

export interface ContentOverlayView {
  readonly overlay: string;
  readonly target: string;
  readonly impact: string;
  readonly expectedFingerprint: string;
  readonly beforeFingerprint: string;
  readonly afterFingerprint: string;
  readonly patchFingerprint: string;
  readonly order: number;
  readonly changes: readonly ContentPatchChangeView[];
}

export interface PlayBundleUpgradeImpactView {
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

export interface ContentRelationshipView {
  readonly kind: string;
  readonly edge: string;
  readonly order: number;
}

export interface PlayBundleArtifactInspectionView {
  readonly artifactId: string;
  readonly schema: string;
  readonly playBundle: string;
  readonly ruleset: string;
  readonly language: string;
  readonly contentPacks: readonly ContentPackSourceView[];
  readonly lock: readonly ContentPackLockView[];
  readonly operations: readonly string[];
  readonly capabilities: readonly string[];
  readonly values: readonly string[];
  readonly numericDomains: readonly string[];
  readonly rulesetValues: readonly RulesetValueView[];
  readonly participantProfiles: readonly ParticipantProfileView[];
  readonly itemDefinitions: readonly ItemDefinitionDto[];
  readonly exportedRoots: readonly string[];
  readonly definitions: readonly ContentDefinitionView[];
  readonly policyBindingIds: readonly string[];
  readonly relationships: readonly ContentRelationshipView[];
  readonly derivations: readonly ContentDerivationView[];
  readonly overlays: readonly ContentOverlayView[];
  readonly reservedSlots: string;
  readonly fingerprints: readonly {
    readonly plane: string;
    readonly value: string;
  }[];
}

export interface PlayWorkspaceView {
  readonly phase: 'empty' | 'candidate' | 'active';
  readonly statusLabel: string;
  readonly headline: string;
  readonly summary: string;
  readonly activationRevision: number;
  readonly scenarioSetupRequired: boolean;
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
  readonly artifact: PlayBundleArtifactInspectionView | null;
  readonly upgradeImpact: PlayBundleUpgradeImpactView | null;
  readonly diagnostics: readonly PlayDiagnosticDto[];
}

export interface GameplayActionView {
  readonly identity: string;
  readonly id: string;
  readonly name: string;
  readonly description: string | null;
  readonly tags: readonly string[];
  readonly itemBinding: GameplayItemBindingDto | null;
  readonly available: boolean;
  readonly unavailable: string | null;
  readonly maximumTargets: number;
  readonly candidateIds: readonly string[];
  readonly cellPaths: readonly GameplayCellPathView[];
  readonly areaIds: readonly string[];
}

export interface GameplayCellPathView {
  readonly destinationCellId: string;
  readonly cellIds: readonly string[];
  readonly movementCost: number;
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
  readonly classDefinitionId: string | null;
  readonly featureDefinitionIds: readonly string[];
  readonly items: readonly GameplayItemInstanceDto[];
  readonly equipment: readonly ScenarioEquipmentSlotDto[];
  readonly stats: readonly string[];
  readonly defenses: readonly string[];
  readonly resources: readonly string[];
  readonly modifiers: readonly string[];
}

export interface GameplayRollContributionView {
  readonly sourceDefinitionId: string;
  readonly sourceLabel: string;
  readonly amount: number;
  readonly reasonKind: string;
  readonly contributionId: string | null;
  readonly selector: string | null;
  readonly condition: string | null;
}

export interface GameplayRollResolutionView {
  readonly kind: string;
  readonly dieResult: number;
  readonly total: number;
  readonly thresholdLabel: string;
  readonly threshold: number;
  readonly outcome: string;
  readonly contributions: readonly GameplayRollContributionView[];
}

export interface GameplayEventView {
  readonly kind: string;
  readonly summary: string;
  readonly roll: GameplayRollResolutionView | null;
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
    readonly itemBinding: GameplayItemBindingDto | null;
    readonly events: readonly GameplayEventView[];
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
    readonly playBundle: string;
    readonly ruleset: string;
    readonly operationSchemas: readonly string[];
    readonly capabilitySchemas: readonly string[];
    readonly contentPacks: readonly string[];
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

export function gameplayActionIdentity(
  definitionId: string,
  itemBinding: GameplayItemBindingDto | null,
): string {
  if (itemBinding === null) return definitionId;
  return [
    definitionId,
    itemBinding.bindingId,
    itemBinding.itemInstanceId,
    itemBinding.itemDefinitionId,
    itemBinding.slotId,
  ].join('\u0000');
}

export function playWorkspaceView(
  response: PlayWorkspaceResponseDto,
): PlayWorkspaceView {
  const inspectedArtifact =
    response.candidateArtifact ?? response.activeArtifact;
  const common = {
    activationRevision: response.activationRevision,
    scenarioSetupRequired: response.scenarioSetupRequired,
    hostRandomSource: response.hostRandomSource,
    supportedRandomSources: response.supportedRandomSources,
    gameplayAvailable: response.gameplayAvailable,
    gameplay:
      response.gameplay === null
        ? null
        : gameplayView(
            response.gameplay,
            response.activeArtifact?.rulesetValues ?? [],
            response.activeArtifact?.definitions ?? [],
          ),
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
      headline: 'PlayBundle active',
      summary: response.scenarioSetupRequired
        ? 'The accepted PlayBundle replaced the active slot atomically. Create or load an explicit PlayBundle-pinned Scenario before play.'
        : 'The accepted PlayBundle and explicit Scenario are active in one persistent Rust authority session.',
    };
  }
  return {
    ...common,
    phase: 'empty',
    statusLabel: 'Awaiting explicit compilation',
    headline: 'No PlayBundle active',
    summary:
      'Select a Ruleset and compatible Content Packs, compile the declared PlayBundle, then activate it.',
  };
}

function gameplayView(
  gameplay: NonNullable<PlayWorkspaceResponseDto['gameplay']>,
  rulesetValues: PlayBundleArtifactSummaryDto['rulesetValues'],
  definitions: PlayBundleArtifactSummaryDto['definitions'],
): GameplayWorkspaceView {
  const rulesetValueLabels = new Map(
    rulesetValues.map((value) => [`${value.kind}:${value.id}`, value.label]),
  );
  const catalogLabels = new Map(
    definitions.flatMap((definition) =>
      definition.catalog === null || definition.catalogId === null
        ? []
        : [
            [
              `${definition.catalog}:${definition.catalogId}`,
              definition.label ?? definition.catalogId,
            ] as const,
          ],
    ),
  );
  const definitionsById = new Map(
    definitions.map((definition) => [definition.id, definition]),
  );
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
    actions: gameplayActionViews(gameplay.actions, definitionsById),
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
      classDefinitionId: entity.classDefinitionId,
      featureDefinitionIds: entity.featureDefinitionIds,
      items: entity.items,
      equipment: entity.equipment,
      stats: entity.stats.map(
        (value) =>
          `${rulesetValueLabels.get(`stat:${value.id}`) ?? value.id} ${value.current}`,
      ),
      defenses: entity.defenses.map(
        (value) =>
          `${rulesetValueLabels.get(`defense:${value.id}`) ?? value.id} ${value.current}`,
      ),
      resources: entity.resources.map(
        (value) =>
          `${catalogLabels.get(`resource:${value.id}`) ?? value.id} ${value.current}/${value.maximum ?? 'unbounded'}`,
      ),
      modifiers: entity.modifiers.map(
        (modifier) =>
          `${catalogLabels.get(`modifier:${modifier.id}`) ?? modifier.id} ${modifier.value} (${modifier.remainingTurns} turns, ${modifier.stackingGroup})`,
      ),
    })),
    log: gameplay.log.map((entry) => ({
      sequence: entry.sequence,
      stateRevision: entry.stateRevision,
      actorId: entry.actorId,
      actionId: entry.actionId,
      itemBinding: entry.itemBinding,
      events: entry.events.map((event) => ({
        kind: event.kind,
        summary: event.summary,
        roll:
          event.roll === null
            ? null
            : {
                kind: event.roll.kind,
                dieResult: event.roll.dieResult,
                total: event.roll.total,
                thresholdLabel: event.roll.thresholdLabel,
                threshold: event.roll.threshold,
                outcome: event.roll.outcome,
                contributions: event.roll.contributions.map((contribution) => ({
                  sourceDefinitionId: contribution.sourceDefinitionId,
                  sourceLabel: contribution.sourceLabel,
                  amount: contribution.amount,
                  reasonKind: contribution.reasonKind,
                  contributionId: contribution.contributionId,
                  selector: contribution.selector,
                  condition: contribution.condition,
                })),
              },
      })),
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
      playBundle: gameplay.archive.playBundle,
      ruleset: gameplay.archive.ruleset,
      operationSchemas: gameplay.archive.operationSchemas,
      capabilitySchemas: gameplay.archive.capabilitySchemas,
      contentPacks: gameplay.archive.contentPacks,
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

function gameplayActionViews(
  actions: NonNullable<PlayWorkspaceResponseDto['gameplay']>['actions'],
  definitionsById: ReadonlyMap<
    string,
    PlayBundleArtifactSummaryDto['definitions'][number]
  >,
): readonly GameplayActionView[] {
  const definitionsWithBoundActions = new Set(
    actions.flatMap((action) =>
      action.itemBinding === null ? [] : [action.definitionId],
    ),
  );
  const viewsByIdentity = new Map<string, GameplayActionView>();
  for (const action of actions) {
    const isRedundantUnboundItemAction =
      action.itemBinding === null &&
      action.unavailable?.code === 'RPG_ACTION_ITEM_BINDING_UNAVAILABLE' &&
      definitionsWithBoundActions.has(action.definitionId);
    if (isRedundantUnboundItemAction) continue;

    const identity = gameplayActionIdentity(
      action.definitionId,
      action.itemBinding,
    );
    if (viewsByIdentity.has(identity)) continue;
    viewsByIdentity.set(identity, {
      identity,
      id: action.definitionId,
      name: action.label,
      description:
        definitionsById.get(action.definitionId)?.description ?? null,
      tags: definitionsById.get(action.definitionId)?.tags ?? [],
      itemBinding: action.itemBinding,
      available: action.available,
      unavailable:
        action.unavailable === null
          ? null
          : `${action.unavailable.code}: ${action.unavailable.message}`,
      maximumTargets: action.maximumTargets,
      candidateIds: action.options.participantIds,
      cellPaths: action.options.cellPaths.map((path) => ({
        destinationCellId: path.destinationCellId,
        cellIds: path.cellIds,
        movementCost: path.movementCost,
      })),
      areaIds: action.options.areaIds,
    });
  }
  return [...viewsByIdentity.values()];
}

function artifactInspection(
  artifact: PlayBundleArtifactSummaryDto,
): PlayBundleArtifactInspectionView {
  return {
    artifactId: artifact.artifactId,
    schema: `${artifact.schema.id}@${artifact.schema.version}`,
    playBundle: `${artifact.playBundle.id}@${artifact.playBundle.version}`,
    ruleset: `${artifact.ruleset.id}@${artifact.ruleset.version}`,
    language: `${artifact.language.id}@${artifact.language.version}`,
    contentPacks: artifact.contentPacks.map((source) => ({
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
    values: artifact.requiredValues,
    numericDomains: artifact.requiredNumericDomains,
    rulesetValues: artifact.rulesetValues,
    participantProfiles: artifact.participantProfiles,
    itemDefinitions: artifact.itemDefinitions,
    exportedRoots: artifact.exportedRoots,
    definitions: artifact.definitions.map((definition) => ({
      id: definition.id,
      fingerprint: definition.fingerprint,
      label: definition.label ?? definition.id,
      description: definition.description,
      tags: definition.tags,
      catalog: definition.catalog,
      catalogId: definition.catalogId,
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
  change: ContentPatchChangeDto,
): ContentPatchChangeView {
  return {
    plane: change.plane,
    path: change.path,
    transition: `${change.before} → ${change.after}`,
    effective: change.effective,
  };
}
