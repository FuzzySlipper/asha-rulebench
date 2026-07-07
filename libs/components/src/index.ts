import { Component, input } from '@angular/core';

@Component({
  selector: 'tpl-status-line',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
        color: var(--tpl-muted);
      }
    `,
  ],
  template: `<p>{{ label() }}</p>`,
})
export class StatusLineComponent {
  readonly label = input.required<string>();
}
