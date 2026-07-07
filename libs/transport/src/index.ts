import type { Result, TemplateStatusDto } from '@template/protocol';

export interface TemplateTransport {
  readonly loadStatus: () => Promise<Result<TemplateStatusDto>>;
}

export const createFakeTemplateTransport = (status: TemplateStatusDto = { status: 'idle', label: 'Session idle' }): TemplateTransport => ({
  loadStatus: async () => ({ ok: true, value: status }),
});
