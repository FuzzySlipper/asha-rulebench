import { Component, input } from '@angular/core';
import { StatusLineComponent } from '@asha-rulebench/components';
import type { RulebenchScenarioView } from '@asha-rulebench/domain';

@Component({
  imports: [StatusLineComponent],
  selector: 'arb-scenario-summary-renderer',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
    `,
  ],
  template: `<arb-status-line [label]="scenario().title" />`,
})
export class ScenarioSummaryRendererComponent {
  readonly scenario = input.required<RulebenchScenarioView>();
}
