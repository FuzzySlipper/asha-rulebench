import {
  ChangeDetectionStrategy,
  Component,
  signal,
  type OnInit,
} from '@angular/core';
import {
  ApplicationMenubarComponent,
  type ApplicationMenuGroup,
  WorkbenchPanelComponent,
} from '@asha-rulebench/components';
import {
  prepareRulebenchRulesetSource,
  RULEBENCH_RULESET_SOURCE_OPTIONS,
  type RulebenchRulesetSourceId,
} from '@asha-rulebench/content-authoring';
import { createBrowserRulesetWorkspaceStore } from '@asha-rulebench/store';

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
        gap: 0.6rem;
        min-height: 15rem;
        padding: clamp(1.5rem, 5vw, 4rem);
      }

      .eyebrow,
      h1,
      h2,
      p,
      dl,
      dd,
      dt,
      ul {
        margin: 0;
      }

      .eyebrow,
      .section-label {
        color: var(--arb-accent-strong);
        font-size: 0.72rem;
        font-weight: 700;
        letter-spacing: 0.16em;
        text-transform: uppercase;
      }

      h1 {
        font-size: clamp(2.2rem, 5vw, 4.6rem);
        letter-spacing: -0.04em;
        line-height: 0.95;
        max-width: 13ch;
      }

      .summary {
        color: var(--arb-muted);
        max-width: 70ch;
      }

      .actions {
        display: flex;
        flex-wrap: wrap;
        gap: 0.65rem;
        margin-top: 0.6rem;
      }

      button {
        background: var(--arb-accent);
        border: 1px solid var(--arb-accent-strong);
        color: var(--arb-on-accent, #071b1a);
        cursor: pointer;
        font: inherit;
        font-weight: 700;
        min-height: 2.8rem;
        padding: 0.65rem 1rem;
      }

      button.secondary {
        background: transparent;
        color: var(--arb-foreground);
      }

      button:disabled {
        cursor: not-allowed;
        opacity: 0.45;
      }

      .panels {
        display: grid;
        gap: 1px;
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }

      .panel-body {
        display: grid;
        gap: 1rem;
        overflow-wrap: anywhere;
        padding: clamp(1rem, 2.5vw, 1.8rem);
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

      .status.active::before {
        background: var(--arb-accent-strong);
      }

      .facts {
        display: grid;
        gap: 0.75rem;
      }

      .facts > div,
      .row-list li {
        border-top: 1px solid var(--arb-border);
        display: grid;
        gap: 0.25rem;
        padding-top: 0.7rem;
      }

      dt,
      .muted {
        color: var(--arb-muted);
        font-size: 0.78rem;
      }

      dd,
      code {
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 0.82rem;
      }

      .row-list {
        display: grid;
        gap: 0.75rem;
        list-style: none;
        padding: 0;
      }

      .diagnostic {
        border-left: 3px solid var(--arb-warning);
        padding-left: 0.8rem;
      }

      .non-claim {
        border-left: 3px solid var(--arb-accent);
        color: var(--arb-muted);
        padding-left: 0.9rem;
      }

      @media (max-width: 50rem) {
        .panels {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <main class="workspace" aria-label="Rulebench ruleset workspace">
      <arb-workbench-panel
        [panelNumber]="1"
        panelTitle="Application menu"
        [compact]="true"
        [overlayTools]="true"
      >
        <arb-application-menubar
          panelTools
          [groups]="menuGroups"
          [statusMessage]="
            store.view()?.headline ?? 'Connecting to Rust compiler'
          "
        />
      </arb-workbench-panel>

      @if (store.view(); as view) {
        <header class="masthead">
          <p class="eyebrow">ASHA Rulebench · explicit artifact lifecycle</p>
          <h1>{{ view.headline }}</h1>
          <p class="summary">{{ view.summary }}</p>
          <div class="actions" aria-label="TypeScript ruleset source">
            @for (source of sourceOptions; track source.id) {
              <button
                class="secondary"
                type="button"
                [attr.aria-pressed]="selectedSourceId() === source.id"
                [disabled]="store.busy()"
                (click)="selectSource(source.id)"
              >
                Use {{ source.label }}
              </button>
            }
          </div>
          <p class="summary">
            Selected source: <strong>{{ selectedSourceLabel() }}</strong
            >. The compile click prepares this TypeScript package graph again in
            the browser before contacting Rust.
          </p>
          <div class="actions" aria-label="Ruleset lifecycle controls">
            <button
              type="button"
              [disabled]="store.busy()"
              (click)="compileRuleset()"
            >
              Compile explicit manifest
            </button>
            <button
              class="secondary"
              type="button"
              [disabled]="store.busy() || view.phase !== 'candidate'"
              (click)="activateRuleset()"
            >
              Activate accepted artifact
            </button>
          </div>
        </header>

        <section class="panels" aria-label="Ruleset compiler inspection">
          <arb-workbench-panel [panelNumber]="2" panelTitle="Lifecycle">
            <div class="panel-body">
              <p
                class="status"
                [class.active]="view.phase === 'active'"
                role="status"
              >
                <strong>{{ view.statusLabel }}</strong>
              </p>
              <dl class="facts">
                <div>
                  <dt>Activation revision</dt>
                  <dd>{{ view.activationRevision }}</dd>
                </div>
                <div>
                  <dt>Active artifact</dt>
                  <dd>{{ view.activeArtifactId ?? 'none' }}</dd>
                </div>
              </dl>
              <p class="non-claim">
                Gameplay execution unavailable. Task #5955 owns runtime sessions
                and visible action execution.
              </p>
            </div>
          </arb-workbench-panel>

          @if (view.artifact; as artifact) {
            <arb-workbench-panel
              [panelNumber]="3"
              panelTitle="Artifact identity"
            >
              <div class="panel-body">
                <p class="section-label">Closed portable artifact</p>
                <dl class="facts">
                  <div>
                    <dt>Artifact</dt>
                    <dd>{{ artifact.artifactId }}</dd>
                  </div>
                  <div>
                    <dt>Schema</dt>
                    <dd>{{ artifact.schema }}</dd>
                  </div>
                  <div>
                    <dt>Composition</dt>
                    <dd>{{ artifact.composition }}</dd>
                  </div>
                  <div>
                    <dt>Language</dt>
                    <dd>{{ artifact.language }}</dd>
                  </div>
                  <div>
                    <dt>Reserved slots</dt>
                    <dd>{{ artifact.reservedSlots }}</dd>
                  </div>
                </dl>
              </div>
            </arb-workbench-panel>

            <arb-workbench-panel
              [panelNumber]="4"
              panelTitle="Fingerprint planes"
            >
              <div class="panel-body">
                <ul class="row-list">
                  @for (
                    fingerprint of artifact.fingerprints;
                    track fingerprint.plane
                  ) {
                    <li>
                      <strong>{{ fingerprint.plane }}</strong>
                      <code>{{ fingerprint.value }}</code>
                    </li>
                  }
                </ul>
              </div>
            </arb-workbench-panel>

            <arb-workbench-panel
              [panelNumber]="5"
              panelTitle="Exact source and dependency lock"
            >
              <div class="panel-body">
                <p class="section-label">
                  {{ artifact.sources.length }} exact sources
                </p>
                <ul class="row-list">
                  @for (source of artifact.sources; track source.identity) {
                    <li>
                      <strong>{{ source.identity }}</strong
                      ><code>{{ source.fingerprint }}</code>
                    </li>
                  }
                </ul>
                <p class="section-label">
                  {{ artifact.lock.length }} lock edges
                </p>
                <ul class="row-list">
                  @for (
                    entry of artifact.lock;
                    track entry.requester + entry.importAs
                  ) {
                    <li>
                      <strong>{{ entry.resolution }}</strong>
                      <span
                        >{{ entry.relationship }} from {{ entry.requester }} as
                        {{ entry.importAs }}</span
                      >
                      <code>{{ entry.fingerprint }}</code>
                    </li>
                  }
                </ul>
              </div>
            </arb-workbench-panel>

            <arb-workbench-panel
              [panelNumber]="6"
              panelTitle="Exported-root closure"
            >
              <div class="panel-body">
                <p class="section-label">Exported roots</p>
                <ul class="row-list">
                  @for (root of artifact.exportedRoots; track root) {
                    <li>
                      <strong>{{ root }}</strong>
                    </li>
                  }
                </ul>
                <p class="section-label">Materialized definitions</p>
                <ul class="row-list">
                  @for (
                    definition of artifact.definitions;
                    track definition.id
                  ) {
                    <li>
                      <strong>{{ definition.label }}</strong>
                      <code>{{ definition.id }}</code>
                      <span>{{ definition.contract }}</span>
                      <span class="muted"
                        >{{ definition.owner }} · {{ definition.source }}</span
                      >
                      <span class="muted">
                        References:
                        {{
                          definition.references.length === 0
                            ? 'none'
                            : definition.references.join(', ')
                        }}
                      </span>
                    </li>
                  }
                </ul>
              </div>
            </arb-workbench-panel>

            <arb-workbench-panel
              [panelNumber]="7"
              panelTitle="Requirements and relationships"
            >
              <div class="panel-body">
                <p class="section-label">Rust-owned semantic requirements</p>
                <ul class="row-list">
                  @for (operation of artifact.operations; track operation) {
                    <li>
                      <strong>{{ operation }}</strong>
                    </li>
                  }
                  @for (capability of artifact.capabilities; track capability) {
                    <li>
                      <strong>{{ capability }}</strong>
                    </li>
                  }
                </ul>
                <p class="section-label">Provenance edges</p>
                <ul class="row-list">
                  @for (
                    relationship of artifact.relationships;
                    track relationship.kind + relationship.edge
                  ) {
                    <li>
                      <strong>{{ relationship.kind }}</strong>
                      <span>{{ relationship.edge }}</span>
                    </li>
                  }
                </ul>
              </div>
            </arb-workbench-panel>
          } @else {
            <arb-workbench-panel
              [panelNumber]="3"
              panelTitle="Compiler boundary"
            >
              <div class="panel-body">
                <strong>No candidate artifact</strong>
                <p>
                  The host starts without runtime truth. The compile action
                  freshly prepares the selected TypeScript package graph and
                  submits that explicit composition to Rust; no directory or
                  global registry is scanned.
                </p>
              </div>
            </arb-workbench-panel>
          }

          @if (view.diagnostics.length > 0) {
            <arb-workbench-panel [panelNumber]="8" panelTitle="Diagnostics">
              <div class="panel-body">
                @for (
                  diagnostic of view.diagnostics;
                  track diagnostic.code + diagnostic.path
                ) {
                  <div class="diagnostic">
                    <strong>{{ diagnostic.code }}</strong>
                    <p>{{ diagnostic.path }} · {{ diagnostic.message }}</p>
                  </div>
                }
              </div>
            </arb-workbench-panel>
          }
        </section>
      } @else {
        <header class="masthead">
          <p class="eyebrow">ASHA Rulebench</p>
          <h1>Connecting to Rust compiler</h1>
          <p class="summary">
            Loading the explicit ruleset lifecycle without activating content.
          </p>
        </header>
      }

      @if (store.state(); as state) {
        @if (state.kind === 'error') {
          <section class="panel-body" role="alert">
            <strong>Compiler transport unavailable</strong>
            <p>{{ state.message }}</p>
          </section>
        }
      }
    </main>
  `,
})
export class RulebenchWorkspaceFeatureComponent implements OnInit {
  protected readonly store = createBrowserRulesetWorkspaceStore();
  protected readonly sourceOptions = RULEBENCH_RULESET_SOURCE_OPTIONS;
  protected readonly selectedSourceId =
    signal<RulebenchRulesetSourceId>('fresh');
  protected readonly selectedSourceLabel = () =>
    this.sourceOptions.find((source) => source.id === this.selectedSourceId())
      ?.label ?? 'Unknown source';

  protected readonly menuGroups: readonly ApplicationMenuGroup[] = [
    {
      id: 'ruleset',
      label: 'Ruleset',
      items: [
        {
          id: 'explicit-compiler',
          label: 'Explicit compiler workspace',
          disabled: true,
        },
      ],
    },
    {
      id: 'run',
      label: 'Run',
      items: [
        {
          id: 'gameplay-unavailable',
          label: 'Gameplay unavailable (#5955)',
          disabled: true,
        },
      ],
    },
  ];

  public ngOnInit(): void {
    void this.store.refresh();
  }

  protected compileRuleset(): void {
    const preparation = prepareRulebenchRulesetSource(this.selectedSourceId());
    void this.store.compile(preparation);
  }

  protected selectSource(sourceId: RulebenchRulesetSourceId): void {
    this.selectedSourceId.set(sourceId);
  }

  protected activateRuleset(): void {
    void this.store.activate();
  }
}
