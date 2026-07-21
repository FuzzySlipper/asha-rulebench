import {
  decodePlayWorkspaceResponse,
  decodeRulesetCatalogResponse,
  decodePlayBundleSourceSetConfig,
  type GameplayCommandRequestDto,
  type GameplayReactionRequestDto,
  type GameplayTurnControlRequestDto,
  type PlayBundleCompileRequestDto,
  type PlayWorkspaceResponseDto,
  type RulesetCatalogRequestDto,
  type RulesetCatalogResponseDto,
  type PlayBundleSourceSetConfigDto,
  type ScenarioSetupRequestDto,
} from '@asha-rulebench/protocol';

export interface JsonHttpClient {
  readonly request: (
    method: 'GET' | 'POST',
    path: string,
    body?: unknown,
  ) => Promise<unknown>;
}

export interface PlayTransport {
  readonly sourceSets: () => Promise<PlayBundleSourceSetConfigDto>;
  readonly inspectRuleset: (
    request: RulesetCatalogRequestDto,
  ) => Promise<RulesetCatalogResponseDto>;
  readonly status: () => Promise<PlayWorkspaceResponseDto>;
  readonly compile: (
    request: PlayBundleCompileRequestDto,
  ) => Promise<PlayWorkspaceResponseDto>;
  readonly activatePlayBundle: () => Promise<PlayWorkspaceResponseDto>;
  readonly startScenario: (
    setup: ScenarioSetupRequestDto,
  ) => Promise<PlayWorkspaceResponseDto>;
  readonly command: (
    command: GameplayCommandRequestDto,
  ) => Promise<PlayWorkspaceResponseDto>;
  readonly react: (
    reaction: GameplayReactionRequestDto,
  ) => Promise<PlayWorkspaceResponseDto>;
  readonly control: (
    control: GameplayTurnControlRequestDto,
  ) => Promise<PlayWorkspaceResponseDto>;
  readonly restoreCheckpoint: () => Promise<PlayWorkspaceResponseDto>;
  readonly replay: () => Promise<PlayWorkspaceResponseDto>;
}

export function createPlayTransport(http: JsonHttpClient): PlayTransport {
  const request = async (
    method: 'GET' | 'POST',
    path: string,
    body?: unknown,
  ): Promise<PlayWorkspaceResponseDto> => {
    const payload = await http.request(method, path, body);
    return decodePlayWorkspaceResponse(payload);
  };

  return {
    sourceSets: async () => {
      const payload = await http.request('GET', '/api/play-bundle/source-sets');
      return decodePlayBundleSourceSetConfig(payload);
    },
    inspectRuleset: async (catalogRequest) => {
      const payload = await http.request(
        'POST',
        '/api/rulesets/inspect',
        catalogRequest,
      );
      return decodeRulesetCatalogResponse(payload);
    },
    status: () => request('GET', '/api/play'),
    compile: (compileRequest) =>
      request('POST', '/api/play-bundle/compile', compileRequest),
    activatePlayBundle: () => request('POST', '/api/play-bundle/activate'),
    startScenario: (setup) => request('POST', '/api/scenario/start', setup),
    command: (command) => request('POST', '/api/session/command', command),
    react: (reaction) => request('POST', '/api/session/reaction', reaction),
    control: (control) => request('POST', '/api/session/control', control),
    restoreCheckpoint: () => request('POST', '/api/session/checkpoint/restore'),
    replay: () => request('POST', '/api/session/replay'),
  };
}
