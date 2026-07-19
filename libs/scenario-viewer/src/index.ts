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
import type { GameplayActionView } from '@asha-rulebench/domain';
import type { RulesetCompileRequestDto } from '@asha-rulebench/protocol';
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

      .manifest-fields {
        display: grid;
        gap: 0.65rem;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        max-width: 72rem;
      }

      .manifest-fields label {
        color: var(--arb-muted);
        display: grid;
        font-size: 0.78rem;
        gap: 0.3rem;
      }

      .manifest-input {
        background: var(--arb-bg);
        border: 1px solid var(--arb-border);
        color: var(--arb-foreground);
        font: inherit;
        min-height: 2.8rem;
        padding: 0.65rem;
        width: 100%;
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

      .action-card {
        border: 1px solid var(--arb-border);
        display: grid;
        gap: 0.65rem;
        padding: 0.9rem;
      }

      .action-card.selected {
        border-color: var(--arb-accent-strong);
      }

      .evidence-input {
        background: var(--arb-bg);
        border: 1px solid var(--arb-border);
        color: var(--arb-foreground);
        min-height: 2.8rem;
        padding: 0.65rem;
        width: 100%;
      }

      .accepted {
        border-left: 3px solid var(--arb-accent-strong);
        padding-left: 0.8rem;
      }

      @media (max-width: 50rem) {
        .manifest-fields,
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
          <div class="manifest-fields" aria-label="Ruleset manifest entrypoint">
            <label>
              Workspace root
              <input
                #workspaceRootInput
                class="manifest-input"
                [disabled]="store.busy()"
                placeholder="/home/dev/my-game"
                [value]="workspaceRoot()"
                (input)="setWorkspaceRoot(workspaceRootInput.value)"
              />
            </label>
            <label>
              Package roots (comma separated)
              <input
                #packageRootsInput
                class="manifest-input"
                [disabled]="store.busy()"
                placeholder="packages/rules, packages/content"
                [value]="packageRootsText()"
                (input)="setPackageRoots(packageRootsInput.value)"
              />
            </label>
            <label>
              Root module
              <input
                #moduleInput
                class="manifest-input"
                [disabled]="store.busy()"
                placeholder="packages/rules/src/ruleset.ts"
                [value]="rootModule()"
                (input)="setRootModule(moduleInput.value)"
              />
            </label>
            <label>
              Exported declaration
              <input
                #declarationInput
                class="manifest-input"
                [disabled]="store.busy()"
                placeholder="ruleset"
                [value]="declaration()"
                (input)="setDeclaration(declarationInput.value)"
              />
            </label>
          </div>
          <p class="summary">
            Selected workspace: <strong>{{ workspaceIdentity() }}</strong
            >. The compile click builds only this explicit module graph in the
            trusted authoring subprocess before contacting Rust.
          </p>
          <div class="actions" aria-label="Ruleset lifecycle controls">
            <button
              type="button"
              [disabled]="store.busy() || !manifestSelectionComplete()"
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
              <p class="summary">
                Gameplay session:
                <strong>{{
                  view.gameplayAvailable ? 'available' : 'inactive'
                }}</strong>
              </p>
            </div>
          </arb-workbench-panel>

          @if (view.gameplay; as gameplay) {
            <arb-workbench-panel
              [panelNumber]="3"
              panelTitle="Persistent Rust authority session"
            >
              <div class="panel-body">
                <p class="section-label">
                  Revision {{ gameplay.stateRevision }} · actor
                  {{ gameplay.actorId }} ·
                  {{ gameplay.acceptedRandomValues }} accepted random values
                </p>
                <p class="summary">
                  Select an action and a Rust-provided candidate. Random
                  evidence is explicit comma-separated die results; Rulebench
                  does not roll or interpret the action program.
                </p>
                <ul class="row-list">
                  @for (action of gameplay.actions; track action.id) {
                    <li
                      class="action-card"
                      [class.selected]="selectedActionId() === action.id"
                    >
                      <strong>{{ action.name }}</strong>
                      <code>{{ action.id }}</code>
                      <span class="muted">Source: {{ action.source }}</span>
                      <span>
                        Candidates:
                        {{
                          action.candidateIds.length === 0
                            ? 'none at this revision'
                            : action.candidateIds.join(', ')
                        }}
                      </span>
                      <span>
                        Costs:
                        {{
                          action.costs.length === 0
                            ? 'none'
                            : action.costs.join(', ')
                        }}
                      </span>
                      <span>Random requirements by authority branch:</span>
                      @if (action.randomPlan.length === 0) {
                        <span class="muted">none</span>
                      } @else {
                        @for (
                          randomPath of action.randomPlan;
                          track randomPath
                        ) {
                          <span class="muted">{{ randomPath }}</span>
                        }
                      }
                      @for (
                        preflight of action.preflight;
                        track preflight.targetId
                      ) {
                        <span class="muted">
                          {{ preflight.targetId }} ·
                          {{
                            preflight.available ? 'available' : 'unavailable'
                          }}
                          ·
                          {{ preflight.message }}
                        </span>
                      }
                      <button
                        class="secondary"
                        type="button"
                        [disabled]="store.busy()"
                        (click)="selectAction(action.id, action.candidateIds)"
                      >
                        Select action
                      </button>
                    </li>
                  }
                </ul>

                @if (selectedAction(); as action) {
                  <p class="section-label">Command {{ action.name }}</p>
                  <div class="actions" aria-label="Rust candidate selection">
                    @for (candidate of action.candidateIds; track candidate) {
                      <button
                        class="secondary"
                        type="button"
                        [attr.aria-pressed]="selectedTargetId() === candidate"
                        [disabled]="store.busy()"
                        (click)="selectedTargetId.set(candidate)"
                      >
                        Target {{ candidate }}
                      </button>
                    }
                  </div>
                  <label for="random-evidence">Random evidence</label>
                  <input
                    #randomEvidenceInput
                    id="random-evidence"
                    class="evidence-input"
                    [value]="randomEvidence()"
                    placeholder="Example: 12, 3, 4"
                    (input)="setRandomEvidence(randomEvidenceInput.value)"
                  />
                  @if (evidenceError(); as error) {
                    <p class="diagnostic">{{ error }}</p>
                  }
                  <button
                    type="button"
                    [disabled]="
                      store.busy() ||
                      selectedTargetId() === null ||
                      gameplay.pendingReaction !== null
                    "
                    (click)="executeAction()"
                  >
                    Submit typed intent
                  </button>
                }

                @if (gameplay.pendingReaction; as reaction) {
                  <div class="action-card selected" role="status">
                    <strong>Reaction pending: {{ reaction.reactionId }}</strong>
                    <span>
                      {{ reaction.actionId }} against
                      {{ reaction.targetId }} remains staged at revision
                      {{ gameplay.stateRevision }}.
                    </span>
                    <div class="actions">
                      @for (option of reaction.options; track option.id) {
                        <button
                          type="button"
                          [disabled]="store.busy()"
                          (click)="
                            resolveReaction(reaction.reactionId, option.id)
                          "
                        >
                          {{ option.label }} (reduce
                          {{ option.damageReduction }})
                        </button>
                      }
                      <button
                        class="secondary"
                        type="button"
                        [disabled]="store.busy()"
                        (click)="resolveReaction(reaction.reactionId, null)"
                      >
                        Decline reaction
                      </button>
                    </div>
                  </div>
                }
              </div>
            </arb-workbench-panel>

            <arb-workbench-panel [panelNumber]="4" panelTitle="Authority state">
              <div class="panel-body">
                <ul class="row-list">
                  @for (entity of gameplay.entities; track entity.id) {
                    <li>
                      <strong>{{ entity.id }} · {{ entity.team }}</strong>
                      <span
                        >Position {{ entity.position }} · vitality
                        {{ entity.vitality }}</span
                      >
                      <span>Stats: {{ entity.stats.join(', ') }}</span>
                      <span>Defenses: {{ entity.defenses.join(', ') }}</span>
                      <span>Resources: {{ entity.resources.join(', ') }}</span>
                      <span>
                        Modifiers:
                        {{
                          entity.modifiers.length === 0
                            ? 'none'
                            : entity.modifiers.join(', ')
                        }}
                      </span>
                    </li>
                  }
                </ul>
              </div>
            </arb-workbench-panel>

            @if (gameplay.result; as result) {
              <arb-workbench-panel
                [panelNumber]="5"
                panelTitle="Last authority outcome"
              >
                <div class="panel-body">
                  <p class="accepted">
                    <strong>{{ result.status }}</strong> · {{ result.message }}
                  </p>
                  @if (result.code) {
                    <code>{{ result.code }}</code>
                  }
                  <span>Random consumed: {{ result.randomConsumed }}</span>
                  @if (result.randomRequest) {
                    <span>Random requested: {{ result.randomRequest }}</span>
                  }
                  <p class="section-label">Accepted events</p>
                  <ul class="row-list">
                    @for (event of result.events; track event) {
                      <li>{{ event }}</li>
                    }
                  </ul>
                  <p class="section-label">Trace</p>
                  <ul class="row-list">
                    @for (trace of result.trace; track trace) {
                      <li>
                        <code>{{ trace }}</code>
                      </li>
                    }
                  </ul>
                </div>
              </arb-workbench-panel>
            }

            <arb-workbench-panel
              data-testid="replay-archive-panel"
              [panelNumber]="6"
              panelTitle="Portable checkpoint and replay archive"
            >
              <div class="panel-body">
                <p class="section-label">
                  {{ gameplay.archive.checkpointSchema }} · replay schema
                  {{ gameplay.archive.replaySchemaVersion }} · event schema
                  {{ gameplay.archive.eventSchemaVersion }}
                </p>
                <p class="summary">
                  Rulebench stores and inspects this record. Exact artifact
                  restore, pending-phase reconstruction, and replay remain Rust
                  authority operations.
                </p>
                <dl class="facts">
                  <div>
                    <dt>Pinned artifact</dt>
                    <dd>{{ gameplay.archive.artifactId }}</dd>
                  </div>
                  <div>
                    <dt>Artifact schema</dt>
                    <dd>{{ gameplay.archive.artifactSchema }}</dd>
                  </div>
                  <div>
                    <dt>Composition</dt>
                    <dd>{{ gameplay.archive.composition }}</dd>
                  </div>
                  <div>
                    <dt>Language</dt>
                    <dd>{{ gameplay.archive.language }}</dd>
                  </div>
                  <div>
                    <dt>Checkpoint phase</dt>
                    <dd>{{ gameplay.archive.phase }}</dd>
                  </div>
                  <div>
                    <dt>State revision</dt>
                    <dd>{{ gameplay.archive.stateRevision }}</dd>
                  </div>
                  <div>
                    <dt>Accepted random position</dt>
                    <dd>{{ gameplay.archive.acceptedRandomPosition }}</dd>
                  </div>
                  <div>
                    <dt>Encoded checkpoint</dt>
                    <dd>{{ gameplay.archive.checkpointBytes }} bytes</dd>
                  </div>
                </dl>
                <p class="section-label">Canonical session state hash</p>
                <code>{{ gameplay.archive.stateHash }}</code>
                <p class="section-label">Exact source package bindings</p>
                <ul class="row-list">
                  @for (
                    source of gameplay.archive.sourcePackages;
                    track source
                  ) {
                    <li>
                      <code>{{ source }}</code>
                    </li>
                  }
                </ul>
                <p class="section-label">Exact dependency lock</p>
                <ul class="row-list">
                  @for (edge of gameplay.archive.dependencyLock; track edge) {
                    <li>
                      <code>{{ edge }}</code>
                    </li>
                  }
                </ul>
                <p class="section-label">Artifact fingerprint planes</p>
                <ul class="row-list">
                  @for (
                    fingerprint of gameplay.archive.fingerprints;
                    track fingerprint
                  ) {
                    <li>
                      <code>{{ fingerprint }}</code>
                    </li>
                  }
                </ul>
                <p class="section-label">Authority schema bindings</p>
                <ul class="row-list">
                  @for (
                    operation of gameplay.archive.operationSchemas;
                    track operation
                  ) {
                    <li>
                      <code>{{ operation }}</code>
                    </li>
                  }
                  @for (
                    capability of gameplay.archive.capabilitySchemas;
                    track capability
                  ) {
                    <li>
                      <code>{{ capability }}</code>
                    </li>
                  }
                </ul>
                <p class="section-label">Definition fingerprints</p>
                <ul class="row-list">
                  @for (
                    definition of gameplay.archive.definitionFingerprints;
                    track definition
                  ) {
                    <li>
                      <code>{{ definition }}</code>
                    </li>
                  }
                </ul>
                <div class="actions" aria-label="Portable archive actions">
                  <button
                    data-testid="restore-checkpoint"
                    class="secondary"
                    type="button"
                    [disabled]="store.busy()"
                    (click)="restoreCheckpoint()"
                  >
                    Restore stored checkpoint
                  </button>
                  <button
                    data-testid="replay-records"
                    type="button"
                    [disabled]="store.busy()"
                    (click)="replayArchive()"
                  >
                    Replay stored records in Rust
                  </button>
                </div>
                <p role="status" class="accepted">
                  <strong>{{ gameplay.archive.verificationStatus }}</strong> ·
                  {{ gameplay.archive.verificationMessage }}
                </p>
                <p class="section-label">
                  {{ gameplay.archive.replayEntries.length }} stored replay
                  record(s)
                </p>
                <ul class="row-list" data-testid="replay-records-list">
                  @for (
                    entry of gameplay.archive.replayEntries;
                    track entry.sequence
                  ) {
                    <li>
                      <strong
                        >{{ entry.sequence }}. {{ entry.operation }} ·
                        {{ entry.outcome }}</strong
                      >
                      <code>{{ entry.transition }}</code>
                      @for (evidence of entry.randomEvidence; track evidence) {
                        <span>Random: {{ evidence }}</span>
                      }
                      @for (event of entry.events; track event) {
                        <span>Event: {{ event }}</span>
                      }
                    </li>
                  }
                </ul>
              </div>
            </arb-workbench-panel>
          }

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
                    <dt>Materialization records</dt>
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

            @if (view.upgradeImpact; as impact) {
              <arb-workbench-panel
                [panelNumber]="5"
                panelTitle="Pre-activation upgrade impact"
              >
                <div class="panel-body">
                  <p class="section-label">
                    Candidate compared with active runtime truth
                  </p>
                  <code>{{ impact.transition }}</code>
                  <p class="summary">
                    This report compares fully materialized Rust-accepted
                    artifacts. Activation has not occurred.
                  </p>
                  <p class="section-label">Changed package sources</p>
                  <ul class="row-list">
                    @for (source of impact.sourceChanges; track source) {
                      <li>
                        <code>{{ source }}</code>
                      </li>
                    }
                  </ul>
                  <p class="section-label">
                    Changed definitions and effective fields
                  </p>
                  <ul class="row-list">
                    @for (
                      definition of impact.definitions;
                      track definition.definitionId
                    ) {
                      <li>
                        <strong>{{ definition.definitionId }}</strong>
                        <span>{{ definition.status }}</span>
                        @for (cause of definition.causes; track cause) {
                          <span class="muted">{{ cause }}</span>
                        }
                        @for (
                          field of definition.fields;
                          track field.plane + field.path
                        ) {
                          <span>{{ field.plane }} · {{ field.path }}</span>
                          <code>{{ field.transition }}</code>
                        }
                      </li>
                    }
                  </ul>
                </div>
              </arb-workbench-panel>
            }

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
                      <code>{{ definition.fingerprint }}</code>
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

            <arb-workbench-panel
              [panelNumber]="8"
              panelTitle="Materialization provenance"
            >
              <div class="panel-body">
                <p class="section-label">Base and ordered mixin chains</p>
                <ul class="row-list">
                  @for (
                    derivation of artifact.derivations;
                    track derivation.definitionId
                  ) {
                    <li>
                      <strong>{{ derivation.definitionId }}</strong>
                      <span
                        >{{ derivation.owner }} derives from
                        {{ derivation.base }}</span
                      >
                      <code>{{ derivation.baseFingerprint }}</code>
                      @for (mixin of derivation.mixins; track mixin.order) {
                        <span>
                          {{ mixin.order + 1 }}. {{ mixin.identity }} ·
                          {{ mixin.parameters }}
                        </span>
                        <code>{{ mixin.fingerprint }}</code>
                      }
                      <span>Local patch</span>
                      <code>{{ derivation.localPatchFingerprint }}</code>
                      @for (
                        change of derivation.changes;
                        track change.plane + change.path
                      ) {
                        <span>
                          {{ change.plane }} · {{ change.path }} ·
                          {{ change.effective ? 'effective' : 'ineffective' }}
                        </span>
                        <code>{{ change.transition }}</code>
                      }
                      <span>Materialized definition</span>
                      <code>{{ derivation.materializedFingerprint }}</code>
                    </li>
                  }
                </ul>
                <p class="section-label">Composition-ordered overlays</p>
                <ul class="row-list">
                  @for (overlay of artifact.overlays; track overlay.order) {
                    <li>
                      <strong>{{ overlay.overlay }}</strong>
                      <span>{{ overlay.target }} · {{ overlay.impact }}</span>
                      <span>Expected and observed before</span>
                      <code>{{ overlay.expectedFingerprint }}</code>
                      <code>{{ overlay.beforeFingerprint }}</code>
                      <span>Patch → materialized after</span>
                      <code>{{ overlay.patchFingerprint }}</code>
                      <code>{{ overlay.afterFingerprint }}</code>
                      @for (
                        change of overlay.changes;
                        track change.plane + change.path
                      ) {
                        <span>
                          {{ change.plane }} · {{ change.path }} ·
                          {{ change.effective ? 'effective' : 'ineffective' }}
                        </span>
                        <code>{{ change.transition }}</code>
                      }
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
                  The host starts without runtime truth. The compile action asks
                  the trusted authoring host to freshly prepare the selected
                  TypeScript package graph, then submits that explicit
                  composition to Rust; no directory or global registry is
                  scanned.
                </p>
              </div>
            </arb-workbench-panel>
          }

          @if (view.diagnostics.length > 0) {
            <arb-workbench-panel [panelNumber]="9" panelTitle="Diagnostics">
              <div class="panel-body">
                @for (
                  diagnostic of view.diagnostics;
                  track diagnostic.code + diagnostic.path
                ) {
                  <div class="diagnostic">
                    <strong>{{ diagnostic.code }}</strong>
                    <p>{{ diagnostic.path }} · {{ diagnostic.message }}</p>
                    @if (diagnostic.packageId) {
                      <p class="muted">Package: {{ diagnostic.packageId }}</p>
                    }
                    @if (diagnostic.definitionId) {
                      <p class="muted">
                        Definition: {{ diagnostic.definitionId }}
                      </p>
                    }
                    @if (diagnostic.source) {
                      <p class="muted">
                        Source: {{ diagnostic.source.module }}#{{
                          diagnostic.source.declaration
                        }}
                      </p>
                    }
                    @if (diagnostic.graphPath) {
                      <p class="muted">
                        Graph: {{ diagnostic.graphPath.join(' → ') }}
                      </p>
                    }
                    @if (diagnostic.expected || diagnostic.actual) {
                      <p class="muted">
                        Expected {{ diagnostic.expected ?? 'not supplied' }} ·
                        actual {{ diagnostic.actual ?? 'not supplied' }}
                      </p>
                    }
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
  protected readonly workspaceRoot = signal('');
  protected readonly packageRootsText = signal('');
  protected readonly rootModule = signal('');
  protected readonly declaration = signal('');
  protected readonly selectedActionId = signal<string | null>(null);
  protected readonly selectedTargetId = signal<string | null>(null);
  protected readonly randomEvidence = signal('');
  protected readonly evidenceError = signal<string | null>(null);
  protected readonly workspaceIdentity = () => {
    if (!this.manifestSelectionComplete()) {
      return 'No manifest entrypoint selected';
    }
    return `${this.workspaceRoot()}/${this.rootModule()}#${this.declaration()}`;
  };
  protected readonly manifestSelectionComplete = () =>
    this.workspaceRoot().trim().length > 0 &&
    this.packageRoots().length > 0 &&
    this.rootModule().trim().length > 0 &&
    this.declaration().trim().length > 0;

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
          id: 'persistent-authority',
          label: 'Persistent authority session',
          disabled: true,
        },
      ],
    },
  ];

  public ngOnInit(): void {
    void this.store.refresh();
  }

  protected compileRuleset(): void {
    const request: RulesetCompileRequestDto = {
      workspaceRoot: this.workspaceRoot(),
      packageRoots: this.packageRoots(),
      module: this.rootModule(),
      declaration: this.declaration(),
    };
    void this.store.compile(request);
  }

  protected setWorkspaceRoot(value: string): void {
    this.workspaceRoot.set(value);
  }

  protected setPackageRoots(value: string): void {
    this.packageRootsText.set(value);
  }

  protected setRootModule(value: string): void {
    this.rootModule.set(value);
  }

  protected setDeclaration(value: string): void {
    this.declaration.set(value);
  }

  private packageRoots(): string[] {
    return this.packageRootsText()
      .split(',')
      .map((root) => root.trim())
      .filter((root) => root.length > 0);
  }

  protected activateRuleset(): void {
    void this.store.activate();
  }

  protected restoreCheckpoint(): void {
    void this.store.restoreCheckpoint();
  }

  protected replayArchive(): void {
    void this.store.replay();
  }

  protected selectedAction(): GameplayActionView | null {
    const selected = this.selectedActionId();
    if (selected === null) return null;
    return (
      this.store
        .view()
        ?.gameplay?.actions.find((action) => action.id === selected) ?? null
    );
  }

  protected selectAction(
    actionId: string,
    candidateIds: readonly string[],
  ): void {
    this.selectedActionId.set(actionId);
    this.selectedTargetId.set(candidateIds[0] ?? null);
    this.evidenceError.set(null);
  }

  protected setRandomEvidence(value: string): void {
    this.randomEvidence.set(value);
    this.evidenceError.set(null);
  }

  protected executeAction(): void {
    const gameplay = this.store.view()?.gameplay;
    const actionId = this.selectedActionId();
    const targetId = this.selectedTargetId();
    const randomValues = this.parseRandomEvidence();
    if (
      gameplay === null ||
      gameplay === undefined ||
      actionId === null ||
      targetId === null ||
      randomValues === null
    ) {
      return;
    }
    void this.store.command({
      expectedRevision: gameplay.stateRevision,
      actionId,
      actorId: gameplay.actorId,
      targetIds: [targetId],
      randomValues,
    });
  }

  protected resolveReaction(reactionId: string, optionId: string | null): void {
    const gameplay = this.store.view()?.gameplay;
    const randomValues = this.parseRandomEvidence();
    if (gameplay === null || gameplay === undefined || randomValues === null) {
      return;
    }
    void this.store.react({
      expectedRevision: gameplay.stateRevision,
      reactionId,
      optionId,
      additionalRandomValues: randomValues,
    });
  }

  private parseRandomEvidence(): number[] | null {
    const source = this.randomEvidence().trim();
    if (source.length === 0) return [];
    const values = source.split(',').map((entry) => Number(entry.trim()));
    if (values.some((value) => !Number.isSafeInteger(value) || value <= 0)) {
      this.evidenceError.set(
        'Random evidence must be comma-separated positive integers.',
      );
      return null;
    }
    return values;
  }
}
