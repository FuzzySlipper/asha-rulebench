import { Component, input } from '@angular/core';

@Component({
  selector: 'arb-status-line',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
        color: var(--arb-muted);
      }
    `,
  ],
  template: `<p>{{ label() }}</p>`,
})
export class StatusLineComponent {
  readonly label = input.required<string>();
}

export * from './lib/workbench-panel/workbench-panel.component';
export * from './lib/application-menubar/application-menubar.component';
export * from './lib/application-dialog/application-dialog.component';
