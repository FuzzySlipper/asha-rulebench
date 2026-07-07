import type { Result, TemplateStatusDto } from '@asha-rulebench/protocol';

export interface RulebenchTransport {
  readonly loadStatus: () => Promise<Result<TemplateStatusDto>>;
}

export const createFakeRulebenchTransport = (status: TemplateStatusDto = { status: 'idle', label: 'Session idle' }): RulebenchTransport => ({
  loadStatus: async () => ({ ok: true, value: status }),
});
