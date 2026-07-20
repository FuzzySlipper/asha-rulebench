import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  signal,
  type ElementRef,
  type OnInit,
  viewChild,
  viewChildren,
} from '@angular/core';
import {
  ApplicationDialogComponent,
  ApplicationMenubarComponent,
  type ApplicationMenuGroup,
  type ApplicationMenuItem,
  WorkbenchPanelComponent,
} from '@asha-rulebench/components';
import type {
  GameplayActionView,
  GameplayEntityView,
} from '@asha-rulebench/domain';
import type {
  EncounterInitialCapabilityDto,
  EncounterParticipantSetupDto,
  EncounterSetupRequestDto,
} from '@asha-rulebench/protocol';
import { createBrowserRulesetWorkspaceStore } from '@asha-rulebench/store';

type DialogName = 'ruleset' | 'encounter' | 'artifact' | 'replay' | null;

interface BoardCell {
  readonly x: number;
  readonly y: number;
  readonly entity: GameplayEntityView | null;
  readonly targetable: boolean;
}

@Component({
  selector: 'arb-rulebench-workspace-feature',
  imports: [
    ApplicationDialogComponent,
    ApplicationMenubarComponent,
    WorkbenchPanelComponent,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
  styles: [
    `
      :host {
        display: block;
        height: 100vh;
        height: 100dvh;
        min-height: 0;
        overflow: hidden;
      }

      .workspace {
        display: grid;
        gap: 1px;
        grid-template-rows: auto minmax(0, 1fr) auto;
        height: 100%;
        min-height: 0;
        overflow: hidden;
        padding: 1px;
      }

      .game-layout {
        display: grid;
        gap: 1px;
        grid-template-areas:
          'board board participants'
          'board board turn'
          'actions actions outcome';
        grid-template-columns: minmax(0, 1.35fr) minmax(0, 1fr) minmax(
            16rem,
            0.72fr
          );
        grid-template-rows: minmax(0, 1.2fr) minmax(0, 0.8fr) minmax(0, 1fr);
        min-height: 0;
        overflow: hidden;
      }

      .board-panel {
        grid-area: board;
      }

      .participants-panel {
        grid-area: participants;
      }

      .turn-panel {
        grid-area: turn;
      }

      .actions-panel {
        grid-area: actions;
      }

      .outcome-panel {
        grid-area: outcome;
      }

      .panel-body,
      .dialog-body {
        display: grid;
        gap: 0.9rem;
        overflow-wrap: anywhere;
        padding: clamp(0.85rem, 2vw, 1.35rem);
      }

      .eyebrow,
      .section-label,
      h3,
      p,
      ul,
      dl,
      dd,
      dt {
        margin: 0;
      }

      .eyebrow,
      .section-label {
        color: var(--arb-accent-strong);
        font-size: 0.7rem;
        font-weight: 750;
        letter-spacing: 0.14em;
        text-transform: uppercase;
      }

      h3 {
        font-size: 1rem;
      }

      .muted,
      dt {
        color: var(--arb-muted);
        font-size: 0.78rem;
      }

      code,
      dd {
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 0.8rem;
      }

      button,
      input,
      select {
        font: inherit;
      }

      button {
        background: var(--arb-accent);
        border: 1px solid var(--arb-accent-strong);
        color: var(--arb-on-accent, #071b1a);
        cursor: pointer;
        font-weight: 700;
        min-height: 2.65rem;
        padding: 0.55rem 0.8rem;
      }

      button.secondary,
      .action-choice {
        background: var(--arb-surface);
        border-color: var(--arb-border);
        color: var(--arb-foreground);
      }

      button:hover:not(:disabled),
      button[aria-pressed='true'] {
        border-color: var(--arb-accent-strong);
      }

      button:focus-visible,
      input:focus-visible,
      select:focus-visible {
        outline: 3px solid var(--arb-accent-strong);
        outline-offset: 2px;
      }

      button:disabled {
        cursor: not-allowed;
        opacity: 0.48;
      }

      .battlefield-wrap {
        align-items: center;
        display: grid;
        min-height: clamp(18rem, 45vh, 25rem);
        overflow: auto;
      }

      .battlefield {
        display: grid;
        gap: 0.45rem;
        margin: auto;
        max-width: 52rem;
        min-width: min(100%, 33rem);
        width: 100%;
      }

      .grid-cell {
        align-content: center;
        aspect-ratio: 1;
        background:
          linear-gradient(135deg, rgb(255 255 255 / 4%), transparent),
          var(--arb-bg);
        border: 1px solid var(--arb-border);
        color: var(--arb-muted);
        display: grid;
        font-size: 0.75rem;
        font-weight: 500;
        gap: 0.25rem;
        min-height: 4.75rem;
        padding: 0.45rem;
        position: relative;
      }

      .grid-cell.entity {
        color: var(--arb-foreground);
      }

      .grid-cell.current {
        box-shadow: inset 0 0 0 2px var(--arb-accent-strong);
      }

      .grid-cell.targetable {
        background: rgb(88 201 189 / 12%);
        border-color: var(--arb-accent-strong);
        color: var(--arb-foreground);
      }

      .grid-cell.targeted {
        background: var(--arb-accent);
        color: var(--arb-on-accent, #071b1a);
      }

      .cell-coordinate {
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 0.65rem;
        inset: 0.25rem auto auto 0.3rem;
        opacity: 0.7;
        position: absolute;
      }

      .piece {
        display: grid;
        gap: 0.15rem;
        justify-items: center;
      }

      .piece-token {
        align-items: center;
        background: var(--arb-surface);
        border: 2px solid var(--arb-accent-strong);
        border-radius: 50%;
        display: flex;
        font-size: 0.82rem;
        height: 2.35rem;
        justify-content: center;
        text-transform: uppercase;
        width: 2.35rem;
      }

      .piece.enemy .piece-token {
        border-color: var(--arb-warning);
      }

      .participant-list,
      .event-list,
      .detail-list,
      .action-list {
        display: grid;
        gap: 0.6rem;
        list-style: none;
        padding: 0;
      }

      .participant,
      .event-list li,
      .detail-list li {
        border-top: 1px solid var(--arb-border);
        display: grid;
        gap: 0.25rem;
        padding-top: 0.6rem;
      }

      .participant.current {
        border-left: 3px solid var(--arb-accent-strong);
        padding-left: 0.65rem;
      }

      .status-card,
      .reaction-card,
      .empty-state {
        border: 1px solid var(--arb-border);
        display: grid;
        gap: 0.65rem;
        padding: 0.8rem;
      }

      .reaction-card {
        border-color: var(--arb-warning);
      }

      .action-list {
        grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
      }

      .action-choice {
        display: grid;
        gap: 0.25rem;
        justify-items: start;
        min-height: 5rem;
        text-align: left;
      }

      .action-context {
        border-top: 1px solid var(--arb-border);
        display: grid;
        gap: 0.8rem;
        padding-top: 0.9rem;
      }

      .button-row,
      .target-row {
        display: flex;
        flex-wrap: wrap;
        gap: 0.55rem;
      }

      .roll-line {
        border-left: 3px solid var(--arb-accent-strong);
        display: grid;
        gap: 0.15rem;
        padding-left: 0.65rem;
      }

      .facts {
        display: grid;
        gap: 0.65rem;
      }

      .facts > div {
        border-top: 1px solid var(--arb-border);
        display: grid;
        gap: 0.2rem;
        padding-top: 0.6rem;
      }

      .ruleset-input,
      .ruleset-select,
      .setup-input,
      .setup-select {
        background: var(--arb-bg);
        border: 1px solid var(--arb-border);
        color: var(--arb-foreground);
        min-height: 2.8rem;
        padding: 0.6rem;
        width: 100%;
      }

      .setup-grid,
      .participant-editor {
        display: grid;
        gap: 0.75rem;
      }

      .setup-grid {
        grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
      }

      .participant-editor {
        border: 1px solid var(--arb-border);
        padding: 0.8rem;
      }

      .definition-choices {
        border: 0;
        display: grid;
        gap: 0.45rem;
        margin: 0;
        padding: 0;
      }

      .definition-choice {
        align-items: center;
        display: flex;
        gap: 0.5rem;
      }

      .diagnostic {
        border-left: 3px solid var(--arb-warning);
        display: grid;
        gap: 0.25rem;
        padding-left: 0.7rem;
      }

      @media (max-width: 60rem) {
        :host {
          height: auto;
          min-height: 100vh;
          min-height: 100dvh;
          overflow: visible;
        }

        .workspace {
          align-content: start;
          grid-template-rows: auto auto auto;
          height: auto;
          min-height: 100vh;
          min-height: 100dvh;
          overflow: visible;
        }

        .game-layout {
          grid-template-areas:
            'board participants'
            'turn participants'
            'actions actions'
            'outcome outcome';
          grid-template-columns: minmax(0, 1.5fr) minmax(15rem, 0.75fr);
          grid-template-rows:
            minmax(24rem, 55vh) minmax(10rem, 28vh) minmax(24rem, 55vh)
            minmax(22rem, 50vh);
          overflow: visible;
        }
      }

      @media (max-width: 44rem) {
        .game-layout {
          grid-template-areas: 'turn' 'participants' 'board' 'actions' 'outcome';
          grid-template-columns: minmax(0, 1fr);
          grid-template-rows:
            13rem minmax(19rem, 45vh) minmax(24rem, 58vh)
            minmax(30rem, 70vh) minmax(24rem, 58vh);
        }

        .battlefield-wrap {
          min-height: 19rem;
        }

        .battlefield {
          min-width: 29rem;
        }
      }
    `,
  ],
  template: `
    <main class="workspace" aria-label="Rulebench interactive combat workspace">
      <arb-workbench-panel
        [panelNumber]="0"
        panelTitle="Application menu"
        [compact]="true"
        [overlayTools]="true"
      >
        <arb-application-menubar
          panelTools
          [groups]="menuGroups()"
          [busy]="store.busy()"
          [statusMessage]="
            store.view()?.statusLabel ?? 'Connecting to Rust authority'
          "
          (itemInvoked)="handleMenuItem($event)"
        />
      </arb-workbench-panel>

      @if (store.view(); as view) {
        <section class="game-layout" aria-label="Combat workspace">
          <arb-workbench-panel
            #boardPanel
            class="board-panel"
            [panelNumber]="1"
            panelTitle="Battlefield"
          >
            <div class="panel-body">
              <div>
                <p class="eyebrow">
                  {{
                    view.phase === 'active'
                      ? 'Live Rust authority session'
                      : 'Inactive workspace'
                  }}
                </p>
                <p class="muted">
                  Choose an action, then choose a highlighted authority target
                  on the grid.
                </p>
              </div>

              @if (view.gameplay; as gameplay) {
                <div class="battlefield-wrap">
                  <div
                    class="battlefield"
                    role="grid"
                    aria-label="Combat grid"
                    [style.grid-template-columns]="boardColumns()"
                  >
                    @for (
                      cell of boardCells();
                      track cell.x + ':' + cell.y;
                      let index = $index
                    ) {
                      <button
                        #gridCell
                        class="grid-cell"
                        type="button"
                        role="gridcell"
                        [class.entity]="cell.entity !== null"
                        [class.current]="cell.entity?.id === gameplay.actorId"
                        [class.targetable]="cell.targetable"
                        [class.targeted]="
                          cell.entity !== null &&
                          cell.entity.id === selectedTargetId()
                        "
                        [attr.aria-rowindex]="cell.y + 1"
                        [attr.aria-colindex]="cell.x + 1"
                        [attr.aria-label]="cellLabel(cell, gameplay.actorId)"
                        [attr.aria-disabled]="cell.targetable ? null : true"
                        (click)="chooseGridCell(cell)"
                        (keydown)="moveGridFocus($event, index)"
                      >
                        <span class="cell-coordinate"
                          >{{ cell.x }},{{ cell.y }}</span
                        >
                        @if (cell.entity; as entity) {
                          <span
                            class="piece"
                            [class.enemy]="
                              isOpposingTeam(entity, gameplay.actorId)
                            "
                          >
                            <span class="piece-token" aria-hidden="true">{{
                              entity.id.slice(0, 2)
                            }}</span>
                            <strong>{{ entity.label }}</strong>
                            <span>{{ entity.vitality }}</span>
                          </span>
                        } @else {
                          <span aria-hidden="true">·</span>
                        }
                      </button>
                    }
                  </div>
                </div>
              } @else {
                <div class="empty-state">
                  <strong>No active encounter</strong>
                  <p>{{ view.summary }}</p>
                  <button
                    type="button"
                    (click)="
                      openDialog(
                        view.encounterSetupRequired ? 'encounter' : 'ruleset'
                      )
                    "
                  >
                    {{
                      view.encounterSetupRequired
                        ? 'Create encounter'
                        : 'Open ruleset setup'
                    }}
                  </button>
                </div>
              }
            </div>
          </arb-workbench-panel>

          <arb-workbench-panel
            class="participants-panel"
            [panelNumber]="2"
            panelTitle="Participants"
          >
            <div class="panel-body">
              @if (view.gameplay; as gameplay) {
                <ul
                  class="participant-list"
                  aria-label="Encounter participants"
                >
                  @for (entity of gameplay.entities; track entity.id) {
                    <li
                      class="participant"
                      [class.current]="entity.id === gameplay.actorId"
                    >
                      <strong>{{ entity.label }} · {{ entity.teamId }}</strong>
                      <code>{{ entity.id }}</code>
                      @if (entity.id === gameplay.actorId) {
                        <span class="section-label">Current actor</span>
                      }
                      <span
                        >Vitality {{ entity.vitality }} · cell
                        {{ entity.position }}</span
                      >
                      <span class="muted"
                        >Resources:
                        {{
                          entity.resources.length === 0
                            ? 'none'
                            : entity.resources.join(', ')
                        }}</span
                      >
                      <span class="muted"
                        >Modifiers:
                        {{
                          entity.modifiers.length === 0
                            ? 'none'
                            : entity.modifiers.join(', ')
                        }}</span
                      >
                    </li>
                  }
                </ul>
              } @else {
                <p class="muted">
                  Participants appear after an explicit encounter setup is
                  accepted.
                </p>
              }
            </div>
          </arb-workbench-panel>

          <arb-workbench-panel
            class="turn-panel"
            [panelNumber]="3"
            panelTitle="Turn status"
          >
            <div class="panel-body" aria-live="polite">
              <div class="status-card">
                <p class="section-label">{{ view.statusLabel }}</p>
                @if (view.gameplay; as gameplay) {
                  <strong>{{ gameplay.actorId }} is acting</strong>
                  <span
                    >Round {{ gameplay.turn.round }} · turn
                    {{ gameplay.turn.turn }}</span
                  >
                  <span class="muted"
                    >Initiative:
                    {{ gameplay.turn.initiativeOrder.join(' → ') }}</span
                  >
                  <span>Authority revision {{ gameplay.stateRevision }}</span>
                  <span class="muted">{{
                    gameplay.pendingReaction === null
                      ? 'Choose any available action.'
                      : 'A reaction must be resolved before play continues.'
                  }}</span>
                } @else {
                  <strong>{{ view.headline }}</strong>
                  <span class="muted"
                    >Gameplay remains inactive until explicit compilation and
                    activation.</span
                  >
                }
              </div>
            </div>
          </arb-workbench-panel>

          <arb-workbench-panel
            #actionPanel
            class="actions-panel"
            [panelNumber]="4"
            panelTitle="Action palette"
          >
            <div class="panel-body">
              @if (view.gameplay; as gameplay) {
                @if (gameplay.pendingReaction; as reaction) {
                  <div
                    #reactionPanel
                    class="reaction-card"
                    role="group"
                    aria-labelledby="reaction-title"
                    tabindex="-1"
                  >
                    <p class="section-label">Immediate choice</p>
                    <h3 id="reaction-title">
                      Reaction for {{ reaction.targetId }}
                    </h3>
                    <p>
                      {{ reaction.actionId }} is staged. Resolve
                      {{ reaction.reactionId }} here to continue.
                    </p>
                    <div class="button-row">
                      @for (option of reaction.options; track option.id) {
                        <button
                          type="button"
                          [disabled]="store.busy()"
                          (click)="
                            resolveReaction(reaction.reactionId, option.id)
                          "
                        >
                          {{ option.label }} · reduce
                          {{ option.damageReduction }}
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
                } @else {
                  <ul class="action-list" aria-label="Available actions">
                    @for (action of gameplay.actions; track action.id) {
                      <li>
                        <button
                          class="action-choice"
                          type="button"
                          [attr.aria-pressed]="selectedActionId() === action.id"
                          [disabled]="
                            store.busy() || !action.available
                          "
                          (click)="selectAction(action)"
                        >
                          <strong>{{ action.name }}</strong>
                          <code>{{ action.id }}</code>
                          <span
                            >{{ action.candidateIds.length }} available target{{
                              action.candidateIds.length === 1 ? '' : 's'
                            }}</span
                          >
                          <span class="muted"
                            >Up to {{ action.maximumTargets }} target{{
                              action.maximumTargets === 1 ? '' : 's'
                            }}</span
                          >
                          @if (action.unavailable) {
                            <span class="muted">{{ action.unavailable }}</span>
                          }
                        </button>
                      </li>
                    }
                  </ul>

                  @if (selectedAction(); as action) {
                    <div class="action-context" tabindex="-1">
                      <div>
                        <p class="section-label">Selected action</p>
                        <h3>{{ action.name }}</h3>
                        <code>{{ action.id }}</code>
                        <p class="muted">
                          Rust exposed these legal candidates at revision
                          {{ gameplay.stateRevision }}.
                        </p>
                      </div>
                      <div
                        class="target-row"
                        aria-label="Authority target choices"
                      >
                        @for (
                          candidate of action.candidateIds;
                          track candidate
                        ) {
                          <button
                            class="secondary"
                            type="button"
                            [attr.aria-pressed]="
                              selectedTargetId() === candidate
                            "
                            (click)="selectedTargetId.set(candidate)"
                          >
                            Target {{ candidate }}
                          </button>
                        }
                      </div>
                      <p class="muted">
                        Rolls happen automatically after you act. Rust requests
                        and consumes the exact dice for the branch it executes.
                      </p>
                      <button
                        type="button"
                        [disabled]="
                          store.busy() ||
                          (action.maximumTargets > 0 &&
                            selectedTargetId() === null)
                        "
                        (click)="executeAction()"
                      >
                        Use {{ action.name
                        }}{{
                          selectedTargetId() === null
                            ? ''
                            : ' on ' + selectedTargetId()
                        }}
                      </button>
                    </div>
                  } @else {
                    <p class="muted">
                      Choose an action to reveal its authority-provided targets.
                    </p>
                  }
                }
              } @else {
                <p class="muted">
                  The action palette opens when a ruleset owns an active
                  session.
                </p>
              }
            </div>
          </arb-workbench-panel>

          <arb-workbench-panel
            #outcomePanel
            class="outcome-panel"
            [panelNumber]="5"
            panelTitle="Combat log"
          >
            <div class="panel-body" aria-live="polite" aria-atomic="false">
              @if (interactionDiagnostics().length > 0) {
                <div
                  class="diagnostic"
                  role="alert"
                  aria-live="assertive"
                  aria-atomic="true"
                >
                  <strong>Gameplay request could not be completed</strong>
                  @for (
                    diagnostic of interactionDiagnostics();
                    track diagnostic.code + ':' + diagnostic.path
                  ) {
                    <span>
                      {{ diagnostic.code }} · {{ diagnostic.message }}
                    </span>
                  }
                </div>
              }
              @if (view.gameplay?.result; as result) {
                <div tabindex="-1">
                  <p class="section-label">{{ result.status }}</p>
                  <strong>{{ result.message }}</strong>
                </div>
                @for (
                  roll of result.randomEvidence;
                  track roll.path + ':' + $index
                ) {
                  <div class="roll-line">
                    <strong
                      >Automatic roll · {{ roll.dice }} →
                      {{ roll.values.join(', ') }}</strong
                    >
                    <span class="muted">{{ roll.kind }} · {{ roll.path }}</span>
                  </div>
                }
                @if (result.events.length > 0) {
                  <ul class="event-list" aria-label="Authority events">
                    @for (event of result.events; track $index) {
                      <li>{{ event }}</li>
                    }
                  </ul>
                }
                @if (result.code) {
                  <p class="diagnostic">
                    <strong>{{ result.code }}</strong>
                  </p>
                }
              }
              @if (view.gameplay; as gameplay) {
                <ul class="event-list" aria-label="Encounter history">
                  @for (entry of gameplay.log; track entry.sequence) {
                    <li>
                      <strong
                        >{{ entry.sequence }}. {{ entry.actorId }} ·
                        {{ entry.actionId }}</strong
                      >
                      <span class="muted"
                        >Authority revision {{ entry.stateRevision }}</span
                      >
                      @for (event of entry.events; track $index) {
                        <span>{{ event }}</span>
                      }
                    </li>
                  } @empty {
                    <li>No accepted actions yet.</li>
                  }
                </ul>
              } @else {
                <p class="muted">
                  Automatic rolls and accepted authority events will appear here
                  after an action.
                </p>
              }
            </div>
          </arb-workbench-panel>
        </section>

        @if (store.state(); as state) {
          @if (state.kind === 'error') {
            <section class="panel-body" role="alert">
              <strong>Rulebench host unavailable</strong>
              <p>{{ state.message }}</p>
            </section>
          }
        }
      } @else {
        <section class="panel-body" aria-live="polite">
          <p class="eyebrow">ASHA Rulebench</p>
          <h3>Connecting to Rust authority</h3>
        </section>
      }
    </main>

    <arb-application-dialog
      dialogId="ruleset-dialog"
      dialogTitle="Ruleset setup"
      dialogDescription="Choose a configured ruleset or custom root, inspect diagnostics, and activate an accepted artifact."
      [open]="openDialogName() === 'ruleset'"
      (closeRequested)="closeDialog()"
    >
      <div class="dialog-body">
        @if (store.view(); as view) {
          <p class="section-label">{{ view.statusLabel }}</p>
          <label for="configured-ruleset" class="section-label"
            >Configured ruleset</label
          >
          <select
            #configuredRulesetSelect
            id="configured-ruleset"
            class="ruleset-select"
            [disabled]="store.busy()"
            [value]="configuredRulesetId()"
            (change)="selectConfiguredRuleset(configuredRulesetSelect.value)"
          >
            <option value="">Choose a configured ruleset</option>
            @for (location of store.configuredRulesets(); track location.id) {
              <option [value]="location.id">{{ location.label }}</option>
            }
          </select>
          @if (store.rulesetConfigurationError(); as configurationError) {
            <div class="diagnostic" role="alert">
              <strong>Local ruleset configuration could not be loaded</strong>
              <span>{{ configurationError }}</span>
            </div>
          } @else if (store.configuredRulesets().length === 0) {
            <p class="muted">
              No local rulesets are configured. Custom roots remain available
              below.
            </p>
          }
          <label for="ruleset-root" class="section-label"
            >Custom ruleset root</label
          >
          <input
            #rulesetRootInput
            id="ruleset-root"
            class="ruleset-input"
            [disabled]="store.busy()"
            placeholder="rulesets/field-manual"
            [value]="store.rulesetRoot()"
            (input)="store.selectRulesetRoot(rulesetRootInput.value)"
          />
          <p class="muted">
            Rulebench infers <code>ruleset.ts#ruleset</code> inside the selected
            root and compiles only its closed package graph.
          </p>
          <div class="button-row" aria-label="Ruleset lifecycle controls">
            <button
              type="button"
              [disabled]="store.busy() || !rootSelectionComplete()"
              (click)="compileRuleset()"
            >
              Load ruleset candidate
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
          <dl class="facts">
            <div>
              <dt>Lifecycle</dt>
              <dd>{{ view.phase }}</dd>
            </div>
            <div>
              <dt>Active artifact</dt>
              <dd>{{ view.activeArtifactId ?? 'none' }}</dd>
            </div>
            <div>
              <dt>Activation revision</dt>
              <dd>{{ view.activationRevision }}</dd>
            </div>
          </dl>
          @for (
            diagnostic of view.diagnostics;
            track diagnostic.code + diagnostic.path
          ) {
            <div class="diagnostic" role="alert">
              <strong>{{ diagnostic.code }}</strong>
              <span>{{ diagnostic.path }} · {{ diagnostic.message }}</span>
            </div>
          }
        }
      </div>
    </arb-application-dialog>

    <arb-application-dialog
      dialogId="encounter-dialog"
      dialogTitle="Encounter setup"
      dialogDescription="Build an explicit artifact-pinned board and participant roster. Rust validates the complete setup before replacing any session."
      [open]="openDialogName() === 'encounter'"
      (closeRequested)="closeDialog()"
    >
      <div class="dialog-body">
        @if (setupDraft(); as setup) {
          <p class="section-label">Artifact binding</p>
          <code>{{ setup.artifactId }}</code>
          <div class="setup-grid">
            <label>
              <span class="section-label">Board width</span>
              <input
                #boardWidthInput
                class="setup-input"
                type="number"
                min="1"
                max="1024"
                [value]="setup.board.width"
                (input)="updateBoardExtent('width', boardWidthInput.value)"
              />
            </label>
            <label>
              <span class="section-label">Board height</span>
              <input
                #boardHeightInput
                class="setup-input"
                type="number"
                min="1"
                max="1024"
                [value]="setup.board.height"
                (input)="updateBoardExtent('height', boardHeightInput.value)"
              />
            </label>
            <label>
              <span class="section-label">Round</span>
              <input
                #roundInput
                class="setup-input"
                type="number"
                min="1"
                [value]="setup.turn.round"
                (input)="updateTurnCounter('round', roundInput.value)"
              />
            </label>
            <label>
              <span class="section-label">Turn</span>
              <input
                #turnInput
                class="setup-input"
                type="number"
                min="1"
                [value]="setup.turn.turn"
                (input)="updateTurnCounter('turn', turnInput.value)"
              />
            </label>
          </div>

          <div>
            <p class="section-label">Selected automatic random source</p>
            <code
              >{{ setup.randomSource.policyId }}@{{
                setup.randomSource.policyVersion
              }} · {{ setup.randomSource.sourceId }}@{{
                setup.randomSource.sourceVersion
              }}</code
            >
          </div>

          <div class="button-row">
            <button class="secondary" type="button" (click)="addParticipant()">
              Add participant
            </button>
            <button class="secondary" type="button" (click)="addTerrainCell()">
              Add terrain cell
            </button>
          </div>

          @for (
            participant of setup.participants;
            track $index;
            let participantIndex = $index
          ) {
            <section
              class="participant-editor"
              [attr.aria-label]="'Participant ' + (participantIndex + 1)"
            >
              <div class="button-row">
                <strong>Participant {{ participantIndex + 1 }}</strong>
                <button
                  class="secondary"
                  type="button"
                  [disabled]="participantIndex === 0"
                  (click)="moveParticipant(participantIndex, -1)"
                >
                  Move earlier
                </button>
                <button
                  class="secondary"
                  type="button"
                  [disabled]="participantIndex === setup.participants.length - 1"
                  (click)="moveParticipant(participantIndex, 1)"
                >
                  Move later
                </button>
                <button
                  class="secondary"
                  type="button"
                  (click)="removeParticipant(participantIndex)"
                >
                  Remove
                </button>
              </div>
              <div class="setup-grid">
                <label>
                  <span class="section-label">ID</span>
                  <input
                    #participantIdInput
                    class="setup-input"
                    [value]="participant.id"
                    (input)="
                      updateParticipantText(
                        participantIndex,
                        'id',
                        participantIdInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Label</span>
                  <input
                    #participantLabelInput
                    class="setup-input"
                    [value]="participant.label"
                    (input)="
                      updateParticipantText(
                        participantIndex,
                        'label',
                        participantLabelInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Team ID</span>
                  <input
                    #participantTeamInput
                    class="setup-input"
                    [value]="participant.teamId"
                    (input)="
                      updateParticipantText(
                        participantIndex,
                        'teamId',
                        participantTeamInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Position X</span>
                  <input
                    #participantXInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [value]="participant.position.x"
                    (input)="
                      updateParticipantPosition(
                        participantIndex,
                        'x',
                        participantXInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Position Y</span>
                  <input
                    #participantYInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [value]="participant.position.y"
                    (input)="
                      updateParticipantPosition(
                        participantIndex,
                        'y',
                        participantYInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Vitality</span>
                  <input
                    #vitalityInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [value]="capabilityValue(participant, 'vitality')"
                    (input)="
                      updateBoundedCapability(
                        participantIndex,
                        'vitality',
                        vitalityInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Power stat</span>
                  <input
                    #powerInput
                    class="setup-input"
                    type="number"
                    [value]="capabilityValue(participant, 'stat')"
                    (input)="
                      updateNumberCapability(
                        participantIndex,
                        'stat',
                        powerInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Guard defense</span>
                  <input
                    #guardInput
                    class="setup-input"
                    type="number"
                    [value]="capabilityValue(participant, 'defense')"
                    (input)="
                      updateNumberCapability(
                        participantIndex,
                        'defense',
                        guardInput.value
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Focus resource</span>
                  <input
                    #focusInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [value]="capabilityValue(participant, 'resource')"
                    (input)="
                      updateBoundedCapability(
                        participantIndex,
                        'resource',
                        focusInput.value
                      )
                    "
                  />
                </label>
              </div>
              <fieldset class="definition-choices">
                <legend class="section-label">Owned action definitions</legend>
                @for (definition of actionDefinitions(); track definition.id) {
                  <label class="definition-choice">
                    <input
                      type="checkbox"
                      [checked]="participant.definitionIds.includes(definition.id)"
                      (change)="
                        toggleParticipantDefinition(
                          participantIndex,
                          definition.id
                        )
                      "
                    />
                    <span>{{ definition.label }} · {{ definition.id }}</span>
                  </label>
                }
              </fieldset>
            </section>
          } @empty {
            <p class="muted">
              Add every participant explicitly. Setup contains no hidden roster
              or action script.
            </p>
          }

          @for (cell of setup.board.cells; track $index; let cellIndex = $index) {
            <section
              class="participant-editor"
              [attr.aria-label]="'Terrain cell ' + (cellIndex + 1)"
            >
              <div class="button-row">
                <strong>Terrain cell {{ cellIndex + 1 }}</strong>
                <button
                  class="secondary"
                  type="button"
                  (click)="removeTerrainCell(cellIndex)"
                >
                  Remove
                </button>
              </div>
              <div class="setup-grid">
                <label>
                  <span class="section-label">Cell ID</span>
                  <input
                    #cellIdInput
                    class="setup-input"
                    [value]="cell.id"
                    (input)="updateCellId(cellIndex, cellIdInput.value)"
                  />
                </label>
                <label>
                  <span class="section-label">Position X</span>
                  <input
                    #cellXInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [value]="cell.position.x"
                    (input)="updateCellPosition(cellIndex, 'x', cellXInput.value)"
                  />
                </label>
                <label>
                  <span class="section-label">Position Y</span>
                  <input
                    #cellYInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [value]="cell.position.y"
                    (input)="updateCellPosition(cellIndex, 'y', cellYInput.value)"
                  />
                </label>
                <label>
                  <span class="section-label">Traversal</span>
                  <select
                    #passableSelect
                    class="setup-select"
                    [value]="cellPassable(cell) ? 'passable' : 'blocked'"
                    (change)="
                      updateCellPassable(
                        cellIndex,
                        passableSelect.value === 'passable'
                      )
                    "
                  >
                    <option value="passable">Passable</option>
                    <option value="blocked">Blocked</option>
                  </select>
                </label>
              </div>
            </section>
          }

          <label for="current-actor" class="section-label"
            >Starting actor</label
          >
          <select
            #currentActorSelect
            id="current-actor"
            class="setup-select"
            [value]="setup.turn.currentActorId"
            (change)="setCurrentActor(currentActorSelect.value)"
          >
            @for (participant of setup.participants; track $index) {
              <option [value]="participant.id">
                {{ participant.label || participant.id }}
              </option>
            }
          </select>
          <p class="muted">
            Initiative follows the participant order shown above. Actions,
            targets, reactions, rolls, expected events, and winners are not part
            of setup.
          </p>

          @for (
            diagnostic of setupDiagnostics();
            track diagnostic.code + diagnostic.path
          ) {
            <div class="diagnostic" role="alert">
              <strong>{{ diagnostic.code }}</strong>
              <span>{{ diagnostic.path }} · {{ diagnostic.message }}</span>
            </div>
          }

          <div class="button-row">
            <button
              type="button"
              [disabled]="store.busy() || setup.participants.length === 0"
              (click)="startEncounter()"
            >
              Validate and start encounter
            </button>
            <button class="secondary" type="button" (click)="closeDialog()">
              Cancel
            </button>
          </div>
        } @else {
          <p>Activate an accepted artifact before creating an encounter.</p>
        }
      </div>
    </arb-application-dialog>

    <arb-application-dialog
      dialogId="artifact-dialog"
      dialogTitle="Artifact and provenance"
      dialogDescription="Secondary inspection for the closed Rust-accepted artifact."
      [open]="openDialogName() === 'artifact'"
      (closeRequested)="closeDialog()"
    >
      <div class="dialog-body">
        @if (store.view()?.artifact; as artifact) {
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
          </dl>
          <p class="section-label">Fingerprint planes</p>
          <ul class="detail-list">
            @for (
              fingerprint of artifact.fingerprints;
              track fingerprint.plane
            ) {
              <li>
                <strong>{{ fingerprint.plane }}</strong
                ><code>{{ fingerprint.value }}</code>
              </li>
            }
          </ul>
          <p class="section-label">Exact package sources</p>
          <ul class="detail-list">
            @for (source of artifact.sources; track source.identity) {
              <li>
                <strong>{{ source.identity }}</strong
                ><code>{{ source.fingerprint }}</code>
              </li>
            }
          </ul>
          <p class="section-label">Exported definition closure</p>
          <ul class="detail-list">
            @for (definition of artifact.definitions; track definition.id) {
              <li>
                <strong>{{ definition.label }}</strong>
                <code>{{ definition.id }}</code>
                <span>{{ definition.contract }} · {{ definition.owner }}</span>
                <span class="muted">{{ definition.source }}</span>
              </li>
            }
          </ul>
          <p class="section-label">Materialization provenance</p>
          <ul class="detail-list">
            @for (
              derivation of artifact.derivations;
              track derivation.definitionId
            ) {
              <li>
                <strong>{{ derivation.definitionId }}</strong
                ><span
                  >{{ derivation.owner }} derives from
                  {{ derivation.base }}</span
                ><code>{{ derivation.materializedFingerprint }}</code>
              </li>
            }
            @for (overlay of artifact.overlays; track overlay.order) {
              <li>
                <strong>{{ overlay.overlay }}</strong
                ><span>{{ overlay.target }} · {{ overlay.impact }}</span
                ><code
                  >{{ overlay.beforeFingerprint }} →
                  {{ overlay.afterFingerprint }}</code
                >
              </li>
            }
          </ul>
        } @else {
          <p>No compiled artifact is available to inspect.</p>
        }
      </div>
    </arb-application-dialog>

    <arb-application-dialog
      dialogId="replay-dialog"
      dialogTitle="Replay and checkpoint tools"
      dialogDescription="Secondary verification for exact authority commands and recorded roll evidence."
      [open]="openDialogName() === 'replay'"
      (closeRequested)="closeDialog()"
    >
      <div class="dialog-body">
        @if (store.view()?.gameplay; as gameplay) {
          <div class="button-row">
            <button
              type="button"
              [disabled]="store.busy()"
              (click)="replayArchive()"
            >
              Verify stored replay
            </button>
            <button
              class="secondary"
              type="button"
              [disabled]="store.busy()"
              (click)="restoreCheckpoint()"
            >
              Restore latest checkpoint
            </button>
          </div>
          <dl class="facts">
            <div>
              <dt>Checkpoint state</dt>
              <dd>
                revision {{ gameplay.archive.stateRevision }} ·
                {{ gameplay.archive.phase }}
              </dd>
            </div>
            <div>
              <dt>State hash</dt>
              <dd>{{ gameplay.archive.stateHash }}</dd>
            </div>
            <div>
              <dt>Verification</dt>
              <dd>
                {{ gameplay.archive.verificationStatus }} ·
                {{ gameplay.archive.verificationMessage }}
              </dd>
            </div>
          </dl>
          <ul class="detail-list" aria-label="Replay records">
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
                @for (evidence of entry.randomEvidence; track $index) {
                  <span>Recorded roll: {{ evidence }}</span>
                }
              </li>
            } @empty {
              <li>No gameplay records yet.</li>
            }
          </ul>
        } @else {
          <p>No active session is available to replay.</p>
        }
      </div>
    </arb-application-dialog>
  `,
})
export class RulebenchWorkspaceFeatureComponent implements OnInit {
  protected readonly store = createBrowserRulesetWorkspaceStore();
  protected readonly openDialogName = signal<DialogName>(null);
  protected readonly selectedActionId = signal<string | null>(null);
  protected readonly selectedTargetId = signal<string | null>(null);
  protected readonly setupDraft = signal<EncounterSetupRequestDto | null>(null);

  private readonly rulesetRootInput =
    viewChild<ElementRef<HTMLInputElement>>('rulesetRootInput');
  private readonly boardPanel =
    viewChild<WorkbenchPanelComponent>('boardPanel');
  private readonly actionPanel =
    viewChild<WorkbenchPanelComponent>('actionPanel');
  private readonly outcomePanel =
    viewChild<WorkbenchPanelComponent>('outcomePanel');
  private readonly reactionPanel =
    viewChild<ElementRef<HTMLElement>>('reactionPanel');
  private readonly gridCells =
    viewChildren<ElementRef<HTMLButtonElement>>('gridCell');

  protected readonly selectedAction = computed<GameplayActionView | null>(
    () => {
      const selectedActionId = this.selectedActionId();
      if (selectedActionId === null) return null;
      return (
        this.store
          .view()
          ?.gameplay?.actions.find(
            (action) => action.id === selectedActionId,
          ) ?? null
      );
    },
  );

  protected readonly boardWidth = computed(() => {
    return this.store.view()?.gameplay?.board.width ?? 1;
  });

  protected readonly boardHeight = computed(() => {
    return this.store.view()?.gameplay?.board.height ?? 1;
  });

  protected readonly boardColumns = computed(
    () => `repeat(${this.boardWidth()}, minmax(4.75rem, 1fr))`,
  );

  protected readonly boardCells = computed<readonly BoardCell[]>(() => {
    const entities = this.store.view()?.gameplay?.entities ?? [];
    const targetIds = new Set(this.selectedAction()?.candidateIds ?? []);
    const cells: BoardCell[] = [];
    for (let y = 0; y < this.boardHeight(); y += 1) {
      for (let x = 0; x < this.boardWidth(); x += 1) {
        const entity =
          entities.find(
            (candidate) => candidate.x === x && candidate.y === y,
          ) ?? null;
        cells.push({
          x,
          y,
          entity,
          targetable: entity !== null && targetIds.has(entity.id),
        });
      }
    }
    return cells;
  });

  protected readonly rootSelectionComplete = computed(
    () => this.store.rulesetRoot().trim().length > 0,
  );

  protected readonly configuredRulesetId = computed(() => {
    const selectedRoot = this.store.rulesetRoot();
    return (
      this.store
        .configuredRulesets()
        .find((location) => location.rulesetRoot === selectedRoot)?.id ?? ''
    );
  });

  protected readonly actionDefinitions = computed(() =>
    (this.store.view()?.artifact?.definitions ?? []).filter((definition) =>
      definition.contract.startsWith('action'),
    ),
  );

  protected readonly setupDiagnostics = computed(() =>
    (this.store.view()?.diagnostics ?? []).filter(
      (diagnostic) => diagnostic.stage === 'setup',
    ),
  );

  protected readonly interactionDiagnostics = computed(() => {
    if (this.store.busy()) return [];
    const view = this.store.view();
    if (view?.gameplay === null || view?.gameplay === undefined) return [];
    return view.diagnostics.filter(
      (diagnostic) =>
        diagnostic.stage === 'gameplay' || diagnostic.stage === 'replay',
    );
  });

  protected readonly menuGroups = computed<readonly ApplicationMenuGroup[]>(
    () => [
      {
        id: 'ruleset',
        label: 'Ruleset',
        items: [
          {
            id: 'load-ruleset-root',
            label: 'Load or switch ruleset…',
            disabled: this.store.busy(),
          },
          ...this.store.recentRulesetRoots().map((rulesetRoot, index) => ({
            id: `recent-ruleset-root-${index}`,
            label: `Switch to ${rulesetRoot}`,
            disabled: this.store.busy(),
          })),
          {
            id: 'inspect-artifact',
            label: 'Artifact and provenance…',
            disabled: this.store.view()?.artifact === null,
          },
        ],
      },
      {
        id: 'session',
        label: 'Session',
        items: [
          {
            id: 'setup-encounter',
            label: this.store.view()?.gameplayAvailable
              ? 'Start new encounter…'
              : 'Create encounter…',
            disabled:
              this.store.busy() || this.store.view()?.phase !== 'active',
          },
          {
            id: 'inspect-replay',
            label: 'Replay and checkpoints…',
            disabled: this.store.view()?.gameplay === null,
          },
          {
            id: 'focus-battlefield',
            label: 'Focus battlefield',
            disabled: this.store.view()?.gameplay === null,
          },
        ],
      },
    ],
  );

  public constructor() {
    effect(() => {
      const reaction = this.store.view()?.gameplay?.pendingReaction;
      const panel = this.reactionPanel();
      if (reaction !== null && reaction !== undefined && panel !== undefined) {
        panel.nativeElement.focus();
      }
    });
    effect(() => {
      const action = this.selectedAction();
      const panel = this.actionPanel();
      if (action !== null && panel !== undefined) panel.focus();
    });
    effect(() => {
      const result = this.store.view()?.gameplay?.result;
      const panel = this.outcomePanel();
      if (result !== null && result !== undefined && panel !== undefined) {
        panel.focus();
      }
    });
    effect(() => {
      const diagnostics = this.interactionDiagnostics();
      const panel = this.outcomePanel();
      if (diagnostics.length > 0 && panel !== undefined) panel.focus();
    });
  }

  public ngOnInit(): void {
    void this.store.refresh();
    void this.store.refreshConfiguredRulesets();
  }

  protected handleMenuItem(item: ApplicationMenuItem): void {
    if (item.id === 'load-ruleset-root') {
      this.openDialog('ruleset');
      return;
    }
    if (item.id === 'inspect-artifact') {
      this.openDialog('artifact');
      return;
    }
    if (item.id === 'inspect-replay') {
      this.openDialog('replay');
      return;
    }
    if (item.id === 'setup-encounter') {
      this.prepareEncounterDraft();
      this.openDialog('encounter');
      return;
    }
    if (item.id === 'focus-battlefield') {
      this.boardPanel()?.focus();
      return;
    }
    const recentIndex = recentRulesetRootIndex(item.id);
    if (recentIndex === null) return;
    const rulesetRoot = this.store.recentRulesetRoots()[recentIndex];
    if (rulesetRoot === undefined) return;
    this.store.selectRulesetRoot(rulesetRoot);
    this.openDialog('ruleset');
    void this.store.compile({ rulesetRoot });
  }

  protected openDialog(dialog: Exclude<DialogName, null>): void {
    this.openDialogName.set(dialog);
  }

  protected closeDialog(): void {
    this.openDialogName.set(null);
  }

  protected compileRuleset(): void {
    const rulesetRoot = this.store.rulesetRoot().trim();
    void this.store.compile({ rulesetRoot });
  }

  protected selectConfiguredRuleset(locationId: string): void {
    const location = this.store
      .configuredRulesets()
      .find((candidate) => candidate.id === locationId);
    this.store.selectRulesetRoot(location?.rulesetRoot ?? '');
  }

  protected activateRuleset(): void {
    void this.store.activate().then(() => {
      if (this.store.view()?.phase === 'active') {
        this.prepareEncounterDraft();
        this.openDialog('encounter');
      }
    });
  }

  protected prepareEncounterDraft(): void {
    const view = this.store.view();
    if (
      view?.activeArtifactId === null ||
      view?.activeArtifactId === undefined
    ) {
      this.setupDraft.set(null);
      return;
    }
    this.setupDraft.set({
      schema: { id: 'asha.rpg.encounter.setup', version: 1 },
      artifactId: view.activeArtifactId,
      board: { width: 5, height: 3, cells: [] },
      participants: [],
      turn: {
        initiativeOrder: [],
        currentActorId: '',
        round: 1,
        turn: 1,
      },
      randomSource: view.hostRandomSource,
    });
  }

  protected updateBoardExtent(field: 'width' | 'height', value: string): void {
    this.updateSetup((setup) => ({
      ...setup,
      board: { ...setup.board, [field]: formInteger(value) },
    }));
  }

  protected updateTurnCounter(field: 'round' | 'turn', value: string): void {
    this.updateSetup((setup) => ({
      ...setup,
      turn: { ...setup.turn, [field]: formInteger(value) },
    }));
  }

  protected addParticipant(): void {
    this.updateSetup((setup) => {
      const participantNumber = setup.participants.length + 1;
      const id = `participant-${participantNumber}`;
      const participant: EncounterParticipantSetupDto = {
        id,
        label: `Participant ${participantNumber}`,
        teamId: `team-${participantNumber}`,
        position: { x: participantNumber - 1, y: 0 },
        definitionIds: [],
        capabilities: [
          { owner: 'vitality', value: { current: 10, max: 10 } },
          { owner: 'stat', id: 'power', value: 0 },
          { owner: 'defense', id: 'guard', value: 10 },
          {
            owner: 'resource',
            id: 'focus',
            value: { current: 2, max: 2 },
          },
        ],
      };
      const participants = [...setup.participants, participant];
      const initiativeOrder = participants.map((entry) => entry.id);
      return {
        ...setup,
        participants,
        turn: {
          ...setup.turn,
          initiativeOrder,
          currentActorId: setup.turn.currentActorId || id,
        },
      };
    });
  }

  protected removeParticipant(index: number): void {
    this.updateSetup((setup) => {
      const participants = setup.participants.filter(
        (_participant, participantIndex) => participantIndex !== index,
      );
      const initiativeOrder = participants.map((entry) => entry.id);
      const currentActorId = initiativeOrder.includes(setup.turn.currentActorId)
        ? setup.turn.currentActorId
        : (initiativeOrder[0] ?? '');
      return {
        ...setup,
        participants,
        turn: { ...setup.turn, initiativeOrder, currentActorId },
      };
    });
  }

  protected moveParticipant(index: number, offset: -1 | 1): void {
    this.updateSetup((setup) => {
      const destination = index + offset;
      if (destination < 0 || destination >= setup.participants.length) {
        return setup;
      }
      const participants = [...setup.participants];
      const participant = participants[index];
      const displaced = participants[destination];
      if (participant === undefined || displaced === undefined) return setup;
      participants[index] = displaced;
      participants[destination] = participant;
      return {
        ...setup,
        participants,
        turn: {
          ...setup.turn,
          initiativeOrder: participants.map((entry) => entry.id),
        },
      };
    });
  }

  protected updateParticipantText(
    index: number,
    field: 'id' | 'label' | 'teamId',
    value: string,
  ): void {
    this.updateSetup((setup) => {
      const previousId = setup.participants[index]?.id;
      const participants = setup.participants.map((participant, entryIndex) =>
        entryIndex === index ? { ...participant, [field]: value } : participant,
      );
      const initiativeOrder = participants.map((participant) => participant.id);
      const currentActorId =
        field === 'id' && setup.turn.currentActorId === previousId
          ? value
          : setup.turn.currentActorId;
      return {
        ...setup,
        participants,
        turn: { ...setup.turn, initiativeOrder, currentActorId },
      };
    });
  }

  protected updateParticipantPosition(
    index: number,
    field: 'x' | 'y',
    value: string,
  ): void {
    this.updateParticipant(index, (participant) => ({
      ...participant,
      position: { ...participant.position, [field]: formInteger(value) },
    }));
  }

  protected toggleParticipantDefinition(
    index: number,
    definitionId: string,
  ): void {
    this.updateParticipant(index, (participant) => ({
      ...participant,
      definitionIds: participant.definitionIds.includes(definitionId)
        ? participant.definitionIds.filter((id) => id !== definitionId)
        : [...participant.definitionIds, definitionId],
    }));
  }

  protected capabilityValue(
    participant: EncounterParticipantSetupDto,
    owner: EncounterInitialCapabilityDto['owner'],
  ): number {
    const capability = participant.capabilities.find(
      (entry) => entry.owner === owner,
    );
    if (capability === undefined) return 0;
    return typeof capability.value === 'number'
      ? capability.value
      : capability.value.current;
  }

  protected updateNumberCapability(
    index: number,
    owner: 'stat' | 'defense',
    value: string,
  ): void {
    this.updateCapability(index, owner, (capability) => {
      if (capability.owner === 'stat') {
        return { ...capability, value: formSignedInteger(value) };
      }
      if (capability.owner === 'defense') {
        return { ...capability, value: formSignedInteger(value) };
      }
      return capability;
    });
  }

  protected updateBoundedCapability(
    index: number,
    owner: 'vitality' | 'resource',
    value: string,
  ): void {
    const next = formInteger(value);
    this.updateCapability(index, owner, (capability) => {
      if (capability.owner === 'vitality') {
        return { ...capability, value: { current: next, max: next } };
      }
      if (capability.owner === 'resource') {
        return { ...capability, value: { current: next, max: next } };
      }
      return capability;
    });
  }

  protected addTerrainCell(): void {
    this.updateSetup((setup) => {
      const cellNumber = setup.board.cells.length + 1;
      return {
        ...setup,
        board: {
          ...setup.board,
          cells: [
            ...setup.board.cells,
            {
              id: `terrain-${cellNumber}`,
              position: { x: 0, y: 0 },
              capabilities: [
                {
                  id: 'capability.traversal',
                  version: 1,
                  definitionId: null,
                  value: {
                    kind: 'traversal',
                    passable: true,
                    movementCost: 1,
                  },
                },
              ],
            },
          ],
        },
      };
    });
  }

  protected removeTerrainCell(index: number): void {
    this.updateSetup((setup) => ({
      ...setup,
      board: {
        ...setup.board,
        cells: setup.board.cells.filter(
          (_cell, cellIndex) => cellIndex !== index,
        ),
      },
    }));
  }

  protected updateCellId(index: number, id: string): void {
    this.updateCell(index, (cell) => ({ ...cell, id }));
  }

  protected updateCellPosition(
    index: number,
    field: 'x' | 'y',
    value: string,
  ): void {
    this.updateCell(index, (cell) => ({
      ...cell,
      position: { ...cell.position, [field]: formInteger(value) },
    }));
  }

  protected cellPassable(
    cell: EncounterSetupRequestDto['board']['cells'][number],
  ): boolean {
    const traversal = cell.capabilities.find(
      (capability) => capability.value.kind === 'traversal',
    );
    return traversal?.value.kind === 'traversal'
      ? traversal.value.passable
      : true;
  }

  protected updateCellPassable(index: number, passable: boolean): void {
    this.updateCell(index, (cell) => ({
      ...cell,
      capabilities: cell.capabilities.map((capability) =>
        capability.value.kind === 'traversal'
          ? {
              ...capability,
              value: { ...capability.value, passable },
            }
          : capability,
      ),
    }));
  }

  protected setCurrentActor(currentActorId: string): void {
    this.updateSetup((setup) => ({
      ...setup,
      turn: { ...setup.turn, currentActorId },
    }));
  }

  protected startEncounter(): void {
    const setup = this.setupDraft();
    if (setup === null) return;
    void this.store.startEncounter(setup).then((started) => {
      if (started) {
        this.selectedActionId.set(null);
        this.selectedTargetId.set(null);
        this.closeDialog();
        this.boardPanel()?.focus();
      }
    });
  }

  private updateSetup(
    update: (setup: EncounterSetupRequestDto) => EncounterSetupRequestDto,
  ): void {
    this.setupDraft.update((setup) => (setup === null ? null : update(setup)));
  }

  private updateParticipant(
    index: number,
    update: (
      participant: EncounterParticipantSetupDto,
    ) => EncounterParticipantSetupDto,
  ): void {
    this.updateSetup((setup) => ({
      ...setup,
      participants: setup.participants.map((participant, participantIndex) =>
        participantIndex === index ? update(participant) : participant,
      ),
    }));
  }

  private updateCapability(
    participantIndex: number,
    owner: EncounterInitialCapabilityDto['owner'],
    update: (
      capability: EncounterInitialCapabilityDto,
    ) => EncounterInitialCapabilityDto,
  ): void {
    this.updateParticipant(participantIndex, (participant) => ({
      ...participant,
      capabilities: participant.capabilities.map((capability) =>
        capability.owner === owner ? update(capability) : capability,
      ),
    }));
  }

  private updateCell(
    index: number,
    update: (
      cell: EncounterSetupRequestDto['board']['cells'][number],
    ) => EncounterSetupRequestDto['board']['cells'][number],
  ): void {
    this.updateSetup((setup) => ({
      ...setup,
      board: {
        ...setup.board,
        cells: setup.board.cells.map((cell, cellIndex) =>
          cellIndex === index ? update(cell) : cell,
        ),
      },
    }));
  }

  protected restoreCheckpoint(): void {
    void this.store.restoreCheckpoint();
  }

  protected replayArchive(): void {
    void this.store.replay();
  }

  protected selectAction(action: GameplayActionView): void {
    this.selectedActionId.set(action.id);
    this.selectedTargetId.set(null);
  }

  protected chooseGridCell(cell: BoardCell): void {
    if (!cell.targetable || cell.entity === null) return;
    this.selectedTargetId.set(cell.entity.id);
  }

  protected executeAction(): void {
    const gameplay = this.store.view()?.gameplay;
    const actionId = this.selectedActionId();
    const targetId = this.selectedTargetId();
    if (
      gameplay === null ||
      gameplay === undefined ||
      actionId === null ||
      (targetId === null && this.selectedAction()?.maximumTargets !== 0)
    ) {
      return;
    }
    this.selectedActionId.set(null);
    this.selectedTargetId.set(null);
    void this.store.command({
      expectedRevision: gameplay.stateRevision,
      actionId,
      actorId: gameplay.actorId,
      targetIds: targetId === null ? [] : [targetId],
    });
  }

  protected resolveReaction(reactionId: string, optionId: string | null): void {
    const gameplay = this.store.view()?.gameplay;
    if (gameplay === null || gameplay === undefined) return;
    void this.store.react({
      expectedRevision: gameplay.stateRevision,
      reactionId,
      optionId,
    });
  }

  protected cellLabel(cell: BoardCell, actorId: string): string {
    const coordinate = `Cell ${cell.x}, ${cell.y}`;
    if (cell.entity === null) return `${coordinate}, empty`;
    const actor = cell.entity.id === actorId ? ', current actor' : '';
    const target = cell.targetable ? ', available target' : '';
    return `${coordinate}, ${cell.entity.label}, ${cell.entity.teamId}, vitality ${cell.entity.vitality}${actor}${target}`;
  }

  protected isOpposingTeam(
    entity: GameplayEntityView,
    actorId: string,
  ): boolean {
    const actor = this.store
      .view()
      ?.gameplay?.entities.find((candidate) => candidate.id === actorId);
    return actor !== undefined && actor.teamId !== entity.teamId;
  }

  protected moveGridFocus(event: KeyboardEvent, index: number): void {
    const width = this.boardWidth();
    const offsets: Readonly<Record<string, number>> = {
      ArrowLeft: -1,
      ArrowRight: 1,
      ArrowUp: -width,
      ArrowDown: width,
    };
    const offset = offsets[event.key];
    if (offset === undefined) return;
    const nextIndex = index + offset;
    if (nextIndex < 0 || nextIndex >= this.gridCells().length) return;
    if (event.key === 'ArrowLeft' && index % width === 0) return;
    if (event.key === 'ArrowRight' && index % width === width - 1) return;
    event.preventDefault();
    this.gridCells()[nextIndex]?.nativeElement.focus();
  }
}

function recentRulesetRootIndex(itemId: string): number | null {
  const prefix = 'recent-ruleset-root-';
  if (!itemId.startsWith(prefix)) return null;
  const index = Number(itemId.slice(prefix.length));
  return Number.isSafeInteger(index) && index >= 0 ? index : null;
}

function formInteger(value: string): number {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed >= 0 ? parsed : 0;
}

function formSignedInteger(value: string): number {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) ? parsed : 0;
}
