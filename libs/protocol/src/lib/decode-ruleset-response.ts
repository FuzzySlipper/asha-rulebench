import type {
  GameplayActionDto,
  GameplayCostDto,
  GameplayEntityDto,
  GameplayEventDto,
  GameplayModifierDto,
  GameplayNamedValueDto,
  GameplayPreflightDto,
  GameplayRandomPlanConditionDto,
  GameplayRandomPlanConditionKindDto,
  GameplayRandomPlanEntryDto,
  GameplayRandomRequestDto,
  GameplayReactionDto,
  GameplayReactionOptionDto,
  GameplayResultDto,
  GameplaySessionDto,
  GameplayTraceDto,
  RulesetArtifactSummaryDto,
  RulesetDefinitionDto,
  RulesetDiagnosticDto,
  RulesetDiagnosticSourceDto,
  RulesetFingerprintDto,
  RulesetIdentityDto,
  RulesetLifecycleStatus,
  RulesetLockEntryDto,
  RulesetRelationshipDto,
  RulesetRequirementDto,
  RulesetSourcePackageDto,
  RulesetWorkspaceResponseDto,
} from '../generated/ruleset-protocol.js';

export class RulesetProtocolDecodeError extends Error {
  public constructor(path: string, message: string) {
    super(`${path}: ${message}`);
    this.name = 'RulesetProtocolDecodeError';
  }
}

export function decodeRulesetWorkspaceResponse(
  value: unknown,
): RulesetWorkspaceResponseDto {
  const record = requiredRecord(value, '$');
  exactKeys(
    record,
    [
      'ok',
      'status',
      'activeArtifact',
      'candidateArtifact',
      'activationRevision',
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
    activationRevision: nonNegativeInteger(
      record['activationRevision'],
      '$.activationRevision',
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

function nullableGameplay(
  value: unknown,
  path: string,
): GameplaySessionDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'actorId',
      'stateRevision',
      'acceptedRandomValues',
      'actions',
      'preflights',
      'entities',
      'pendingReaction',
      'lastResult',
    ],
    path,
  );
  return {
    actorId: requiredString(record['actorId'], `${path}.actorId`),
    stateRevision: nonNegativeInteger(
      record['stateRevision'],
      `${path}.stateRevision`,
    ),
    acceptedRandomValues: nonNegativeInteger(
      record['acceptedRandomValues'],
      `${path}.acceptedRandomValues`,
    ),
    actions: requiredArray(record['actions'], `${path}.actions`).map(
      (entry, index) => gameplayAction(entry, `${path}.actions[${index}]`),
    ),
    preflights: requiredArray(record['preflights'], `${path}.preflights`).map(
      (entry, index) =>
        gameplayPreflight(entry, `${path}.preflights[${index}]`),
    ),
    entities: requiredArray(record['entities'], `${path}.entities`).map(
      (entry, index) => gameplayEntity(entry, `${path}.entities[${index}]`),
    ),
    pendingReaction: nullableReaction(
      record['pendingReaction'],
      `${path}.pendingReaction`,
    ),
    lastResult: nullableResult(record['lastResult'], `${path}.lastResult`),
  };
}

function gameplayAction(value: unknown, path: string): GameplayActionDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'id',
      'name',
      'sourcePath',
      'team',
      'maximumRange',
      'maximumTargets',
      'costs',
      'randomPlan',
      'candidateIds',
    ],
    path,
  );
  return {
    id: requiredString(record['id'], `${path}.id`),
    name: requiredString(record['name'], `${path}.name`),
    sourcePath: requiredString(record['sourcePath'], `${path}.sourcePath`),
    team: requiredString(record['team'], `${path}.team`),
    maximumRange: nonNegativeInteger(
      record['maximumRange'],
      `${path}.maximumRange`,
    ),
    maximumTargets: nonNegativeInteger(
      record['maximumTargets'],
      `${path}.maximumTargets`,
    ),
    costs: requiredArray(record['costs'], `${path}.costs`).map((entry, index) =>
      gameplayCost(entry, `${path}.costs[${index}]`),
    ),
    randomPlan: requiredArray(
      record['randomPlan'],
      `${path}.randomPlan`,
    ).map((entry, index) =>
      gameplayRandomPlanEntry(entry, `${path}.randomPlan[${index}]`),
    ),
    candidateIds: stringArray(record['candidateIds'], `${path}.candidateIds`),
  };
}

function gameplayCost(value: unknown, path: string): GameplayCostDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['resourceId', 'amount'], path);
  return {
    resourceId: requiredString(record['resourceId'], `${path}.resourceId`),
    amount: requiredInteger(record['amount'], `${path}.amount`),
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

function gameplayRandomPlanEntry(
  value: unknown,
  path: string,
): GameplayRandomPlanEntryDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['request', 'conditions'], path);
  return {
    request: gameplayRandomRequest(record['request'], `${path}.request`),
    conditions: requiredArray(record['conditions'], `${path}.conditions`).map(
      (condition, index) =>
        gameplayRandomPlanCondition(
          condition,
          `${path}.conditions[${index}]`,
        ),
    ),
  };
}

function gameplayRandomPlanCondition(
  value: unknown,
  path: string,
): GameplayRandomPlanConditionDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'path'], path);
  return {
    kind: randomPlanConditionKind(record['kind'], `${path}.kind`),
    path: requiredString(record['path'], `${path}.path`),
  };
}

