import { Component, computed, inject } from '@angular/core';
import type { OnInit } from '@angular/core';
import { RulebenchScenarioRendererComponent } from '@asha-rulebench/renderer';
import { SessionStore } from '@asha-rulebench/store';

@Component({
  imports: [RulebenchScenarioRendererComponent],
  selector: 'arb-scenario-viewer-feature',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
        min-height: 100vh;
      }

      .state {
        align-items: center;
        display: grid;
        min-height: 100vh;
        padding: 24px;
      }

      .state-inner {
        border-left: 4px solid var(--arb-accent);
        display: grid;
        gap: 8px;
        max-width: 720px;
        padding-left: 16px;
      }

      h1,
      p {
        margin: 0;
      }
    `,
  ],
  template: `
    @switch (scenario().kind) {
      @case ('idle') {
        <section class="state" aria-label="Scenario status">
          <div class="state-inner">
            <h1>ASHA Rulebench</h1>
            <p>Scenario idle</p>
          </div>
        </section>
      }
      @case ('loading') {
        <section class="state" aria-label="Scenario status">
          <div class="state-inner">
            <h1>ASHA Rulebench</h1>
            <p>Loading scenario</p>
          </div>
        </section>
      }
      @case ('data') {
        <arb-rulebench-scenario-renderer [scenario]="scenario().value" />
      }
      @case ('error') {
        <section class="state" aria-label="Scenario status">
          <div class="state-inner">
            <h1>ASHA Rulebench</h1>
            <p>{{ scenario().error.message }}</p>
          </div>
        </section>
      }
    }
  `,
})
export class ScenarioViewerFeatureComponent implements OnInit {
  private readonly sessionStore = inject(SessionStore);
  protected readonly scenario = computed(() => this.sessionStore.scenario());

  ngOnInit(): void {
    void this.sessionStore.loadScenario();
  }
}
