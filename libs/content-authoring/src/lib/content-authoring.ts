import type {
  ContentPackSource,
  PlayBundleManifest,
  Ruleset,
} from '@asha-rpg/authoring';

/**
 * Rulebench discovers immutable public authoring values from a canonical
 * Ruleset root. It does not accept an aggregate wrapper that could blur the
 * Ruleset, Content Pack, and PlayBundle boundaries.
 */
export function isRuleset(value: unknown): value is Ruleset {
  if (!isFrozenRecord(value)) return false;
  const schema = value['schema'];
  const identity = value['identity'];
  const language = value['language'];
  const models = value['models'];
  const provides = value['provides'];
  return (
    isFrozenRecord(schema) &&
    schema['identity'] === 'asha.rpg.ruleset' &&
    schema['major'] === 1 &&
    isVersionedIdentity(identity) &&
    isVersionedIdentity(language) &&
    isFrozenRecord(models) &&
    isFrozenRecord(provides)
  );
}

export function isContentPackSource(
  value: unknown,
): value is ContentPackSource {
  if (!isFrozenRecord(value)) return false;
  const manifest = value['manifest'];
  return (
    typeof value['sourceFingerprint'] === 'string' &&
    isFrozenRecord(manifest) &&
    isVersionedIdentity(manifest['identity']) &&
    Array.isArray(manifest['definitions']) &&
    Array.isArray(manifest['exports'])
  );
}

export function isPlayBundleManifest(
  value: unknown,
): value is PlayBundleManifest {
  if (!isFrozenRecord(value)) return false;
  return (
    isVersionedIdentity(value['identity']) &&
    isRuleset(value['ruleset']) &&
    isFrozenRecord(value['base']) &&
    Array.isArray(value['add']) &&
    Array.isArray(value['overlays']) &&
    isFrozenRecord(value['configure'])
  );
}

function isVersionedIdentity(value: unknown): boolean {
  return (
    isFrozenRecord(value) &&
    typeof value['id'] === 'string' &&
    value['id'].length > 0 &&
    typeof value['version'] === 'string' &&
    value['version'].length > 0
  );
}

function isFrozenRecord(
  value: unknown,
): value is Readonly<Record<string, unknown>> {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    Object.isFrozen(value)
  );
}
