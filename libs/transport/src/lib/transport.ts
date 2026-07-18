import {
  decodeRulesetWorkspaceResponse,
  type GameplayCommandRequestDto,
  type GameplayReactionRequestDto,
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

export interface RulesetTransport {
  readonly status: () => Promise<RulesetWorkspaceResponseDto>;
  readonly compile: (sourceId: string) => Promise<RulesetWorkspaceResponseDto>;
  readonly activate: () => Promise<RulesetWorkspaceResponseDto>;
  readonly command: (
    command: GameplayCommandRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly react: (
    reaction: GameplayReactionRequestDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
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
    status: () => request('GET', '/api/ruleset'),
    compile: (sourceId) => {
      const compileRequest: RulesetCompileRequestDto = { sourceId };
      return request('POST', '/api/ruleset/compile', compileRequest);
    },
    activate: () => request('POST', '/api/ruleset/activate'),
    command: (command) => request('POST', '/api/ruleset/command', command),
    react: (reaction) => request('POST', '/api/ruleset/reaction', reaction),
  };
}
