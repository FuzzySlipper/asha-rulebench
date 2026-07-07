import type { TemplateStatusDto } from '@template/protocol';

export interface TemplateStatusView {
  readonly label: string;
  readonly isReady: boolean;
}

export function projectTemplateStatus(status: TemplateStatusDto): TemplateStatusView {
  return {
    label: status.label,
    isReady: status.status === 'ready',
  };
}
