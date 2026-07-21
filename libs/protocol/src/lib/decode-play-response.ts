import type {
  ScenarioBoardDto,
  ScenarioBoundedValueDto,
  ScenarioCellCapabilityDto,
  ScenarioCellCapabilityValueDto,
  ScenarioCellDto,
  ScenarioRandomSourceDto,
  ScenarioInitialCapabilityDto,
  ScenarioParticipantDto,
  ScenarioSetupRequestDto,
  ScenarioTurnDto,
  GameplayActionOptionsDto,
  GameplayArchiveDto,
  GameplayAuthorityActionDto,
  GameplayTurnControlDto,
  GameplayEntityDto,
  GameplayEventDto,
  GameplayLogEntryDto,
  GameplayModifierDto,
  GameplayNamedValueDto,
  GameplayOutcomeDto,
  GameplayRandomEvidenceDto,
  GameplayRandomRequestDto,
  GameplayReactionDto,
  GameplayReactionOptionDto,
  GameplayResultDto,
  GameplayReplayBoundaryDto,
  GameplayReplayEntryDto,
  GameplaySessionDto,
  GameplayTraceDto,
  PlayBundleArtifactSummaryDto,
  ContentDefinitionDto,
  ContentDerivationProvenanceDto,
  PlayDiagnosticDto,
  PlayDiagnosticSourceDto,
  PlayBundleFingerprintDto,
  VersionedIdentityDto,
  PlayBundleLifecycleStatus,
  ContentPackLockEntryDto,
  ContentMixinProvenanceDto,
  ContentOverlayProvenanceDto,
  ContentPatchChangeDto,
  ContentRelationshipDto,
  VersionedRequirementDto,
  ContentPackSummaryDto,
  RulesetCatalogContentPackDto,
  RulesetCatalogDto,
  RulesetCatalogPlayBundleDto,
  RulesetCatalogResponseDto,
  RulesetLocationConfigDto,
  PlayBundleUpgradeDefinitionDto,
  PlayBundleUpgradeFieldDto,
  PlayBundleUpgradeImpactDto,
  PlayWorkspaceResponseDto,
} from '../generated/play-protocol.js';

export class PlayProtocolDecodeError extends Error {
  public constructor(path: string, message: string) {
    super(`${path}: ${message}`);
    this.name = 'PlayProtocolDecodeError';
  }
}

export function decodePlayWorkspaceResponse(
  value: unknown,
): PlayWorkspaceResponseDto {
  const record = requiredRecord(value, '$');
  exactKeys(
    record,
    [
      'ok',
      'status',
      'activeArtifact',
      'candidateArtifact',
      'upgradeImpact',
      'activationRevision',
      'hostRandomSource',
      'supportedRandomSources',
      'scenarioSetupRequired',
      'gameplayAvailable',
      'gameplay',
      'diagnostics',
    ],
    '$',
  );
  return {
    ok: requiredBoolean(record['ok'], '$.ok'),
    status: lifecycleStatus(record['status'], '$.status'),
    activeArtifact: nullableArtifact(
      record['activeArtifact'],
      '$.activeArtifact',
    ),
    candidateArtifact: nullableArtifact(
      record['candidateArtifact'],
      '$.candidateArtifact',
    ),
    upgradeImpact: nullableUpgradeImpact(
      record['upgradeImpact'],
      '$.upgradeImpact',
    ),
    activationRevision: nonNegativeInteger(
      record['activationRevision'],
      '$.activationRevision',
    ),
    hostRandomSource: scenarioRandomSource(
      record['hostRandomSource'],
      '$.hostRandomSource',
    ),
    supportedRandomSources: requiredArray(
      record['supportedRandomSources'],
      '$.supportedRandomSources',
    ).map((entry, index) =>
      scenarioRandomSource(entry, `$.supportedRandomSources[${index}]`),
    ),
    scenarioSetupRequired: requiredBoolean(
      record['scenarioSetupRequired'],
      '$.scenarioSetupRequired',
    ),
    gameplayAvailable: requiredBoolean(
      record['gameplayAvailable'],
      '$.gameplayAvailable',
    ),
    gameplay: nullableGameplay(record['gameplay'], '$.gameplay'),
    diagnostics: requiredArray(record['diagnostics'], '$.diagnostics').map(
      (entry, index) => diagnostic(entry, `$.diagnostics[${index}]`),
    ),
  };
}

export function decodeRulesetLocationConfig(
  value: unknown,
): RulesetLocationConfigDto {
  const record = requiredRecord(value, '$');
  exactKeys(record, ['schemaVersion', 'rulesets'], '$');
  const schemaVersion = nonNegativeInteger(
    record['schemaVersion'],
    '$.schemaVersion',
  );
  if (schemaVersion !== 1) {
    throw new PlayProtocolDecodeError('$.schemaVersion', 'expected version 1');
  }
  return {
    schemaVersion,
    rulesets: requiredArray(record['rulesets'], '$.rulesets').map(
      (entry, index) => {
        const path = `$.rulesets[${index}]`;
        const location = requiredRecord(entry, path);
        exactKeys(location, ['id', 'label', 'rulesetRoot'], path);
        return {
          id: requiredString(location['id'], `${path}.id`),
          label: requiredString(location['label'], `${path}.label`),
          rulesetRoot: requiredString(
            location['rulesetRoot'],
            `${path}.rulesetRoot`,
          ),
        };
      },
    ),
  };
}

export function decodeRulesetCatalogResponse(
  value: unknown,
): RulesetCatalogResponseDto {
  const record = requiredRecord(value, '$');
  exactKeys(record, ['ok', 'catalog', 'diagnostics'], '$');
  return {
    ok: requiredBoolean(record['ok'], '$.ok'),
    catalog:
      record['catalog'] === null
        ? null
        : rulesetCatalog(record['catalog'], '$.catalog'),
    diagnostics: requiredArray(record['diagnostics'], '$.diagnostics').map(
      (entry, index) => diagnostic(entry, `$.diagnostics[${index}]`),
    ),
  };
}

