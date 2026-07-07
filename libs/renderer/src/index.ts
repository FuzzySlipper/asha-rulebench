import { Component, input } from '@angular/core';
import type { RulebenchScenarioView } from '@asha-rulebench/domain';

@Component({
  selector: 'arb-rulebench-scenario-renderer',
  standalone: true,
  styles: [
    `
      :host {
        display: block;
        min-height: 100vh;
      }

      .workspace {
        display: grid;
        gap: 16px;
        margin: 0 auto;
        max-width: 1240px;
        padding: 24px;
      }

      header {
        border-bottom: 1px solid var(--arb-border);
        display: grid;
        gap: 6px;
        padding-bottom: 16px;
      }

      h1,
      h2,
      h3,
      p {
        margin: 0;
      }

      h1 {
        font-size: 2rem;
        line-height: 1.1;
      }

      h2 {
        font-size: 1rem;
        line-height: 1.2;
      }

      h3 {
        font-size: 0.9rem;
        line-height: 1.2;
      }

      .summary {
        color: var(--arb-muted);
      }

      .layout {
        align-items: start;
        display: grid;
        gap: 16px;
        grid-template-columns: minmax(320px, 1.15fr) minmax(280px, 0.85fr);
      }

      .panel {
        border: 1px solid var(--arb-border);
        border-radius: 8px;
        display: grid;
        gap: 12px;
        padding: 14px;
      }

      .board {
        display: grid;
        gap: 4px;
      }

      .cell {
        align-items: center;
        aspect-ratio: 1;
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        display: grid;
        font-size: 0.8rem;
        justify-items: center;
        min-width: 0;
        padding: 4px;
      }

      .cell-cover {
        background: #eef4f0;
      }

      .occupant {
        background: var(--arb-accent);
        border-radius: 999px;
        color: white;
        display: inline-grid;
        font-weight: 700;
        height: 28px;
        place-items: center;
        width: 28px;
      }

      .split {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      .list {
        display: grid;
        gap: 8px;
      }

      .row {
        border-left: 3px solid var(--arb-border);
        display: grid;
        gap: 3px;
        padding-left: 10px;
      }

      .actor {
        border-left-color: var(--arb-accent);
      }

      .meta {
        color: var(--arb-muted);
        font-size: 0.85rem;
      }

      .chip-row {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
      }

      .chip {
        border: 1px solid var(--arb-border);
        border-radius: 999px;
        font-size: 0.78rem;
        padding: 2px 8px;
      }

      .timeline,
      .trace,
      .final-state {
        display: grid;
        gap: 10px;
      }

      .event,
      .trace-entry {
        display: grid;
        gap: 3px;
      }

      .event-sequence {
        color: var(--arb-accent-strong);
        font-weight: 700;
      }

      .trace-group {
        display: grid;
        gap: 6px;
      }

      @media (max-width: 820px) {
        .workspace {
          padding: 16px;
        }

        .layout,
        .split {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <main class="workspace">
      <header>
        <h1>ASHA Rulebench</h1>
        <p class="summary">{{ scenario().title }} · {{ scenario().seedLabel }}</p>
        <p>{{ scenario().summary }}</p>
      </header>

      <section class="layout">
        <div class="panel" aria-label="Scenario board">
          <h2>Board</h2>
          <div class="board" [style.grid-template-columns]="'repeat(' + scenario().board.width + ', minmax(36px, 1fr))'">
            @for (cell of scenario().board.cells; track cell.x + ':' + cell.y) {
              <div class="cell" [class.cell-cover]="cell.terrainLabel.includes('cover')" [attr.aria-label]="'Cell ' + cell.x + ', ' + cell.y">
                @for (occupantId of cell.occupantIds; track occupantId) {
                  <span class="occupant">{{ occupantId.slice(7, 8).toUpperCase() }}</span>
                }
              </div>
            }
          </div>
        </div>

        <div class="panel" aria-label="Selected action">
          <h2>{{ scenario().selectedAction.name }}</h2>
          <p class="meta">Actor: {{ scenario().selectedAction.actorLabel }}</p>
          <p class="meta">Target: {{ scenario().selectedAction.targetLabels.join(', ') }}</p>
          <p>{{ scenario().selectedAction.actionText }}</p>
          <p>{{ scenario().selectedAction.effectText }}</p>
          <p class="meta">{{ scenario().selectedTarget.legalityLabel }}: {{ scenario().selectedTarget.reason }}</p>
        </div>
      </section>

      <section class="split">
        <div class="panel" aria-label="Combatants">
          <h2>Combatants</h2>
          <div class="list">
            @for (combatant of scenario().combatants; track combatant.id) {
              <article class="row" [class.actor]="combatant.isActor">
                <h3>{{ combatant.name }}</h3>
                <p class="meta">{{ combatant.teamLabel }} · {{ combatant.positionLabel }} · {{ combatant.hitPointLabel }}</p>
                <div class="chip-row">
                  @for (defense of combatant.defenseLabels; track defense) {
                    <span class="chip">{{ defense }}</span>
                  }
                  @for (condition of combatant.conditionLabels; track condition) {
                    <span class="chip">{{ condition }}</span>
                  }
                </div>
              </article>
            }
          </div>
        </div>

        <div class="panel timeline" aria-label="DomainEvents timeline">
          <h2>DomainEvents</h2>
          @for (event of scenario().timeline; track event.sequenceLabel) {
            <article class="event">
              <p><span class="event-sequence">{{ event.sequenceLabel }}</span> {{ event.typeLabel }}</p>
              <p>{{ event.summary }}</p>
              <p class="meta">{{ event.participantLabels.join(', ') }}</p>
            </article>
          }
        </div>
      </section>

      <section class="split">
        <div class="panel trace" aria-label="Rule trace">
          <h2>Rule Trace</h2>
          @for (group of scenario().traceGroups; track group.phaseLabel) {
            <article class="trace-group">
              <h3>{{ group.phaseLabel }}</h3>
              @for (entry of group.entries; track entry.sequenceLabel) {
                <div class="trace-entry">
                  <p>{{ entry.sequenceLabel }} · {{ entry.statusLabel }} · {{ entry.message }}</p>
                  <p class="meta">{{ entry.detail }}</p>
                </div>
              }
            </article>
          }
        </div>

        <div class="panel final-state" aria-label="Final state">
          <h2>Final State</h2>
          <p>{{ scenario().finalState.summary }}</p>
          @for (combatant of scenario().finalState.combatants; track combatant.id) {
            <article class="row">
              <h3>{{ combatant.name }}</h3>
              <p class="meta">{{ combatant.hitPointLabel }}</p>
              <div class="chip-row">
                @for (condition of combatant.conditionLabels; track condition) {
                  <span class="chip">{{ condition }}</span>
                }
              </div>
            </article>
          }
        </div>
      </section>
    </main>
  `,
})
export class RulebenchScenarioRendererComponent {
  readonly scenario = input.required<RulebenchScenarioView>();
}
