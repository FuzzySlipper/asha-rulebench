import type { TemplateStatusDto } from '@template/protocol';

export function makeTemplateStatus(overrides: Partial<TemplateStatusDto> = {}): TemplateStatusDto {
  return {
    status: 'idle',
    label: 'Session idle',
    ...overrides,
  };
}
