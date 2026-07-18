import {
  decodeRulesetWorkspaceResponse,
  type RulesetWorkspaceResponseDto,
} from '@asha-rulebench/protocol';

export interface JsonHttpClient {
  readonly request: (method: 'GET' | 'POST', path: string) => Promise<unknown>;
}

export interface RulesetTransport {
  readonly status: () => Promise<RulesetWorkspaceResponseDto>;
  readonly compile: () => Promise<RulesetWorkspaceResponseDto>;
  readonly activate: () => Promise<RulesetWorkspaceResponseDto>;
}

export function createRulesetTransport(http: JsonHttpClient): RulesetTransport {
  const request = async (
    method: 'GET' | 'POST',
    path: string,
  ): Promise<RulesetWorkspaceResponseDto> => {
    const payload = await http.request(method, path);
    return decodeRulesetWorkspaceResponse(payload);
  };

  return {
    status: () => request('GET', '/api/ruleset'),
    compile: () => request('POST', '/api/ruleset/compile'),
    activate: () => request('POST', '/api/ruleset/activate'),
  };
}
