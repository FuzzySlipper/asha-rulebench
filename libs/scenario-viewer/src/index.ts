import { Component, computed, inject, signal } from '@angular/core';
import type { OnInit } from '@angular/core';
import { RulebenchScenarioRendererComponent } from '@asha-rulebench/renderer';
import { SessionStore } from '@asha-rulebench/store';
import type { RulebenchCommandOutcomeClassDto, RulebenchScenarioOutcomeClassDto } from '@asha-rulebench/protocol';

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

      .session {
        border-bottom: 1px solid var(--arb-border);
        display: grid;
        gap: 12px;
        padding: 16px 44px 0;
      }

      .catalog h2 {
        font-size: 0.95rem;
        margin: 0;
      }

      .session h2,
      .session h3 {
        font-size: 0.95rem;
        margin: 0;
      }

      .catalog-row {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .session-row,
      .session-actions,
      .state-pair {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .scenario-button,
      .step-button,
      .control-button {
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        color: var(--arb-text);
        cursor: pointer;
      }

      .scenario-button,
      .step-button {
        display: grid;
        gap: 3px;
        min-width: 190px;
        padding: 10px 12px;
        text-align: left;
      }

      .control-button {
        min-height: 34px;
        padding: 7px 12px;
      }

      .scenario-button[aria-pressed='true'],
      .step-button[aria-pressed='true'] {
        border-color: var(--arb-accent);
        box-shadow: inset 3px 0 0 var(--arb-accent);
      }

      .scenario-title,
      .step-title {
        font-weight: 700;
      }

      .scenario-meta,
      .step-meta,
      .catalog-status,
      .session-status,
      .command-line,
      .log-event-types {
        color: var(--arb-muted);
        font-size: 0.85rem;
      }

      .session-detail {
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        display: grid;
        gap: 10px;
        padding: 12px;
      }

      .log-list,
      .state-list {
        display: grid;
        gap: 6px;
        list-style: none;
        margin: 0;
        padding: 0;
      }

      .log-list li,
      .state-card {
        border-left: 3px solid var(--arb-border);
        padding-left: 8px;
      }

      .state-card {
        display: grid;
        gap: 4px;
      }

      .state-card h4 {
        font-size: 0.85rem;
        margin: 0;
      }

      h1,
      h2,
      h3,
      h4,
      p {
        margin: 0;
      }
    `,
  ],
  template: `
    <div class="viewer">
      <section class="session" aria-label="Combat session">
        <h2>Combat Session</h2>
        @switch (sessionCatalog().kind) {
          @case ('data') {
            @for (session of sessionCatalog().value; track session.id) {
              <div class="session-row">
                @for (step of session.steps; track step.id) {
                  <button
                    class="step-button"
                    type="button"
                    [attr.aria-pressed]="selectedSessionId() === session.id && selectedSessionStepId() === step.id && viewerMode() === 'session'"
                    (click)="selectSessionStep(session.id, step.id)"
                  >
                    <span class="step-title">{{ step.logIndex }} · {{ step.title }}</span>
                    <span class="step-meta">{{ sessionOutcomeClassLabel(step.outcomeClass) }}</span>
                  </button>
                }
              </div>
              <div class="session-actions" aria-label="Combat session controls">
                <button class="control-button" type="button" (click)="previousSessionStep()">Previous</button>
                <button class="control-button" type="button" (click)="nextSessionStep()">Next</button>
              </div>
            }
          }
          @case ('loading') {
            <p class="session-status">Loading combat session</p>
          }
          @case ('error') {
            <p class="session-status">{{ sessionCatalog().error.message }}</p>
          }
          @case ('idle') {
            <p class="session-status">Combat session idle</p>
          }
        }

        @switch (sessionStep().kind) {
          @case ('data') {
            <section class="session-detail" aria-label="Combat session step">
              <h3>{{ sessionStep().value.step.indexLabel }} · {{ sessionStep().value.step.title }}</h3>
              <p class="command-line">
                {{ sessionStep().value.command.actorId }} · {{ sessionStep().value.command.actionId }} →
                {{ sessionStep().value.command.targetId }} · rolls {{ sessionStep().value.command.rollStreamLabel }} ·
                {{ sessionStep().value.command.outcomeLabel }}
              </p>
              <ul class="log-list" aria-label="Combat log">
                @for (entry of sessionStep().value.combatLog; track entry.id) {
                  <li>
                    <strong>{{ entry.logIndexLabel }} · {{ entry.title }}</strong>
                    <p>{{ entry.summary }}</p>
                    <p class="log-event-types">{{ entry.eventTypeLabels.join(', ') || 'No accepted DomainEvents' }}</p>
                  </li>
                }
              </ul>
              <div class="state-pair" aria-label="Step state review">
                <div class="state-card">
                  <h4>Before</h4>
                  <p>{{ sessionStep().value.stateBefore.summary }}</p>
                  <ul class="state-list">
                    @for (combatant of sessionStep().value.stateBefore.combatants; track combatant.id) {
                      <li>{{ combatant.name }} · {{ combatant.hitPointLabel }} · {{ combatant.conditionLabels.join(', ') }}</li>
                    }
                  </ul>
                </div>
                <div class="state-card">
                  <h4>After</h4>
                  <p>{{ sessionStep().value.stateAfter.summary }}</p>
                  <ul class="state-list">
                    @for (combatant of sessionStep().value.stateAfter.combatants; track combatant.id) {
                      <li>{{ combatant.name }} · {{ combatant.hitPointLabel }} · {{ combatant.conditionLabels.join(', ') }}</li>
                    }
                  </ul>
                </div>
              </div>
            </section>
          }
          @case ('loading') {
            <p class="session-status">Loading combat session step</p>
          }
          @case ('error') {
            <p class="session-status">{{ sessionStep().error.message }}</p>
          }
          @case ('idle') {
            <p class="session-status">Combat session step idle</p>
          }
        }
      </section>

      <section class="catalog" aria-label="Scenario catalog">
        <h2>Scenario Cases</h2>
        @switch (catalog().kind) {
          @case ('data') {
            <div class="catalog-row">
              @for (summary of catalog().value; track summary.id) {
                <button
                  class="scenario-button"
                  type="button"
                  [attr.aria-pressed]="viewerMode() === 'scenario' && selectedScenarioId() === summary.id"
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

      @switch (activeScenario().kind) {
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
          <arb-rulebench-scenario-renderer [scenario]="activeScenario().value" />
        }
        @case ('error') {
          <section class="state" aria-label="Scenario status">
            <div class="state-inner">
              <h1>ASHA Rulebench</h1>
              <p>{{ activeScenario().error.message }}</p>
            </div>
          </section>
        }
      }
    </div>
  `,
})
export class ScenarioViewerFeatureComponent implements OnInit {
  private readonly sessionStore = inject(SessionStore);
  protected readonly viewerMode = signal<'session' | 'scenario'>('session');
  protected readonly catalog = computed(() => this.sessionStore.catalog());
  protected readonly selectedScenarioId = computed(() => this.sessionStore.selectedScenarioId());
  protected readonly scenario = computed(() => this.sessionStore.scenario());
  protected readonly sessionCatalog = computed(() => this.sessionStore.sessionCatalog());
  protected readonly selectedSessionId = computed(() => this.sessionStore.selectedSessionId());
  protected readonly selectedSessionStepId = computed(() => this.sessionStore.selectedSessionStepId());
  protected readonly sessionStep = computed(() => this.sessionStore.sessionStep());
  protected readonly activeScenario = computed(() => {
    if (this.viewerMode() === 'scenario') {
      return this.scenario();
    }

    const step = this.sessionStep();
    if (step.kind === 'data') {
      return { kind: 'data' as const, value: step.value.scenario };
    }
    if (step.kind === 'error') {
      return { kind: 'error' as const, error: step.error };
    }
    return step.kind === 'loading' ? { kind: 'loading' as const } : { kind: 'idle' as const };
  });

  ngOnInit(): void {
    void this.loadInitialScenario();
  }

  protected selectScenario(scenarioId: string): void {
    this.viewerMode.set('scenario');
    void this.sessionStore.selectScenario(scenarioId);
  }

  protected selectSessionStep(sessionId: string, stepId: string): void {
    this.viewerMode.set('session');
    void this.sessionStore.selectSessionStep(sessionId, stepId);
  }

  protected nextSessionStep(): void {
    this.viewerMode.set('session');
    void this.sessionStore.nextSessionStep();
  }

  protected previousSessionStep(): void {
    this.viewerMode.set('session');
    void this.sessionStore.previousSessionStep();
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

  protected sessionOutcomeClassLabel(outcomeClass: RulebenchCommandOutcomeClassDto): string {
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
    await this.sessionStore.loadSessionCatalog();
    await this.sessionStore.loadSessionStep();
    await this.sessionStore.loadCatalog();
    await this.sessionStore.loadScenario();
  }
}
