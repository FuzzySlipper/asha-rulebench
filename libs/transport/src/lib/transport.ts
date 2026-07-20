import {
  decodeRulesetWorkspaceResponse,
  type EncounterSetupRequestDto,
  type GameplayCommandRequestDto,
  type GameplayReactionRequestDto,
  type GameplayTurnControlRequestDto,
  type RulesetCompileRequestDto,
  type RulesetWorkspaceResponseDto,
} from '@asha-rulebench/protocol';

export interface JsonHttpClient {
  readonly request: (
    method: 'GET' | 'POST',
    path: string,
    body?: unknown,
  ) => Promise<unknown>;
}

export interface ConfiguredRulesetLocation {
  readonly id: string;
  readonly label: string;
  readonly rulesetRoot: string;
}

export interface RulesetTransport {
  readonly configuredRulesets: () => Promise<
    readonly ConfiguredRulesetLocation[]
  >;
  readonly status: () => Promise<RulesetWorkspaceResponseDto>;
  readonly compile: (
    request: RulesetCompileRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly activate: () => Promise<RulesetWorkspaceResponseDto>;
  readonly startEncounter: (
    setup: EncounterSetupRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly command: (
    command: GameplayCommandRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly react: (
    reaction: GameplayReactionRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly control: (
    control: GameplayTurnControlRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly restoreCheckpoint: () => Promise<RulesetWorkspaceResponseDto>;
  readonly replay: () => Promise<RulesetWorkspaceResponseDto>;
}

export function createRulesetTransport(http: JsonHttpClient): RulesetTransport {
  const request = async (
    method: 'GET' | 'POST',
    path: string,
    body?: unknown,
  ): Promise<RulesetWorkspaceResponseDto> => {
    const payload = await http.request(method, path, body);
    return decodeRulesetWorkspaceResponse(payload);
  };

  return {
    configuredRulesets: async () => {
      const payload = await http.request('GET', '/api/ruleset/config');
      return decodeConfiguredRulesets(payload);
    },
    status: () => request('GET', '/api/ruleset'),
    compile: (compileRequest) =>
      request('POST', '/api/ruleset/compile', compileRequest),
    activate: () => request('POST', '/api/ruleset/activate'),
    startEncounter: (setup) => request('POST', '/api/ruleset/encounter', setup),
    command: (command) => request('POST', '/api/ruleset/command', command),
    react: (reaction) => request('POST', '/api/ruleset/reaction', reaction),
    control: (control) => request('POST', '/api/ruleset/control', control),
    restoreCheckpoint: () => request('POST', '/api/ruleset/checkpoint/restore'),
    replay: () => request('POST', '/api/ruleset/replay'),
  };
}

function decodeConfiguredRulesets(
  value: unknown,
): readonly ConfiguredRulesetLocation[] {
  const record = requiredRecord(value, '$');
  exactKeys(record, ['schemaVersion', 'rulesets'], '$');
  if (record['schemaVersion'] !== 1) {
    throw new Error('$.schemaVersion: expected ruleset config version 1');
  }
  const rulesets = record['rulesets'];
  if (!Array.isArray(rulesets)) {
    throw new Error('$.rulesets: expected an array');
  }
  const ids = new Set<string>();
  const roots = new Set<string>();
  return rulesets.map((entry, index) => {
    const path = `$.rulesets[${index}]`;
    const location = requiredRecord(entry, path);
    exactKeys(location, ['id', 'label', 'rulesetRoot'], path);
    const id = requiredString(location['id'], `${path}.id`);
    const label = requiredString(location['label'], `${path}.label`);
    const rulesetRoot = requiredString(
      location['rulesetRoot'],
      `${path}.rulesetRoot`,
    );
    if (ids.has(id)) throw new Error(`${path}.id: duplicate ${id}`);
    if (roots.has(rulesetRoot)) {
      throw new Error(`${path}.rulesetRoot: duplicate ${rulesetRoot}`);
    }
    ids.add(id);
    roots.add(rulesetRoot);
    return { id, label, rulesetRoot };
  });
}

function requiredRecord(
  value: unknown,
  path: string,
): Readonly<Record<string, unknown>> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${path}: expected an object`);
  }
  return Object.fromEntries(Object.entries(value));
}

function requiredString(value: unknown, path: string): string {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${path}: expected a non-empty string`);
  }
  return value.trim();
}

function exactKeys(
  record: Readonly<Record<string, unknown>>,
  expectedKeys: readonly string[],
  path: string,
): void {
  const expected = new Set(expectedKeys);
  const unexpected = Object.keys(record).filter((key) => !expected.has(key));
  const missing = expectedKeys.filter(
    (key) => !Object.prototype.hasOwnProperty.call(record, key),
  );
  if (unexpected.length > 0 || missing.length > 0) {
    throw new Error(
      `${path}: invalid keys; missing ${missing.join(', ') || 'none'}; unexpected ${unexpected.join(', ') || 'none'}`,
    );
  }
}
