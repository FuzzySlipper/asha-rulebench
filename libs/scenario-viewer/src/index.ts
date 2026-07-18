import { ChangeDetectionStrategy, Component } from '@angular/core';
import {
  ApplicationMenubarComponent,
  type ApplicationMenuGroup,
  WorkbenchPanelComponent,
} from '@asha-rulebench/components';

@Component({
  selector: 'arb-rulebench-workspace-feature',
  imports: [ApplicationMenubarComponent, WorkbenchPanelComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  styles: [
    `
      :host {
        display: block;
        min-height: 100vh;
      }

      .workspace {
        display: grid;
        gap: 1px;
        min-height: 100vh;
        padding: 1px;
      }

      .masthead {
        align-content: end;
        background:
          linear-gradient(120deg, rgb(88 201 189 / 14%), transparent 42%),
          var(--arb-surface);
        border: 1px solid var(--arb-border);
        display: grid;
        gap: 0.45rem;
        min-height: 13rem;
        padding: clamp(1.5rem, 5vw, 4rem);
      }

      .eyebrow,
      h1,
      p {
        margin: 0;
      }

      .eyebrow {
        color: var(--arb-accent-strong);
        font-size: 0.72rem;
        font-weight: 700;
        letter-spacing: 0.16em;
        text-transform: uppercase;
      }

      h1 {
        font-size: clamp(2rem, 5vw, 4.5rem);
        letter-spacing: -0.04em;
        line-height: 0.95;
        max-width: 12ch;
      }

      .summary {
        color: var(--arb-muted);
        max-width: 66ch;
      }

      .panels {
        display: grid;
        gap: 1px;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        min-height: 21rem;
      }

      .state {
        align-content: start;
        display: grid;
        gap: 1rem;
        padding: clamp(1.25rem, 3vw, 2.25rem);
      }

      .state strong {
        font-size: 1.15rem;
      }

      .status {
        align-items: center;
        display: flex;
        gap: 0.65rem;
      }

      .status::before {
        background: var(--arb-warning);
        border-radius: 999px;
        content: '';
        height: 0.65rem;
        width: 0.65rem;
      }

      .next-boundary {
        border-left: 3px solid var(--arb-accent);
        color: var(--arb-muted);
        padding-left: 0.9rem;
      }

      @media (max-width: 44rem) {
        .panels {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <main class="workspace" aria-label="Rulebench empty workspace">
      <arb-workbench-panel
        [panelNumber]="1"
        panelTitle="Application menu"
        [compact]="true"
        [overlayTools]="true"
      >
        <arb-application-menubar
          panelTools
          [groups]="menuGroups"
          statusMessage="No compiled ruleset active"
        />
      </arb-workbench-panel>

      <header class="masthead">
        <p class="eyebrow">ASHA Rulebench</p>
        <h1>No compiled ruleset active</h1>
        <p class="summary">
          The legacy prototype corpus and its implicit authority paths have been
          removed. Rulebench has no actions, scenarios, sessions, or replay
          content to execute.
        </p>
      </header>

      <section class="panels" aria-label="Empty ruleset state">
        <arb-workbench-panel [panelNumber]="2" panelTitle="Ruleset status">
          <div class="state">
            <p class="status" role="status">
              <strong>No compiled ruleset active</strong>
            </p>
            <p>
              Execution controls remain unavailable until an explicit compiled
              artifact is selected.
            </p>
          </div>
        </arb-workbench-panel>

        <arb-workbench-panel [panelNumber]="3" panelTitle="Next authority boundary">
          <div class="state">
            <strong>Explicit manifests only</strong>
            <p class="next-boundary">
              Future content enters through the package manifest and compiler
              boundary introduced by Den task #5953. Files, imports, scenarios,
              and startup behavior do not define an active ruleset.
            </p>
          </div>
        </arb-workbench-panel>
      </section>
    </main>
  `,
})
export class RulebenchWorkspaceFeatureComponent {
  protected readonly menuGroups: readonly ApplicationMenuGroup[] = [
    {
      id: 'ruleset',
      label: 'Ruleset',
      items: [
        {
          id: 'no-active-ruleset',
          label: 'No compiled ruleset active',
          disabled: true,
        },
      ],
    },
    {
      id: 'run',
      label: 'Run',
      items: [
        {
          id: 'execution-unavailable',
          label: 'Execution unavailable',
          disabled: true,
        },
      ],
    },
  ];
}
