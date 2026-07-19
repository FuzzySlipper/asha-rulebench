import type {
  RulesetCompositionManifest,
  RulesetPackageSource,
} from '@asha-rpg/authoring';

/**
 * The only value a selected TypeScript entrypoint may expose to Rulebench.
 * It is immutable authoring data; preparation and runtime authority stay in
 * the trusted compiler host and Rust respectively.
 */
export interface RulesetWorkspaceDeclaration {
  readonly composition: RulesetCompositionManifest;
  readonly packages: readonly RulesetPackageSource[];
}

export function isRulesetWorkspaceDeclaration(
  value: unknown,
): value is RulesetWorkspaceDeclaration {
  if (!isRecord(value)) return false;
  if (
    Object.keys(value).length !== 2 ||
    !('composition' in value) ||
    !('packages' in value) ||
    !Object.isFrozen(value)
  ) {
    return false;
  }
  if (!isRecord(value['composition'])) return false;
  if (!Object.isFrozen(value['composition'])) return false;
  if (!Array.isArray(value['packages'])) return false;
  if (!Object.isFrozen(value['packages'])) return false;
  return value['packages'].every((source) => {
    if (!isRecord(source)) return false;
    return (
      Object.isFrozen(source) &&
      isRecord(source['manifest']) &&
      Object.isFrozen(source['manifest']) &&
      typeof source['sourceFingerprint'] === 'string'
    );
  });
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
