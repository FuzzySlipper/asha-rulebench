import { Component, input } from '@angular/core';
import { StatusLineComponent } from '@asha-rulebench/components';
import type { RulebenchStatusView } from '@asha-rulebench/domain';

@Component({
  imports: [StatusLineComponent],
  selector: 'arb-status-renderer',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
    `,
  ],
  template: `<arb-status-line [label]="status().label" />`,
})
export class StatusRendererComponent {
  readonly status = input.required<RulebenchStatusView>();
}