function rulesetCatalog(value: unknown, path: string): RulesetCatalogDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['rulesetRoot', 'ruleset', 'contentPacks', 'playBundles'],
    path,
  );
  return {
    rulesetRoot: requiredString(record['rulesetRoot'], `${path}.rulesetRoot`),
    ruleset: identity(record['ruleset'], `${path}.ruleset`),
    contentPacks: requiredArray(
      record['contentPacks'],
      `${path}.contentPacks`,
    ).map((entry, index) =>
      catalogContentPack(entry, `${path}.contentPacks[${index}]`),
    ),
    playBundles: requiredArray(
      record['playBundles'],
      `${path}.playBundles`,
    ).map((entry, index) =>
      catalogPlayBundle(entry, `${path}.playBundles[${index}]`),
    ),
  };
}

function catalogContentPack(
  value: unknown,
  path: string,
): RulesetCatalogContentPackDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version', 'label', 'requirements'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: requiredString(record['version'], `${path}.version`),
    label: requiredString(record['label'], `${path}.label`),
    requirements: stringArray(record['requirements'], `${path}.requirements`),
  };
}

function catalogPlayBundle(
  value: unknown,
  path: string,
): RulesetCatalogPlayBundleDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['id', 'version', 'contentPackIds', 'compatible', 'diagnostics'],
    path,
  );
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: requiredString(record['version'], `${path}.version`),
    contentPackIds: stringArray(
      record['contentPackIds'],
      `${path}.contentPackIds`,
    ),
    compatible: requiredBoolean(record['compatible'], `${path}.compatible`),
    diagnostics: requiredArray(
      record['diagnostics'],
      `${path}.diagnostics`,
    ).map((entry, index) => diagnostic(entry, `${path}.diagnostics[${index}]`)),
  };
}

export function decodeScenarioDocument(
  value: unknown,
): ScenarioSetupRequestDto {
  const record = requiredRecord(value, '$');
  exactKeys(
    record,
    ['schema', 'playBundleId', 'board', 'participants', 'turn', 'randomSource'],
    '$',
  );
  const schema = requiredRecord(record['schema'], '$.schema');
  exactKeys(schema, ['id', 'version'], '$.schema');
  return {
    schema: {
      id: requiredString(schema['id'], '$.schema.id'),
      version: nonNegativeInteger(schema['version'], '$.schema.version'),
    },
    playBundleId: requiredString(record['playBundleId'], '$.playBundleId'),
    board: scenarioBoard(record['board'], '$.board'),
    participants: requiredArray(record['participants'], '$.participants').map(
      (entry, index) => scenarioParticipant(entry, `$.participants[${index}]`),
    ),
    turn: scenarioTurn(record['turn'], '$.turn'),
    randomSource: scenarioRandomSource(
      record['randomSource'],
      '$.randomSource',
    ),
  };
}

function nullableGameplay(
  value: unknown,
  path: string,
): GameplaySessionDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'artifactId',
      'actorId',
      'stateRevision',
      'acceptedRandomValues',
      'randomSource',
      'board',
      'turn',
      'actions',
      'controls',
      'entities',
      'pendingReaction',
      'log',
      'outcome',
      'lastResult',
      'archive',
    ],
    path,
  );
  return {
    artifactId: requiredString(record['artifactId'], `${path}.artifactId`),
    actorId: requiredString(record['actorId'], `${path}.actorId`),
    stateRevision: nonNegativeInteger(
      record['stateRevision'],
      `${path}.stateRevision`,
    ),
    acceptedRandomValues: nonNegativeIntegerString(
      record['acceptedRandomValues'],
      `${path}.acceptedRandomValues`,
    ),
    randomSource: scenarioRandomSource(
      record['randomSource'],
      `${path}.randomSource`,
    ),
    board: scenarioBoard(record['board'], `${path}.board`),
    turn: scenarioTurn(record['turn'], `${path}.turn`),
    actions: requiredArray(record['actions'], `${path}.actions`).map(
      (entry, index) =>
        gameplayAuthorityAction(entry, `${path}.actions[${index}]`),
    ),
    controls: requiredArray(record['controls'], `${path}.controls`).map(
      (entry, index) =>
        gameplayTurnControl(entry, `${path}.controls[${index}]`),
    ),
    entities: requiredArray(record['entities'], `${path}.entities`).map(
      (entry, index) => gameplayEntity(entry, `${path}.entities[${index}]`),
    ),
    pendingReaction: nullableReaction(
      record['pendingReaction'],
      `${path}.pendingReaction`,
    ),
    log: requiredArray(record['log'], `${path}.log`).map((entry, index) =>
      gameplayLogEntry(entry, `${path}.log[${index}]`),
    ),
    outcome: gameplayOutcome(record['outcome'], `${path}.outcome`),
    lastResult: nullableResult(record['lastResult'], `${path}.lastResult`),
    archive: gameplayArchive(record['archive'], `${path}.archive`),
  };
}

function scenarioParticipant(
  value: unknown,
  path: string,
): ScenarioParticipantDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['id', 'label', 'teamId', 'position', 'definitionIds', 'capabilities'],
    path,
  );
  const position = requiredRecord(record['position'], `${path}.position`);
  exactKeys(position, ['x', 'y'], `${path}.position`);
  return {
    id: requiredString(record['id'], `${path}.id`),
    label: requiredString(record['label'], `${path}.label`),
    teamId: requiredString(record['teamId'], `${path}.teamId`),
    position: {
      x: nonNegativeInteger(position['x'], `${path}.position.x`),
      y: nonNegativeInteger(position['y'], `${path}.position.y`),
    },
    definitionIds: stringArray(
      record['definitionIds'],
      `${path}.definitionIds`,
    ),
    capabilities: requiredArray(
      record['capabilities'],
      `${path}.capabilities`,
    ).map((entry, index) =>
      scenarioInitialCapability(entry, `${path}.capabilities[${index}]`),
    ),
  };
}

