import {
  decodeRulesetWorkspaceResponse,
  type RulesetCompileRequestDto,
  type RulesetSourceIdDto,
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
  readonly compile: (
    sourceId: RulesetSourceIdDto,
  ) => Promise<RulesetWorkspaceResponseDto>;
  readonly activate: () => Promise<RulesetWorkspaceResponseDto>;
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
  };
}
