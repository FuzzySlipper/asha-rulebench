import { Component, computed, inject } from '@angular/core';
import type { OnInit } from '@angular/core';
import { RulebenchScenarioRendererComponent } from '@asha-rulebench/renderer';
import { SessionStore } from '@asha-rulebench/store';
import type { RulebenchScenarioOutcomeClassDto } from '@asha-rulebench/protocol';

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

      .viewer {
        display: grid;
        gap: 16px;
      }

      .catalog {
        border-bottom: 1px solid var(--arb-border);
        display: grid;
        gap: 10px;
        padding: 16px 44px 0;
      }

      .catalog h2 {
        font-size: 0.95rem;
        margin: 0;
      }

      .catalog-row {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .scenario-button {
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        color: var(--arb-text);
        cursor: pointer;
        display: grid;
        gap: 3px;
        min-width: 190px;
        padding: 10px 12px;
        text-align: left;
      }

      .scenario-button[aria-pressed='true'] {
        border-color: var(--arb-accent);
        box-shadow: inset 3px 0 0 var(--arb-accent);
      }

      .scenario-title {
        font-weight: 700;
      }

      .scenario-meta,
      .catalog-status {
        color: var(--arb-muted);
        font-size: 0.85rem;
      }

      h1,
      h2,
      p {
        margin: 0;
      }
    `,
  ],
  template: `
    <div class="viewer">
      <section class="catalog" aria-label="Scenario catalog">
        <h2>Scenario Cases</h2>
        @switch (catalog().kind) {
          @case ('data') {
            <div class="catalog-row">
              @for (summary of catalog().value; track summary.id) {
                <button
                  class="scenario-button"
                  type="button"
                  [attr.aria-pressed]="selectedScenarioId() === summary.id"
                  (click)="selectScenario(summary.id)"
                >
                  <span class="scenario-title">{{ summary.title }}</span>
                  <span class="scenario-meta">{{ outcomeClassLabel(summary.outcomeClass) }} · {{ summary.seedLabel }}</span>
                </button>
              }
            </div>
          }
          @case ('loading') {
            <p class="catalog-status">Loading scenario catalog</p>
          }
          @case ('error') {
            <p class="catalog-status">{{ catalog().error.message }}</p>
          }
          @case ('idle') {
            <p class="catalog-status">Scenario catalog idle</p>
          }
        }
      </section>

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
    </div>
  `,
})
export class ScenarioViewerFeatureComponent implements OnInit {
  private readonly sessionStore = inject(SessionStore);
  protected readonly catalog = computed(() => this.sessionStore.catalog());
  protected readonly selectedScenarioId = computed(() => this.sessionStore.selectedScenarioId());
  protected readonly scenario = computed(() => this.sessionStore.scenario());

  ngOnInit(): void {
    void this.loadInitialScenario();
  }

  protected selectScenario(scenarioId: string): void {
    void this.sessionStore.selectScenario(scenarioId);
  }

  protected outcomeClassLabel(outcomeClass: RulebenchScenarioOutcomeClassDto): string {
    switch (outcomeClass) {
      case 'acceptedHit':
        return 'Accepted hit';
      case 'acceptedMiss':
        return 'Accepted miss';
      case 'rejectedTargetLegality':
        return 'Rejected target';
    }
  }

  private async loadInitialScenario(): Promise<void> {
    await this.sessionStore.loadCatalog();
    await this.sessionStore.loadScenario();
  }
}