function scenarioInitialCapability(
  value: unknown,
  path: string,
): ScenarioInitialCapabilityDto {
  const record = requiredRecord(value, path);
  const owner = requiredString(record['owner'], `${path}.owner`);
  switch (owner) {
    case 'vitality':
      exactKeys(record, ['owner', 'value'], path);
      return {
        owner,
        value: scenarioBoundedValue(record['value'], `${path}.value`),
      };
    case 'stat':
    case 'defense':
      exactKeys(record, ['owner', 'id', 'value'], path);
      return {
        owner,
        id: requiredString(record['id'], `${path}.id`),
        value: requiredInteger(record['value'], `${path}.value`),
      };
    case 'resource':
      exactKeys(record, ['owner', 'id', 'value'], path);
      return {
        owner,
        id: requiredString(record['id'], `${path}.id`),
        value: scenarioBoundedValue(record['value'], `${path}.value`),
      };
    case 'modifier':
      exactKeys(
        record,
        ['owner', 'stackingGroup', 'id', 'value', 'remainingTurns'],
        path,
      );
      return {
        owner,
        stackingGroup: requiredString(
          record['stackingGroup'],
          `${path}.stackingGroup`,
        ),
        id: requiredString(record['id'], `${path}.id`),
        value: requiredInteger(record['value'], `${path}.value`),
        remainingTurns: nonNegativeInteger(
          record['remainingTurns'],
          `${path}.remainingTurns`,
        ),
      };
    default:
      throw new PlayProtocolDecodeError(
        path,
        `unknown capability owner ${owner}`,
      );
  }
}

function scenarioBoundedValue(
  value: unknown,
  path: string,
): ScenarioBoundedValueDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['current', 'max'], path);
  return {
    current: requiredInteger(record['current'], `${path}.current`),
    max: requiredInteger(record['max'], `${path}.max`),
  };
}

function scenarioRandomSource(
  value: unknown,
  path: string,
): ScenarioRandomSourceDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['policyId', 'policyVersion', 'sourceId', 'sourceVersion'],
    path,
  );
  return {
    policyId: requiredString(record['policyId'], `${path}.policyId`),
    policyVersion: nonNegativeInteger(
      record['policyVersion'],
      `${path}.policyVersion`,
    ),
    sourceId: requiredString(record['sourceId'], `${path}.sourceId`),
    sourceVersion: nonNegativeInteger(
      record['sourceVersion'],
      `${path}.sourceVersion`,
    ),
  };
}

function scenarioBoard(value: unknown, path: string): ScenarioBoardDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['width', 'height', 'cells'], path);
  return {
    width: nonNegativeInteger(record['width'], `${path}.width`),
    height: nonNegativeInteger(record['height'], `${path}.height`),
    cells: requiredArray(record['cells'], `${path}.cells`).map((entry, index) =>
      scenarioCell(entry, `${path}.cells[${index}]`),
    ),
  };
}

function scenarioCell(value: unknown, path: string): ScenarioCellDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'position', 'capabilities'], path);
  const position = requiredRecord(record['position'], `${path}.position`);
  exactKeys(position, ['x', 'y'], `${path}.position`);
  return {
    id: requiredString(record['id'], `${path}.id`),
    position: {
      x: nonNegativeInteger(position['x'], `${path}.position.x`),
      y: nonNegativeInteger(position['y'], `${path}.position.y`),
    },
    capabilities: requiredArray(
      record['capabilities'],
      `${path}.capabilities`,
    ).map((entry, index) =>
      encounterCellCapability(entry, `${path}.capabilities[${index}]`),
    ),
  };
}

function encounterCellCapability(
  value: unknown,
  path: string,
): ScenarioCellCapabilityDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version', 'definitionId', 'value'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: nonNegativeInteger(record['version'], `${path}.version`),
    definitionId: nullableString(
      record['definitionId'],
      `${path}.definitionId`,
    ),
    value: encounterCellCapabilityValue(record['value'], `${path}.value`),
  };
}

function encounterCellCapabilityValue(
  value: unknown,
  path: string,
): ScenarioCellCapabilityValueDto {
  const record = requiredRecord(value, path);
  const kind = requiredString(record['kind'], `${path}.kind`);
  switch (kind) {
    case 'traversal':
      exactKeys(record, ['kind', 'passable', 'movementCost'], path);
      return {
        kind,
        passable: requiredBoolean(record['passable'], `${path}.passable`),
        movementCost: nonNegativeInteger(
          record['movementCost'],
          `${path}.movementCost`,
        ),
      };
    case 'flag':
      exactKeys(record, ['kind', 'value'], path);
      return {
        kind,
        value: requiredBoolean(record['value'], `${path}.value`),
      };
    case 'integer':
      exactKeys(record, ['kind', 'value'], path);
      return {
        kind,
        value: requiredInteger(record['value'], `${path}.value`),
      };
    case 'identifier':
      exactKeys(record, ['kind', 'valueId'], path);
      return {
        kind,
        valueId: requiredString(record['valueId'], `${path}.valueId`),
      };
    default:
      throw new PlayProtocolDecodeError(path, `unknown cell value ${kind}`);
  }
}

function scenarioTurn(value: unknown, path: string): ScenarioTurnDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['initiativeOrder', 'currentActorId', 'round', 'turn'],
    path,
  );
  return {
    initiativeOrder: stringArray(
      record['initiativeOrder'],
      `${path}.initiativeOrder`,
    ),
    currentActorId: requiredString(
      record['currentActorId'],
      `${path}.currentActorId`,
    ),
    round: nonNegativeInteger(record['round'], `${path}.round`),
    turn: nonNegativeInteger(record['turn'], `${path}.turn`),
  };
}

