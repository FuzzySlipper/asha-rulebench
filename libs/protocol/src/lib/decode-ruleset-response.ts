import type {
  RulesetArtifactSummaryDto,
  RulesetDefinitionDto,
  RulesetDiagnosticDto,
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
    diagnostics: requiredArray(record['diagnostics'], '$.diagnostics').map(
      (entry, index) => diagnostic(entry, `$.diagnostics[${index}]`),
    ),
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
  exactKeys(record, ['stage', 'severity', 'code', 'path', 'message'], path);
  return {
    stage: requiredString(record['stage'], `${path}.stage`),
    severity: requiredString(record['severity'], `${path}.severity`),
    code: requiredString(record['code'], `${path}.code`),
    path: requiredString(record['path'], `${path}.path`),
    message: requiredString(record['message'], `${path}.message`),
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
