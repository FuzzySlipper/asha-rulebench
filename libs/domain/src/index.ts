import type { TemplateStatusDto } from '@asha-rulebench/protocol';

export interface RulebenchStatusView {
  readonly label: string;
  readonly isReady: boolean;
}

export function projectRulebenchStatus(status: TemplateStatusDto): RulebenchStatusView {
  return {
    label: status.label,
    isReady: status.status === 'ready',
  };
}