function gameplayLogEntry(value: unknown, path: string): GameplayLogEntryDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['sequence', 'stateRevision', 'actorId', 'actionId', 'events'],
    path,
  );
  return {
    sequence: nonNegativeIntegerString(record['sequence'], `${path}.sequence`),
    stateRevision: nonNegativeIntegerString(
      record['stateRevision'],
      `${path}.stateRevision`,
    ),
    actorId: requiredString(record['actorId'], `${path}.actorId`),
    actionId: requiredString(record['actionId'], `${path}.actionId`),
    events: requiredArray(record['events'], `${path}.events`).map(
      (entry, index) => gameplayEvent(entry, `${path}.events[${index}]`),
    ),
  };
}

function gameplayOutcome(value: unknown, path: string): GameplayOutcomeDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['status', 'winningTeamIds'], path);
  return {
    status: requiredString(record['status'], `${path}.status`),
    winningTeamIds: stringArray(
      record['winningTeamIds'],
      `${path}.winningTeamIds`,
    ),
  };
}

function gameplayArchive(value: unknown, path: string): GameplayArchiveDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'checkpointSchema',
      'replaySchemaVersion',
      'eventSchemaVersion',
      'artifactId',
      'artifactSchema',
      'playBundle',
      'ruleset',
      'operationSchemas',
      'capabilitySchemas',
      'contentPacks',
      'dependencyLock',
      'fingerprints',
      'definitionFingerprints',
      'stateRevision',
      'acceptedRandomPosition',
      'phase',
      'stateHash',
      'checkpointBytes',
      'replayEntries',
      'verificationStatus',
      'verificationMessage',
    ],
    path,
  );
  return {
    checkpointSchema: requiredString(
      record['checkpointSchema'],
      `${path}.checkpointSchema`,
    ),
    replaySchemaVersion: nonNegativeInteger(
      record['replaySchemaVersion'],
      `${path}.replaySchemaVersion`,
    ),
    eventSchemaVersion: nonNegativeInteger(
      record['eventSchemaVersion'],
      `${path}.eventSchemaVersion`,
    ),
    artifactId: requiredString(record['artifactId'], `${path}.artifactId`),
    artifactSchema: requiredString(
      record['artifactSchema'],
      `${path}.artifactSchema`,
    ),
    playBundle: requiredString(record['playBundle'], `${path}.playBundle`),
    ruleset: requiredString(record['ruleset'], `${path}.ruleset`),
    operationSchemas: stringArray(
      record['operationSchemas'],
      `${path}.operationSchemas`,
    ),
    capabilitySchemas: stringArray(
      record['capabilitySchemas'],
      `${path}.capabilitySchemas`,
    ),
    contentPacks: stringArray(record['contentPacks'], `${path}.contentPacks`),
    dependencyLock: stringArray(
      record['dependencyLock'],
      `${path}.dependencyLock`,
    ),
    fingerprints: fingerprints(record['fingerprints'], `${path}.fingerprints`),
    definitionFingerprints: stringArray(
      record['definitionFingerprints'],
      `${path}.definitionFingerprints`,
    ),
    stateRevision: nonNegativeIntegerString(
      record['stateRevision'],
      `${path}.stateRevision`,
    ),
    acceptedRandomPosition: nonNegativeIntegerString(
      record['acceptedRandomPosition'],
      `${path}.acceptedRandomPosition`,
    ),
    phase: requiredString(record['phase'], `${path}.phase`),
    stateHash: requiredString(record['stateHash'], `${path}.stateHash`),
    checkpointBytes: nonNegativeInteger(
      record['checkpointBytes'],
      `${path}.checkpointBytes`,
    ),
    replayEntries: requiredArray(
      record['replayEntries'],
      `${path}.replayEntries`,
    ).map((entry, index) =>
      gameplayReplayEntry(entry, `${path}.replayEntries[${index}]`),
    ),
    verificationStatus: requiredString(
      record['verificationStatus'],
      `${path}.verificationStatus`,
    ),
    verificationMessage: requiredString(
      record['verificationMessage'],
      `${path}.verificationMessage`,
    ),
  };
}

function gameplayReplayEntry(
  value: unknown,
  path: string,
): GameplayReplayEntryDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'sequence',
      'operation',
      'outcome',
      'before',
      'after',
      'randomEvidence',
      'events',
    ],
    path,
  );
  return {
    sequence: nonNegativeInteger(record['sequence'], `${path}.sequence`),
    operation: requiredString(record['operation'], `${path}.operation`),
    outcome: requiredString(record['outcome'], `${path}.outcome`),
    before: gameplayReplayBoundary(record['before'], `${path}.before`),
    after: gameplayReplayBoundary(record['after'], `${path}.after`),
    randomEvidence: stringArray(
      record['randomEvidence'],
      `${path}.randomEvidence`,
    ),
    events: stringArray(record['events'], `${path}.events`),
  };
}

function gameplayReplayBoundary(
  value: unknown,
  path: string,
): GameplayReplayBoundaryDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['revision', 'acceptedRandomPosition', 'phase', 'stateHash'],
    path,
  );
  return {
    revision: nonNegativeIntegerString(record['revision'], `${path}.revision`),
    acceptedRandomPosition: nonNegativeIntegerString(
      record['acceptedRandomPosition'],
      `${path}.acceptedRandomPosition`,
    ),
    phase: requiredString(record['phase'], `${path}.phase`),
    stateHash: requiredString(record['stateHash'], `${path}.stateHash`),
  };
}

function gameplayAuthorityAction(
  value: unknown,
  path: string,
): GameplayAuthorityActionDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'definitionId',
      'label',
      'available',
      'unavailable',
      'maximumTargets',
      'options',
    ],
    path,
  );
  return {
    definitionId: requiredString(
      record['definitionId'],
      `${path}.definitionId`,
    ),
    label: requiredString(record['label'], `${path}.label`),
    available: requiredBoolean(record['available'], `${path}.available`),
    unavailable:
      record['unavailable'] === null
        ? null
        : gameplayUnavailable(record['unavailable'], `${path}.unavailable`),
    maximumTargets: nonNegativeInteger(
      record['maximumTargets'],
      `${path}.maximumTargets`,
    ),
    options: gameplayActionOptions(record['options'], `${path}.options`),
  };
}

