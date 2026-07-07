import type { TemplateStatusDto } from '@asha-rulebench/protocol';

export function makeRulebenchStatus(overrides: Partial<TemplateStatusDto> = {}): TemplateStatusDto {
  return {
    status: 'idle',
    label: 'Session idle',
    ...overrides,
  };
}
