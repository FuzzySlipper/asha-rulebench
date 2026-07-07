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