function gameplayTurnControl(
  value: unknown,
  path: string,
): GameplayTurnControlDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'label', 'available', 'unavailable'], path);
  return {
    kind: requiredString(record['kind'], `${path}.kind`),
    label: requiredString(record['label'], `${path}.label`),
    available: requiredBoolean(record['available'], `${path}.available`),
    unavailable:
      record['unavailable'] === null
        ? null
        : gameplayUnavailable(record['unavailable'], `${path}.unavailable`),
  };
}

function gameplayUnavailable(value: unknown, path: string) {
  const record = requiredRecord(value, path);
  exactKeys(record, ['code', 'path', 'message'], path);
  return {
    code: requiredString(record['code'], `${path}.code`),
    path: requiredString(record['path'], `${path}.path`),
    message: requiredString(record['message'], `${path}.message`),
  };
}

function gameplayActionOptions(
  value: unknown,
  path: string,
): GameplayActionOptionsDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['participantIds', 'cellIds', 'areaIds'], path);
  return {
    participantIds: stringArray(
      record['participantIds'],
      `${path}.participantIds`,
    ),
    cellIds: stringArray(record['cellIds'], `${path}.cellIds`),
    areaIds: stringArray(record['areaIds'], `${path}.areaIds`),
  };
}

function gameplayRandomRequest(
  value: unknown,
  path: string,
): GameplayRandomRequestDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'count', 'sides', 'path'], path);
  return {
    kind: requiredString(record['kind'], `${path}.kind`),
    count: nonNegativeInteger(record['count'], `${path}.count`),
    sides: nonNegativeInteger(record['sides'], `${path}.sides`),
    path: requiredString(record['path'], `${path}.path`),
  };
}

function gameplayEntity(value: unknown, path: string): GameplayEntityDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'id',
      'label',
      'teamId',
      'x',
      'y',
      'definitionIds',
      'vitality',
      'stats',
      'defenses',
      'resources',
      'modifiers',
    ],
    path,
  );
  return {
    id: requiredString(record['id'], `${path}.id`),
    label: requiredString(record['label'], `${path}.label`),
    teamId: requiredString(record['teamId'], `${path}.teamId`),
    x: nonNegativeInteger(record['x'], `${path}.x`),
    y: nonNegativeInteger(record['y'], `${path}.y`),
    definitionIds: stringArray(
      record['definitionIds'],
      `${path}.definitionIds`,
    ),
    vitality: gameplayNamedValue(record['vitality'], `${path}.vitality`),
    stats: gameplayNamedValues(record['stats'], `${path}.stats`),
    defenses: gameplayNamedValues(record['defenses'], `${path}.defenses`),
    resources: gameplayNamedValues(record['resources'], `${path}.resources`),
    modifiers: requiredArray(record['modifiers'], `${path}.modifiers`).map(
      (entry, index) => gameplayModifier(entry, `${path}.modifiers[${index}]`),
    ),
  };
}

function gameplayNamedValues(
  value: unknown,
  path: string,
): GameplayNamedValueDto[] {
  return requiredArray(value, path).map((entry, index) =>
    gameplayNamedValue(entry, `${path}[${index}]`),
  );
}

function gameplayNamedValue(
  value: unknown,
  path: string,
): GameplayNamedValueDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'current', 'maximum'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    current: requiredInteger(record['current'], `${path}.current`),
    maximum: nullableInteger(record['maximum'], `${path}.maximum`),
  };
}

function gameplayModifier(value: unknown, path: string): GameplayModifierDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['stackingGroup', 'id', 'value', 'remainingTurns'], path);
  return {
    stackingGroup: requiredString(
      record['stackingGroup'],
      `${path}.stackingGroup`,
    ),
    id: requiredString(record['id'], `${path}.id`),
    value: requiredInteger(record['value'], `${path}.value`),
    remainingTurns: nonNegativeInteger(
      record['remainingTurns'],
      `${path}.remainingTurns`,
    ),
  };
}

function nullableReaction(
  value: unknown,
  path: string,
): GameplayReactionDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['reactionId', 'actorId', 'targetId', 'actionId', 'options', 'path'],
    path,
  );
  return {
    reactionId: requiredString(record['reactionId'], `${path}.reactionId`),
    actorId: requiredString(record['actorId'], `${path}.actorId`),
    targetId: requiredString(record['targetId'], `${path}.targetId`),
    actionId: requiredString(record['actionId'], `${path}.actionId`),
    options: requiredArray(record['options'], `${path}.options`).map(
      (entry, index) =>
        gameplayReactionOption(entry, `${path}.options[${index}]`),
    ),
    path: requiredString(record['path'], `${path}.path`),
  };
}

function gameplayReactionOption(
  value: unknown,
  path: string,
): GameplayReactionOptionDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'label', 'damageReduction'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    label: requiredString(record['label'], `${path}.label`),
    damageReduction: nonNegativeInteger(
      record['damageReduction'],
      `${path}.damageReduction`,
    ),
  };
}

function nullableResult(
  value: unknown,
  path: string,
): GameplayResultDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'status',
      'code',
      'message',
      'events',
      'trace',
      'randomConsumed',
      'randomEvidence',
      'stateRevision',
      'randomRequest',
    ],
    path,
  );
  return {
    status: requiredString(record['status'], `${path}.status`),
    code: nullableString(record['code'], `${path}.code`),
    message: requiredString(record['message'], `${path}.message`),
    events: requiredArray(record['events'], `${path}.events`).map(
      (entry, index) => gameplayEvent(entry, `${path}.events[${index}]`),
    ),
    trace: requiredArray(record['trace'], `${path}.trace`).map((entry, index) =>
      gameplayTrace(entry, `${path}.trace[${index}]`),
    ),
    randomConsumed: nonNegativeIntegerString(
      record['randomConsumed'],
      `${path}.randomConsumed`,
    ),
    randomEvidence: requiredArray(
      record['randomEvidence'],
      `${path}.randomEvidence`,
    ).map((entry, index) =>
      gameplayRandomEvidence(entry, `${path}.randomEvidence[${index}]`),
    ),
    stateRevision: nonNegativeInteger(
      record['stateRevision'],
      `${path}.stateRevision`,
    ),
    randomRequest:
      record['randomRequest'] === null
        ? null
        : gameplayRandomRequest(
            record['randomRequest'],
            `${path}.randomRequest`,
          ),
  };
}