function randomPlanConditionKind(
  value: unknown,
  path: string,
): GameplayRandomPlanConditionKindDto {
  const kind = requiredString(value, path);
  switch (kind) {
    case 'whenThen':
    case 'whenOtherwise':
    case 'checkHit':
    case 'checkMiss':
    case 'checkSaved':
    case 'checkFailed':
    case 'checkNoRoll':
    case 'allPreviousTrue':
    case 'anyPreviousFalse':
      return kind;
    default:
      throw new RulesetProtocolDecodeError(path, `unknown random branch ${kind}`);
  }
}

function gameplayPreflight(value: unknown, path: string): GameplayPreflightDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    ['actionId', 'targetId', 'available', 'code', 'message'],
    path,
  );
  return {
    actionId: requiredString(record['actionId'], `${path}.actionId`),
    targetId: requiredString(record['targetId'], `${path}.targetId`),
    available: requiredBoolean(record['available'], `${path}.available`),
    code: nullableString(record['code'], `${path}.code`),
    message: requiredString(record['message'], `${path}.message`),
  };
}

function gameplayEntity(value: unknown, path: string): GameplayEntityDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'id',
      'team',
      'x',
      'y',
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
    team: requiredString(record['team'], `${path}.team`),
    x: nonNegativeInteger(record['x'], `${path}.x`),
    y: nonNegativeInteger(record['y'], `${path}.y`),
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
    randomConsumed: nonNegativeInteger(
      record['randomConsumed'],
      `${path}.randomConsumed`,
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

function nullableArtifact(
  value: unknown,
  path: string,
): RulesetArtifactSummaryDto | null {
  return value === null ? null : artifact(value, path);
}

function artifact(value: unknown, path: string): RulesetArtifactSummaryDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'schema',
      'artifactId',
      'composition',
      'language',
      'sourcePackages',
      'dependencyLock',
      'requiredOperations',
      'requiredCapabilities',
      'exportedRoots',
      'definitions',
      'policyBindingIds',
      'relationships',
      'derivationSlots',
      'overlaySlots',
      'fingerprints',
    ],
    path,
  );
  return {
    schema: identity(record['schema'], `${path}.schema`),
    artifactId: requiredString(record['artifactId'], `${path}.artifactId`),
    composition: identity(record['composition'], `${path}.composition`),
    language: identity(record['language'], `${path}.language`),
    sourcePackages: requiredArray(
      record['sourcePackages'],
      `${path}.sourcePackages`,
    ).map((entry, index) =>
      sourcePackage(entry, `${path}.sourcePackages[${index}]`),
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
    fingerprints: fingerprints(record['fingerprints'], `${path}.fingerprints`),
  };
}

function diagnostic(value: unknown, path: string): RulesetDiagnosticDto {
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
): RulesetDiagnosticSourceDto | null {
  if (value === null) return null;
  const record = requiredRecord(value, path);
  exactKeys(record, ['module', 'declaration'], path);
  return {
    module: requiredString(record['module'], `${path}.module`),
    declaration: requiredString(record['declaration'], `${path}.declaration`),
  };
}

function identity(value: unknown, path: string): RulesetIdentityDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: requiredString(record['version'], `${path}.version`),
  };
}

function sourcePackage(value: unknown, path: string): RulesetSourcePackageDto {
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

function lockEntry(value: unknown, path: string): RulesetLockEntryDto {
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

function requirement(value: unknown, path: string): RulesetRequirementDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['id', 'version'], path);
  return {
    id: requiredString(record['id'], `${path}.id`),
    version: nonNegativeInteger(record['version'], `${path}.version`),
  };
}

function definition(value: unknown, path: string): RulesetDefinitionDto {
  const record = requiredRecord(value, path);
  exactKeys(
    record,
    [
      'id',
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

function relationship(value: unknown, path: string): RulesetRelationshipDto {
  const record = requiredRecord(value, path);
  exactKeys(record, ['kind', 'source', 'target', 'order'], path);
  return {
    kind: requiredString(record['kind'], `${path}.kind`),
    source: requiredString(record['source'], `${path}.source`),
    target: requiredString(record['target'], `${path}.target`),
    order: nonNegativeInteger(record['order'], `${path}.order`),
  };
}

function fingerprints(value: unknown, path: string): RulesetFingerprintDto {
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

function lifecycleStatus(value: unknown, path: string): RulesetLifecycleStatus {
  if (
    value === 'noActiveRuleset' ||
    value === 'compiledCandidate' ||
    value === 'active'
  ) {
    return value;
  }
  throw new RulesetProtocolDecodeError(path, 'unknown lifecycle status');
}

function requiredRecord(
  value: unknown,
  path: string,
): Readonly<Record<string, unknown>> {
  if (!isUnknownRecord(value)) {
    throw new RulesetProtocolDecodeError(path, 'expected an object');
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
    throw new RulesetProtocolDecodeError(path, 'expected an array');
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
    throw new RulesetProtocolDecodeError(path, 'expected a string');
  }
  return value;
}

function nullableString(value: unknown, path: string): string | null {
  return value === null ? null : requiredString(value, path);
}

function requiredBoolean(value: unknown, path: string): boolean {
  if (typeof value !== 'boolean') {
    throw new RulesetProtocolDecodeError(path, 'expected a boolean');
  }
  return value;
}

function nonNegativeInteger(value: unknown, path: string): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0) {
    throw new RulesetProtocolDecodeError(
      path,
      'expected a non-negative integer',
    );
  }
  return value;
}

function requiredInteger(value: unknown, path: string): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value)) {
    throw new RulesetProtocolDecodeError(path, 'expected an integer');
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
      throw new RulesetProtocolDecodeError(`${path}.${key}`, 'unknown field');
    }
  }
  for (const key of keys) {
    if (!(key in record)) {
      throw new RulesetProtocolDecodeError(`${path}.${key}`, 'missing field');
    }
  }
}
