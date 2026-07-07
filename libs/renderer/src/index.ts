import { Component, input } from '@angular/core';
import { StatusLineComponent } from '@template/components';
import type { TemplateStatusView } from '@template/domain';

@Component({
  imports: [StatusLineComponent],
  selector: 'tpl-status-renderer',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
    `,
  ],
  template: `<tpl-status-line [label]="status().label" />`,
})
export class StatusRendererComponent {
  readonly status = input.required<TemplateStatusView>();
}