function gameplayRandomEvidence(
  value: unknown,
  path: string,
): GameplayRandomEvidenceDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'count', 'sides', 'path', 'values'], path);
  return {
    kind: requiredString(record['kind'], `${path}.kind`),
    count: nonNegativeInteger(record['count'], `${path}.count`),
    sides: nonNegativeInteger(record['sides'], `${path}.sides`),
    path: requiredString(record['path'], `${path}.path`),
    values: requiredArray(record['values'], `${path}.values`).map(
      (entry, index) => nonNegativeInteger(entry, `${path}.values[${index}]`),
    ),
  };
}

function gameplayEvent(value: unknown, path: string): GameplayEventDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'summary'], path);
  return {
    kind: requiredString(record['kind'], `${path}.kind`),
    summary: requiredString(record['summary'], `${path}.summary`),
  };
}

function gameplayTrace(value: unknown, path: string): GameplayTraceDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['path', 'code', 'detail'], path);
  return {
    path: requiredString(record['path'], `${path}.path`),
    code: requiredString(record['code'], `${path}.code`),
    detail: requiredString(record['detail'], `${path}.detail`),
  };
}

function nullableUpgradeImpact(
  value: unknown,
  path: string,
): PlayBundleUpgradeImpactDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['fromArtifactId', 'toArtifactId', 'sourceChanges', 'definitions'],
    path,
  );
  return {
    fromArtifactId: requiredString(
      record['fromArtifactId'],
      `${path}.fromArtifactId`,
    ),
    toArtifactId: requiredString(
      record['toArtifactId'],
      `${path}.toArtifactId`,
    ),
    sourceChanges: stringArray(
      record['sourceChanges'],
      `${path}.sourceChanges`,
    ),
    definitions: requiredArray(
      record['definitions'],
      `${path}.definitions`,
    ).map((entry, index) =>
      upgradeDefinition(entry, `${path}.definitions[${index}]`),
    ),
  };
}

function upgradeDefinition(
  value: unknown,
  path: string,
): PlayBundleUpgradeDefinitionDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['definitionId', 'change', 'descendant', 'causes', 'fields'],
    path,
  );
  return {
    definitionId: requiredString(
      record['definitionId'],
      `${path}.definitionId`,
    ),
    change: requiredString(record['change'], `${path}.change`),
    descendant: requiredBoolean(record['descendant'], `${path}.descendant`),
    causes: stringArray(record['causes'], `${path}.causes`),
    fields: requiredArray(record['fields'], `${path}.fields`).map(
      (entry, index) => upgradeField(entry, `${path}.fields[${index}]`),
    ),
  };
}

function upgradeField(value: unknown, path: string): PlayBundleUpgradeFieldDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['plane', 'path', 'before', 'after'], path);
  return {
    plane: requiredString(record['plane'], `${path}.plane`),
    path: requiredString(record['path'], `${path}.path`),
    before: requiredString(record['before'], `${path}.before`),
    after: requiredString(record['after'], `${path}.after`),
  };
}

function nullableArtifact(
  value: unknown,
  path: string,
): PlayBundleArtifactSummaryDto | null {
  return value === null ? null : artifact(value, path);
}

function artifact(value: unknown, path: string): PlayBundleArtifactSummaryDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'schema',
      'artifactId',
      'playBundle',
      'ruleset',
      'language',
      'contentPacks',
      'dependencyLock',
      'requiredOperations',
      'requiredCapabilities',
      'requiredValues',
      'requiredNumericDomains',
      'exportedRoots',
      'definitions',
      'policyBindingIds',
      'relationships',
      'derivationSlots',
      'overlaySlots',
      'derivations',
      'overlays',
      'fingerprints',
    ],
    path,
  );
  return {
    schema: identity(record['schema'], `${path}.schema`),
    artifactId: requiredString(record['artifactId'], `${path}.artifactId`),
    playBundle: identity(record['playBundle'], `${path}.playBundle`),
    ruleset: identity(record['ruleset'], `${path}.ruleset`),
    language: identity(record['language'], `${path}.language`),
    contentPacks: requiredArray(
      record['contentPacks'],
      `${path}.contentPacks`,
    ).map((entry, index) =>
      sourcePackage(entry, `${path}.contentPacks[${index}]`),
    ),
    dependencyLock: requiredArray(
      record['dependencyLock'],
      `${path}.dependencyLock`,
    ).map((entry, index) =>
      lockEntry(entry, `${path}.dependencyLock[${index}]`),
    ),
    requiredOperations: requiredArray(
      record['requiredOperations'],
      `${path}.requiredOperations`,
    ).map((entry, index) =>
      requirement(entry, `${path}.requiredOperations[${index}]`),
    ),
    requiredCapabilities: requiredArray(
      record['requiredCapabilities'],
      `${path}.requiredCapabilities`,
    ).map((entry, index) =>
      requirement(entry, `${path}.requiredCapabilities[${index}]`),
    ),
    requiredValues: stringArray(
      record['requiredValues'],
      `${path}.requiredValues`,
    ),
    requiredNumericDomains: stringArray(
      record['requiredNumericDomains'],
      `${path}.requiredNumericDomains`,
    ),
    exportedRoots: stringArray(
      record['exportedRoots'],
      `${path}.exportedRoots`,
    ),
    definitions: requiredArray(
      record['definitions'],
      `${path}.definitions`,
    ).map((entry, index) => definition(entry, `${path}.definitions[${index}]`)),
    policyBindingIds: stringArray(
      record['policyBindingIds'],
      `${path}.policyBindingIds`,
    ),
    relationships: requiredArray(
      record['relationships'],
      `${path}.relationships`,
    ).map((entry, index) =>
      relationship(entry, `${path}.relationships[${index}]`),
    ),
    derivationSlots: nonNegativeInteger(
      record['derivationSlots'],
      `${path}.derivationSlots`,
    ),
    overlaySlots: nonNegativeInteger(
      record['overlaySlots'],
      `${path}.overlaySlots`,
    ),
    derivations: requiredArray(
      record['derivations'],
      `${path}.derivations`,
    ).map((entry, index) => derivation(entry, `${path}.derivations[${index}]`)),
    overlays: requiredArray(record['overlays'], `${path}.overlays`).map(
      (entry, index) => overlay(entry, `${path}.overlays[${index}]`),
    ),
    fingerprints: fingerprints(record['fingerprints'], `${path}.fingerprints`),
  };
}

function derivation(
  value: unknown,
  path: string,
): ContentDerivationProvenanceDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'definitionId',
      'owner',
      'base',
      'baseFingerprint',
      'mixins',
      'localPatchFingerprint',
      'materializedFingerprint',
      'changes',
    ],
    path,
  );
  return {
    definitionId: requiredString(
      record['definitionId'],
      `${path}.definitionId`,
    ),
    owner: requiredString(record['owner'], `${path}.owner`),
    base: requiredString(record['base'], `${path}.base`),
    baseFingerprint: requiredString(
      record['baseFingerprint'],
      `${path}.baseFingerprint`,
    ),
    mixins: requiredArray(record['mixins'], `${path}.mixins`).map(
      (entry, index) => mixin(entry, `${path}.mixins[${index}]`),
    ),
    localPatchFingerprint: requiredString(
      record['localPatchFingerprint'],
      `${path}.localPatchFingerprint`,
    ),
    materializedFingerprint: requiredString(
      record['materializedFingerprint'],
      `${path}.materializedFingerprint`,
    ),
    changes: requiredArray(record['changes'], `${path}.changes`).map(
      (entry, index) => patchChange(entry, `${path}.changes[${index}]`),
    ),
  };
}

function mixin(value: unknown, path: string): ContentMixinProvenanceDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['identity', 'fingerprint', 'parameters', 'order'], path);
  return {
    identity: requiredString(record['identity'], `${path}.identity`),
    fingerprint: requiredString(record['fingerprint'], `${path}.fingerprint`),
    parameters: stringArray(record['parameters'], `${path}.parameters`),
    order: nonNegativeInteger(record['order'], `${path}.order`),
  };
}

function overlay(value: unknown, path: string): ContentOverlayProvenanceDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'overlay',
      'target',
      'expectedFingerprint',
      'beforeFingerprint',
      'afterFingerprint',
      'plane',
      'conflictPolicy',
      'patchFingerprint',
      'order',
      'changes',
    ],
    path,
  );
  return {
    overlay: requiredString(record['overlay'], `${path}.overlay`),
    target: requiredString(record['target'], `${path}.target`),
    expectedFingerprint: requiredString(
      record['expectedFingerprint'],
      `${path}.expectedFingerprint`,
    ),
    beforeFingerprint: requiredString(
      record['beforeFingerprint'],
      `${path}.beforeFingerprint`,
    ),
    afterFingerprint: requiredString(
      record['afterFingerprint'],
      `${path}.afterFingerprint`,
    ),
    plane: requiredString(record['plane'], `${path}.plane`),
    conflictPolicy: requiredString(
      record['conflictPolicy'],
      `${path}.conflictPolicy`,
    ),
    patchFingerprint: requiredString(
      record['patchFingerprint'],
      `${path}.patchFingerprint`,
    ),
    order: nonNegativeInteger(record['order'], `${path}.order`),
    changes: requiredArray(record['changes'], `${path}.changes`).map(
      (entry, index) => patchChange(entry, `${path}.changes[${index}]`),
    ),
  };
}

function patchChange(value: unknown, path: string): ContentPatchChangeDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['plane', 'path', 'before', 'after', 'effective'], path);
  return {
    plane: requiredString(record['plane'], `${path}.plane`),
    path: requiredString(record['path'], `${path}.path`),
    before: requiredString(record['before'], `${path}.before`),
    after: requiredString(record['after'], `${path}.after`),
    effective: requiredBoolean(record['effective'], `${path}.effective`),
  };
}

function diagnostic(value: unknown, path: string): PlayDiagnosticDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'stage',
      'severity',
      'code',
      'path',
      'message',
      'packageId',
      'definitionId',
      'source',
      'graphPath',
      'expected',
      'actual',
    ],
    path,
  );
  return {
    stage: requiredString(record['stage'], `${path}.stage`),
    severity: requiredString(record['severity'], `${path}.severity`),
    code: requiredString(record['code'], `${path}.code`),
    path: requiredString(record['path'], `${path}.path`),
    message: requiredString(record['message'], `${path}.message`),
    packageId: nullableString(record['packageId'], `${path}.packageId`),
    definitionId: nullableString(
      record['definitionId'],
      `${path}.definitionId`,
    ),
    source: nullableDiagnosticSource(record['source'], `${path}.source`),
    graphPath: nullableStringArray(record['graphPath'], `${path}.graphPath`),
    expected: nullableString(record['expected'], `${path}.expected`),
    actual: nullableString(record['actual'], `${path}.actual`),
  };
}

function nullableDiagnosticSource(
  value: unknown,
  path: string,
): PlayDiagnosticSourceDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(record, ['module', 'declaration'], path);
  return {
    module: requiredString(record['module'], `${path}.module`),
    declaration: requiredString(record['declaration'], `${path}.declaration`),
  };
}

function identity(value: unknown, path: string): VersionedIdentityDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: requiredString(record['version'], `${path}.version`),
  };
}

function sourcePackage(value: unknown, path: string): ContentPackSummaryDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version', 'sourceFingerprint'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: requiredString(record['version'], `${path}.version`),
    sourceFingerprint: requiredString(
      record['sourceFingerprint'],
      `${path}.sourceFingerprint`,
    ),
  };
}

function lockEntry(value: unknown, path: string): ContentPackLockEntryDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'requester',
      'packageId',
      'requestedVersion',
      'resolvedVersion',
      'sourceFingerprint',
      'importAs',
      'relationship',
    ],
    path,
  );
  return {
    requester: requiredString(record['requester'], `${path}.requester`),
    packageId: requiredString(record['packageId'], `${path}.packageId`),
    requestedVersion: requiredString(
      record['requestedVersion'],
      `${path}.requestedVersion`,
    ),
    resolvedVersion: requiredString(
      record['resolvedVersion'],
      `${path}.resolvedVersion`,
    ),
    sourceFingerprint: requiredString(
      record['sourceFingerprint'],
      `${path}.sourceFingerprint`,
    ),
    importAs: requiredString(record['importAs'], `${path}.importAs`),
    relationship: requiredString(
      record['relationship'],
      `${path}.relationship`,
    ),
  };
}

function requirement(value: unknown, path: string): VersionedRequirementDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: nonNegativeInteger(record['version'], `${path}.version`),
  };
}

function definition(value: unknown, path: string): ContentDefinitionDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'id',
      'fingerprint',
      'label',
      'kind',
      'visibility',
      'extensionPolicy',
      'references',
      'packageId',
      'packageVersion',
      'sourceModule',
      'sourceDeclaration',
    ],
    path,
  );
  return {
    id: requiredString(record['id'], `${path}.id`),
    fingerprint: requiredString(record['fingerprint'], `${path}.fingerprint`),
    label: nullableString(record['label'], `${path}.label`),
    kind: requiredString(record['kind'], `${path}.kind`),
    visibility: requiredString(record['visibility'], `${path}.visibility`),
    extensionPolicy: requiredString(
      record['extensionPolicy'],
      `${path}.extensionPolicy`,
    ),
    references: stringArray(record['references'], `${path}.references`),
    packageId: requiredString(record['packageId'], `${path}.packageId`),
    packageVersion: requiredString(
      record['packageVersion'],
      `${path}.packageVersion`,
    ),
    sourceModule: requiredString(
      record['sourceModule'],
      `${path}.sourceModule`,
    ),
    sourceDeclaration: requiredString(
      record['sourceDeclaration'],
      `${path}.sourceDeclaration`,
    ),
  };
}

function relationship(value: unknown, path: string): ContentRelationshipDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'source', 'target', 'order'], path);
  return {
    kind: requiredString(record['kind'], `${path}.kind`),
    source: requiredString(record['source'], `${path}.source`),
    target: requiredString(record['target'], `${path}.target`),
    order: nonNegativeInteger(record['order'], `${path}.order`),
  };
}

function fingerprints(value: unknown, path: string): PlayBundleFingerprintDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['source', 'semantic', 'presentation'], path);
  return {
    source: requiredString(record['source'], `${path}.source`),
    semantic: requiredString(record['semantic'], `${path}.semantic`),
    presentation: requiredString(
      record['presentation'],
      `${path}.presentation`,
    ),
  };
}

function lifecycleStatus(
  value: unknown,
  path: string,
): PlayBundleLifecycleStatus {
  if (
    value === 'noActivePlayBundle' ||
    value === 'compiledCandidate' ||
    value === 'active'
  ) {
    return value;
  }
  throw new PlayProtocolDecodeError(path, 'unknown lifecycle status');
}

function requiredRecord(
  value: unknown,
  path: string,
): Readonly<Record<string, unknown>> {
  if (!isUnknownRecord(value)) {
    throw new PlayProtocolDecodeError(path, 'expected an object');
  }
  return value;
}

function isUnknownRecord(
  value: unknown,
): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function requiredArray(value: unknown, path: string): readonly unknown[] {
  if (!Array.isArray(value)) {
    throw new PlayProtocolDecodeError(path, 'expected an array');
  }
  return value;
}

function stringArray(value: unknown, path: string): string[] {
  return requiredArray(value, path).map((entry, index) =>
    requiredString(entry, `${path}[${index}]`),
  );
}

function nullableStringArray(value: unknown, path: string): string[] | null {
  return value === null ? null : stringArray(value, path);
}

function requiredString(value: unknown, path: string): string {
  if (typeof value !== 'string') {
    throw new PlayProtocolDecodeError(path, 'expected a string');
  }
  return value;
}

function nonNegativeIntegerString(value: unknown, path: string): string {
  const source = requiredString(value, path);
  if (!/^(?:0|[1-9][0-9]*)$/.test(source)) {
    throw new PlayProtocolDecodeError(
      path,
      'expected a canonical non-negative integer string',
    );
  }
  return source;
}

function nullableString(value: unknown, path: string): string | null {
  return value === null ? null : requiredString(value, path);
}

function requiredBoolean(value: unknown, path: string): boolean {
  if (typeof value !== 'boolean') {
    throw new PlayProtocolDecodeError(path, 'expected a boolean');
  }
  return value;
}

function nonNegativeInteger(value: unknown, path: string): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0) {
    throw new PlayProtocolDecodeError(path, 'expected a non-negative integer');
  }
  return value;
}

function requiredInteger(value: unknown, path: string): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value)) {
    throw new PlayProtocolDecodeError(path, 'expected an integer');
  }
  return value;
}

function nullableInteger(value: unknown, path: string): number | null {
  return value === null ? null : requiredInteger(value, path);
}

function exactKeys(
  record: Readonly<Record<string, unknown>>,
  keys: readonly string[],
  path: string,
) {
  const expected = new Set(keys);
  for (const key of Object.keys(record)) {
    if (!expected.has(key)) {
      throw new PlayProtocolDecodeError(`${path}.${key}`, 'unknown field');
    }
  }
  for (const key of keys) {
    if (!(key in record)) {
      throw new PlayProtocolDecodeError(`${path}.${key}`, 'missing field');
    }
  }
}
