import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  input,
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
  EncounterCellCapabilityDto,
  EncounterInitialCapabilityDto,
  EncounterParticipantSetupDto,
  EncounterRandomSourceDto,
  EncounterSetupRequestDto,
  RulesetDiagnosticDto,
} from '@asha-rulebench/protocol';
import { decodeEncounterSetupDocument } from '@asha-rulebench/protocol';
import { browserTextFileInput } from '@asha-rulebench/platform';
import { createBrowserRulesetWorkspaceStore } from '@asha-rulebench/store';

type DialogName = 'ruleset' | 'encounter' | 'artifact' | 'replay' | null;

interface BoardCell {
  readonly x: number;
  readonly y: number;
  readonly entity: GameplayEntityView | null;
  readonly targetable: boolean;
  readonly selection: AuthorityOptionSelection | null;
}

export type AuthorityOptionKind = 'participant' | 'cell' | 'area';

export interface AuthorityOptionSelection {
  readonly kind: AuthorityOptionKind;
  readonly id: string;
}

export function toggleAuthorityOption(
  current: readonly AuthorityOptionSelection[],
  selection: AuthorityOptionSelection,
  maximumTargets: number,
): readonly AuthorityOptionSelection[] {
  const existing = current.findIndex(
    (candidate) =>
      candidate.kind === selection.kind && candidate.id === selection.id,
  );
  if (existing >= 0) {
    return current.filter((_candidate, index) => index !== existing);
  }
  if (maximumTargets <= 0) return current;
  if (current.length >= maximumTargets) {
    return maximumTargets === 1 ? [selection] : current;
  }
  return [...current, selection];
}

export function authorityTargetIds(
  selections: readonly AuthorityOptionSelection[],
): readonly string[] {
  return selections.map((selection) => selection.id);
}

@Component({
  selector: 'arb-setup-diagnostics',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '[attr.id]': 'diagnostics().length === 0 ? null : messageId()',
    '[attr.hidden]': 'diagnostics().length === 0',
    '[class.diagnostic]': 'diagnostics().length > 0',
    '[class.field-diagnostic]': 'diagnostics().length > 0',
  },
  template: `
    @for (diagnostic of diagnostics(); track $index) {
      <span>
        @if (showPath()) {
          {{ diagnostic.path }} ·
        }
        {{ diagnostic.message }}
      </span>
    }
  `,
})
class SetupDiagnosticsComponent {
  public readonly diagnostics =
    input.required<readonly RulesetDiagnosticDto[]>();
  public readonly messageId = input.required<string>();
  public readonly showPath = input(false);
}

@Component({
  selector: 'arb-rulebench-workspace-feature',
  imports: [
    ApplicationDialogComponent,
    ApplicationMenubarComponent,
    SetupDiagnosticsComponent,
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

      .capability-editor {
        border: 1px solid var(--arb-border);
        padding: 0.65rem;
      }

      .field-diagnostic {
        display: block;
        font-size: 0.85rem;
        margin-top: 0.35rem;
      }

      [aria-invalid='true'] {
        border-color: var(--arb-warning);
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
                        [class.targeted]="isSelectionSelected(cell.selection)"
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
            #turnPanel
            class="turn-panel"
            [panelNumber]="3"
            panelTitle="Turn status"
          >
            <div class="panel-body" aria-live="polite">
              <div class="status-card">
                <p class="section-label">{{ view.statusLabel }}</p>
                @if (view.gameplay; as gameplay) {
                  @if (gameplay.outcome.status === 'completed') {
                    <strong>Encounter complete</strong>
                    <span
                      >Winning team{{
                        gameplay.outcome.winningTeamIds.length === 1 ? '' : 's'
                      }}:
                      {{
                        gameplay.outcome.winningTeamIds.length === 0
                          ? 'none'
                          : gameplay.outcome.winningTeamIds.join(', ')
                      }}</span
                    >
                    <span
                      >Final authority revision
                      {{ gameplay.stateRevision }}</span
                    >
                    <span class="muted"
                      >Start a new encounter from the Session menu to continue
                      experimenting.</span
                    >
                  } @else {
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
                        ? 'Choose an authority action or turn control.'
                        : 'A reaction must be resolved before play continues.'
                    }}</span>
                  }
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
                  @if (gameplay.outcome.status === 'completed') {
                    <div class="empty-state" tabindex="-1">
                      <strong>Encounter complete</strong>
                      <p>
                        Winning team{{
                          gameplay.outcome.winningTeamIds.length === 1
                            ? ''
                            : 's'
                        }}:
                        {{
                          gameplay.outcome.winningTeamIds.length === 0
                            ? 'none'
                            : gameplay.outcome.winningTeamIds.join(', ')
                        }}
                      </p>
                    </div>
                  } @else {
                    <ul class="action-list" aria-label="Available actions">
                      @for (action of gameplay.actions; track action.id) {
                        <li>
                          <button
                            class="action-choice"
                            type="button"
                            [attr.aria-pressed]="
                              selectedActionId() === action.id
                            "
                            [disabled]="store.busy() || !action.available"
                            (click)="selectAction(action)"
                          >
                            <strong>{{ action.name }}</strong>
                            <code>{{ action.id }}</code>
                            <span
                              >{{ actionOptionCount(action) }} available
                              option{{
                                actionOptionCount(action) === 1 ? '' : 's'
                              }}</span
                            >
                            <span class="muted"
                              >Up to {{ action.maximumTargets }} target{{
                                action.maximumTargets === 1 ? '' : 's'
                              }}</span
                            >
                            @if (action.unavailable) {
                              <span class="muted">{{
                                action.unavailable
                              }}</span>
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
                                isOptionSelected('participant', candidate)
                              "
                              [disabled]="
                                optionDisabled(
                                  'participant',
                                  candidate,
                                  action.maximumTargets
                                )
                              "
                              (click)="
                                toggleOption(
                                  'participant',
                                  candidate,
                                  action.maximumTargets
                                )
                              "
                            >
                              Participant {{ candidate }}
                            </button>
                          }
                          @for (candidate of action.cellIds; track candidate) {
                            <button
                              class="secondary"
                              type="button"
                              [attr.aria-pressed]="
                                isOptionSelected('cell', candidate)
                              "
                              [disabled]="
                                optionDisabled(
                                  'cell',
                                  candidate,
                                  action.maximumTargets
                                )
                              "
                              (click)="
                                toggleOption(
                                  'cell',
                                  candidate,
                                  action.maximumTargets
                                )
                              "
                            >
                              Cell {{ candidate }}
                            </button>
                          }
                          @for (candidate of action.areaIds; track candidate) {
                            <button
                              class="secondary"
                              type="button"
                              [attr.aria-pressed]="
                                isOptionSelected('area', candidate)
                              "
                              [disabled]="
                                optionDisabled(
                                  'area',
                                  candidate,
                                  action.maximumTargets
                                )
                              "
                              (click)="
                                toggleOption(
                                  'area',
                                  candidate,
                                  action.maximumTargets
                                )
                              "
                            >
                              Area {{ candidate }}
                            </button>
                          }
                        </div>
                        <p class="muted">
                          Rolls happen automatically after you act. Rust
                          requests and consumes the exact dice for the branch it
                          executes.
                        </p>
                        <button
                          type="button"
                          [disabled]="
                            store.busy() ||
                            (action.maximumTargets > 0 &&
                              selectedOptions().length === 0)
                          "
                          (click)="executeAction()"
                        >
                          Use {{ action.name
                          }}{{
                            selectedOptions().length === 0
                              ? ''
                              : ' with ' + selectedOptionSummary()
                          }}
                        </button>
                      </div>
                    } @else {
                      <p class="muted">
                        Choose an action to reveal its authority-provided
                        targets.
                      </p>
                    }
                    <ul class="action-list" aria-label="Turn controls">
                      @for (control of gameplay.controls; track control.kind) {
                        <li>
                          <button
                            class="action-choice secondary"
                            type="button"
                            [disabled]="store.busy() || !control.available"
                            (click)="executeTurnControl(control.kind)"
                          >
                            <strong>{{ control.label }}</strong>
                            <code>{{ control.kind }}</code>
                            @if (control.unavailable) {
                              <span class="muted">{{
                                control.unavailable
                              }}</span>
                            }
                          </button>
                        </li>
                      }
                    </ul>
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
          <div>
            <p class="section-label">Setup document</p>
            <input
              #setupFileInput
              class="setup-input"
              type="file"
              accept="application/json,.json"
              [disabled]="store.busy()"
              (change)="
                loadSetupDocument(setupFileInput.files);
                setupFileInput.value = ''
              "
            />
            <p class="muted">
              {{
                setupDocumentName() === null
                  ? 'Choose an explicit JSON setup document, or author one below.'
                  : 'Loaded ' + setupDocumentName()
              }}
            </p>
            @if (setupImportError(); as importError) {
              <div class="diagnostic" role="alert">
                <strong>Setup document was not loaded</strong>
                <span>{{ importError }}</span>
              </div>
            }
          </div>

          <div
            #setupControl
            tabindex="-1"
            data-setup-path="$.artifactId"
            [attr.aria-invalid]="setupHasError('$.artifactId')"
            [attr.aria-describedby]="setupDescribedBy('$.artifactId')"
          >
            <p class="section-label">Artifact binding</p>
            <code>{{ setup.artifactId }}</code>
            <arb-setup-diagnostics
              [diagnostics]="setupDiagnosticsFor('$.artifactId')"
              [messageId]="setupDiagnosticId('$.artifactId')"
            />
          </div>
          <div class="setup-grid">
            <label>
              <span class="section-label">Board width</span>
              <input
                #setupControl
                #boardWidthInput
                class="setup-input"
                type="number"
                min="1"
                max="1024"
                data-setup-path="$.board.width"
                [attr.aria-invalid]="setupHasError('$.board.width')"
                [attr.aria-describedby]="setupDescribedBy('$.board.width')"
                [value]="setup.board.width"
                (input)="updateBoardExtent('width', boardWidthInput.value)"
              />
              <arb-setup-diagnostics
                [diagnostics]="setupDiagnosticsFor('$.board.width')"
                [messageId]="setupDiagnosticId('$.board.width')"
              />
            </label>
            <label>
              <span class="section-label">Board height</span>
              <input
                #setupControl
                #boardHeightInput
                class="setup-input"
                type="number"
                min="1"
                max="1024"
                data-setup-path="$.board.height"
                [attr.aria-invalid]="setupHasError('$.board.height')"
                [attr.aria-describedby]="setupDescribedBy('$.board.height')"
                [value]="setup.board.height"
                (input)="updateBoardExtent('height', boardHeightInput.value)"
              />
              <arb-setup-diagnostics
                [diagnostics]="setupDiagnosticsFor('$.board.height')"
                [messageId]="setupDiagnosticId('$.board.height')"
              />
            </label>
            <label>
              <span class="section-label">Round</span>
              <input
                #setupControl
                #roundInput
                class="setup-input"
                type="number"
                min="1"
                max="4294967295"
                data-setup-path="$.turn.round"
                [attr.aria-invalid]="setupHasError('$.turn.round')"
                [attr.aria-describedby]="setupDescribedBy('$.turn.round')"
                [value]="setup.turn.round"
                (input)="updateTurnCounter('round', roundInput.value)"
              />
              <arb-setup-diagnostics
                [diagnostics]="setupDiagnosticsFor('$.turn.round')"
                [messageId]="setupDiagnosticId('$.turn.round')"
              />
            </label>
            <label>
              <span class="section-label">Turn</span>
              <input
                #setupControl
                #turnInput
                class="setup-input"
                type="number"
                min="1"
                max="4294967295"
                data-setup-path="$.turn.turn"
                [attr.aria-invalid]="setupHasError('$.turn.turn')"
                [attr.aria-describedby]="setupDescribedBy('$.turn.turn')"
                [value]="setup.turn.turn"
                (input)="updateTurnCounter('turn', turnInput.value)"
              />
              <arb-setup-diagnostics
                [diagnostics]="setupDiagnosticsFor('$.turn.turn')"
                [messageId]="setupDiagnosticId('$.turn.turn')"
              />
            </label>
          </div>

          <label>
            <span class="section-label">Automatic random source</span>
            <select
              #setupControl
              #randomSourceSelect
              class="setup-select"
              data-setup-path="$.randomSource"
              [attr.aria-invalid]="setupHasError('$.randomSource')"
              [attr.aria-describedby]="setupDescribedBy('$.randomSource')"
              [value]="randomSourceKey(setup.randomSource)"
              (change)="selectRandomSource(randomSourceSelect.value)"
            >
              @for (
                source of supportedRandomSources();
                track randomSourceKey(source)
              ) {
                <option [value]="randomSourceKey(source)">
                  {{ randomSourceLabel(source) }}
                </option>
              }
            </select>
            <arb-setup-diagnostics
              [diagnostics]="setupDiagnosticsFor('$.randomSource')"
              [messageId]="setupDiagnosticId('$.randomSource')"
            />
          </label>

          <div class="button-row">
            <button
              #setupControl
              class="secondary"
              type="button"
              data-setup-path="$.participants"
              [attr.aria-invalid]="setupHasExactError('$.participants')"
              [attr.aria-describedby]="setupExactDescribedBy('$.participants')"
              (click)="addParticipant()"
            >
              Add participant
            </button>
            <button
              #setupControl
              class="secondary"
              type="button"
              data-setup-path="$.board.cells"
              [attr.aria-invalid]="setupHasExactError('$.board.cells')"
              [attr.aria-describedby]="setupExactDescribedBy('$.board.cells')"
              (click)="addTerrainCell()"
            >
              Add terrain cell
            </button>
          </div>
          <arb-setup-diagnostics
            [diagnostics]="setupExactDiagnosticsFor('$.participants')"
            [messageId]="setupDiagnosticId('$.participants')"
          />
          <arb-setup-diagnostics
            [diagnostics]="setupExactDiagnosticsFor('$.board.cells')"
            [messageId]="setupDiagnosticId('$.board.cells')"
          />

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
                  [disabled]="
                    participantIndex === setup.participants.length - 1
                  "
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
                    #setupControl
                    #participantIdInput
                    class="setup-input"
                    [attr.data-setup-path]="
                      participantPath(participantIndex, 'id')
                    "
                    [attr.aria-invalid]="
                      setupHasError(participantPath(participantIndex, 'id'))
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(participantPath(participantIndex, 'id'))
                    "
                    [value]="participant.id"
                    (input)="
                      updateParticipantText(
                        participantIndex,
                        'id',
                        participantIdInput.value
                      )
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        participantPath(participantIndex, 'id')
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(participantPath(participantIndex, 'id'))
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Label</span>
                  <input
                    #setupControl
                    #participantLabelInput
                    class="setup-input"
                    [attr.data-setup-path]="
                      participantPath(participantIndex, 'label')
                    "
                    [attr.aria-invalid]="
                      setupHasError(participantPath(participantIndex, 'label'))
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(
                        participantPath(participantIndex, 'label')
                      )
                    "
                    [value]="participant.label"
                    (input)="
                      updateParticipantText(
                        participantIndex,
                        'label',
                        participantLabelInput.value
                      )
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        participantPath(participantIndex, 'label')
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(
                        participantPath(participantIndex, 'label')
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Team ID</span>
                  <input
                    #setupControl
                    #participantTeamInput
                    class="setup-input"
                    [attr.data-setup-path]="
                      participantPath(participantIndex, 'teamId')
                    "
                    [attr.aria-invalid]="
                      setupHasError(participantPath(participantIndex, 'teamId'))
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(
                        participantPath(participantIndex, 'teamId')
                      )
                    "
                    [value]="participant.teamId"
                    (input)="
                      updateParticipantText(
                        participantIndex,
                        'teamId',
                        participantTeamInput.value
                      )
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        participantPath(participantIndex, 'teamId')
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(
                        participantPath(participantIndex, 'teamId')
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Position X</span>
                  <input
                    #setupControl
                    #participantXInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [attr.data-setup-path]="
                      participantPath(participantIndex, 'position.x')
                    "
                    [attr.aria-invalid]="
                      setupHasError(
                        participantPath(participantIndex, 'position.x')
                      )
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(
                        participantPath(participantIndex, 'position.x')
                      )
                    "
                    [value]="participant.position.x"
                    (input)="
                      updateParticipantPosition(
                        participantIndex,
                        'x',
                        participantXInput.value
                      )
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        participantPath(participantIndex, 'position.x')
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(
                        participantPath(participantIndex, 'position.x')
                      )
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Position Y</span>
                  <input
                    #setupControl
                    #participantYInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [attr.data-setup-path]="
                      participantPath(participantIndex, 'position.y')
                    "
                    [attr.aria-invalid]="
                      setupHasError(
                        participantPath(participantIndex, 'position.y')
                      )
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(
                        participantPath(participantIndex, 'position.y')
                      )
                    "
                    [value]="participant.position.y"
                    (input)="
                      updateParticipantPosition(
                        participantIndex,
                        'y',
                        participantYInput.value
                      )
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        participantPath(participantIndex, 'position.y')
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(
                        participantPath(participantIndex, 'position.y')
                      )
                    "
                  />
                </label>
              </div>
              <div class="button-row" aria-label="Add participant capability">
                @for (owner of participantCapabilityOwners; track owner) {
                  <button
                    class="secondary"
                    type="button"
                    (click)="addParticipantCapability(participantIndex, owner)"
                  >
                    Add {{ owner }}
                  </button>
                }
              </div>
              @for (
                capability of participant.capabilities;
                track $index;
                let capabilityIndex = $index
              ) {
                <fieldset
                  class="definition-choices capability-editor"
                  [attr.data-setup-path]="
                    capabilityPath(participantIndex, capabilityIndex)
                  "
                  [attr.aria-invalid]="
                    setupHasError(
                      capabilityPath(participantIndex, capabilityIndex)
                    )
                  "
                  [attr.aria-describedby]="
                    setupDescribedBy(
                      capabilityPath(participantIndex, capabilityIndex)
                    )
                  "
                >
                  <legend class="section-label">
                    {{ capability.owner }} capability {{ capabilityIndex + 1 }}
                  </legend>
                  <div class="setup-grid">
                    @if (capability.owner !== 'vitality') {
                      <label>
                        <span class="section-label">ID</span>
                        <input
                          #setupControl
                          #capabilityIdInput
                          class="setup-input"
                          [attr.data-setup-path]="
                            capabilityFieldPath(
                              participantIndex,
                              capabilityIndex,
                              'id'
                            )
                          "
                          [attr.aria-invalid]="
                            setupHasError(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'id'
                              )
                            )
                          "
                          [attr.aria-describedby]="
                            setupDescribedBy(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'id'
                              )
                            )
                          "
                          [value]="capability.id"
                          (input)="
                            updateParticipantCapabilityText(
                              participantIndex,
                              capabilityIndex,
                              'id',
                              capabilityIdInput.value
                            )
                          "
                        />
                        <arb-setup-diagnostics
                          [diagnostics]="
                            setupDiagnosticsFor(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'id'
                              )
                            )
                          "
                          [messageId]="
                            setupDiagnosticId(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'id'
                              )
                            )
                          "
                        />
                      </label>
                    }
                    @if (capability.owner === 'modifier') {
                      <label>
                        <span class="section-label">Stacking group</span>
                        <input
                          #setupControl
                          #stackingGroupInput
                          class="setup-input"
                          [attr.data-setup-path]="
                            capabilityFieldPath(
                              participantIndex,
                              capabilityIndex,
                              'stackingGroup'
                            )
                          "
                          [attr.aria-invalid]="
                            setupHasError(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'stackingGroup'
                              )
                            )
                          "
                          [attr.aria-describedby]="
                            setupDescribedBy(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'stackingGroup'
                              )
                            )
                          "
                          [value]="capability.stackingGroup"
                          (input)="
                            updateParticipantCapabilityText(
                              participantIndex,
                              capabilityIndex,
                              'stackingGroup',
                              stackingGroupInput.value
                            )
                          "
                        />
                        <arb-setup-diagnostics
                          [diagnostics]="
                            setupDiagnosticsFor(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'stackingGroup'
                              )
                            )
                          "
                          [messageId]="
                            setupDiagnosticId(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'stackingGroup'
                              )
                            )
                          "
                        />
                      </label>
                    }
                    @if (
                      capability.owner === 'vitality' ||
                      capability.owner === 'resource'
                    ) {
                      <label>
                        <span class="section-label">Current</span>
                        <input
                          #setupControl
                          #capabilityCurrentInput
                          class="setup-input"
                          type="number"
                          min="0"
                          [attr.data-setup-path]="
                            capabilityFieldPath(
                              participantIndex,
                              capabilityIndex,
                              'value.current'
                            )
                          "
                          [attr.aria-invalid]="
                            setupHasError(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.current'
                              )
                            )
                          "
                          [attr.aria-describedby]="
                            setupDescribedBy(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.current'
                              )
                            )
                          "
                          [value]="capability.value.current"
                          (input)="
                            updateParticipantCapabilityNumber(
                              participantIndex,
                              capabilityIndex,
                              'current',
                              capabilityCurrentInput.value
                            )
                          "
                        />
                        <arb-setup-diagnostics
                          [diagnostics]="
                            setupDiagnosticsFor(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.current'
                              )
                            )
                          "
                          [messageId]="
                            setupDiagnosticId(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.current'
                              )
                            )
                          "
                        />
                      </label>
                      <label>
                        <span class="section-label">Maximum</span>
                        <input
                          #setupControl
                          #capabilityMaximumInput
                          class="setup-input"
                          type="number"
                          min="0"
                          [attr.data-setup-path]="
                            capabilityFieldPath(
                              participantIndex,
                              capabilityIndex,
                              'value.max'
                            )
                          "
                          [attr.aria-invalid]="
                            setupHasError(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.max'
                              )
                            )
                          "
                          [attr.aria-describedby]="
                            setupDescribedBy(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.max'
                              )
                            )
                          "
                          [value]="capability.value.max"
                          (input)="
                            updateParticipantCapabilityNumber(
                              participantIndex,
                              capabilityIndex,
                              'max',
                              capabilityMaximumInput.value
                            )
                          "
                        />
                        <arb-setup-diagnostics
                          [diagnostics]="
                            setupDiagnosticsFor(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.max'
                              )
                            )
                          "
                          [messageId]="
                            setupDiagnosticId(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value.max'
                              )
                            )
                          "
                        />
                      </label>
                    } @else {
                      <label>
                        <span class="section-label">Value</span>
                        <input
                          #setupControl
                          #capabilityValueInput
                          class="setup-input"
                          type="number"
                          [attr.data-setup-path]="
                            capabilityFieldPath(
                              participantIndex,
                              capabilityIndex,
                              'value'
                            )
                          "
                          [attr.aria-invalid]="
                            setupHasError(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value'
                              )
                            )
                          "
                          [attr.aria-describedby]="
                            setupDescribedBy(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value'
                              )
                            )
                          "
                          [value]="capability.value"
                          (input)="
                            updateParticipantCapabilityNumber(
                              participantIndex,
                              capabilityIndex,
                              'value',
                              capabilityValueInput.value
                            )
                          "
                        />
                        <arb-setup-diagnostics
                          [diagnostics]="
                            setupDiagnosticsFor(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value'
                              )
                            )
                          "
                          [messageId]="
                            setupDiagnosticId(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'value'
                              )
                            )
                          "
                        />
                      </label>
                    }
                    @if (capability.owner === 'modifier') {
                      <label>
                        <span class="section-label">Remaining turns</span>
                        <input
                          #setupControl
                          #remainingTurnsInput
                          class="setup-input"
                          type="number"
                          min="1"
                          max="1000"
                          [attr.data-setup-path]="
                            capabilityFieldPath(
                              participantIndex,
                              capabilityIndex,
                              'remainingTurns'
                            )
                          "
                          [attr.aria-invalid]="
                            setupHasError(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'remainingTurns'
                              )
                            )
                          "
                          [attr.aria-describedby]="
                            setupDescribedBy(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'remainingTurns'
                              )
                            )
                          "
                          [value]="capability.remainingTurns"
                          (input)="
                            updateParticipantCapabilityNumber(
                              participantIndex,
                              capabilityIndex,
                              'remainingTurns',
                              remainingTurnsInput.value
                            )
                          "
                        />
                        <arb-setup-diagnostics
                          [diagnostics]="
                            setupDiagnosticsFor(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'remainingTurns'
                              )
                            )
                          "
                          [messageId]="
                            setupDiagnosticId(
                              capabilityFieldPath(
                                participantIndex,
                                capabilityIndex,
                                'remainingTurns'
                              )
                            )
                          "
                        />
                      </label>
                    }
                  </div>
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        capabilityPath(participantIndex, capabilityIndex)
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(
                        capabilityPath(participantIndex, capabilityIndex)
                      )
                    "
                    [showPath]="true"
                  />
                  <button
                    class="secondary"
                    type="button"
                    (click)="
                      removeParticipantCapability(
                        participantIndex,
                        capabilityIndex
                      )
                    "
                  >
                    Remove capability
                  </button>
                </fieldset>
              }
              <fieldset
                class="definition-choices"
                [attr.data-setup-path]="
                  participantPath(participantIndex, 'definitionIds')
                "
                [attr.aria-invalid]="
                  setupHasError(
                    participantPath(participantIndex, 'definitionIds')
                  )
                "
                [attr.aria-describedby]="
                  setupDescribedBy(
                    participantPath(participantIndex, 'definitionIds')
                  )
                "
              >
                <legend class="section-label">Owned action definitions</legend>
                @for (definition of actionDefinitions(); track definition.id) {
                  <label class="definition-choice">
                    <input
                      #setupControl
                      type="checkbox"
                      [attr.data-setup-path]="
                        participantPath(participantIndex, 'definitionIds')
                      "
                      [attr.aria-invalid]="
                        setupHasError(
                          participantPath(participantIndex, 'definitionIds')
                        )
                      "
                      [attr.aria-describedby]="
                        setupDescribedBy(
                          participantPath(participantIndex, 'definitionIds')
                        )
                      "
                      [checked]="
                        participant.definitionIds.includes(definition.id)
                      "
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
                <arb-setup-diagnostics
                  [diagnostics]="
                    setupDiagnosticsFor(
                      participantPath(participantIndex, 'definitionIds')
                    )
                  "
                  [messageId]="
                    setupDiagnosticId(
                      participantPath(participantIndex, 'definitionIds')
                    )
                  "
                />
              </fieldset>
            </section>
          } @empty {
            <p class="muted">
              Add every participant explicitly. Setup contains no hidden roster
              or action script.
            </p>
          }

          @for (
            cell of setup.board.cells;
            track $index;
            let cellIndex = $index
          ) {
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
                    #setupControl
                    #cellIdInput
                    class="setup-input"
                    [attr.data-setup-path]="cellPath(cellIndex, 'id')"
                    [attr.aria-invalid]="
                      setupHasError(cellPath(cellIndex, 'id'))
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(cellPath(cellIndex, 'id'))
                    "
                    [value]="cell.id"
                    (input)="updateCellId(cellIndex, cellIdInput.value)"
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(cellPath(cellIndex, 'id'))
                    "
                    [messageId]="setupDiagnosticId(cellPath(cellIndex, 'id'))"
                  />
                </label>
                <label>
                  <span class="section-label">Position X</span>
                  <input
                    #setupControl
                    #cellXInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [attr.data-setup-path]="cellPath(cellIndex, 'position.x')"
                    [attr.aria-invalid]="
                      setupHasError(cellPath(cellIndex, 'position.x'))
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(cellPath(cellIndex, 'position.x'))
                    "
                    [value]="cell.position.x"
                    (input)="
                      updateCellPosition(cellIndex, 'x', cellXInput.value)
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(cellPath(cellIndex, 'position.x'))
                    "
                    [messageId]="
                      setupDiagnosticId(cellPath(cellIndex, 'position.x'))
                    "
                  />
                </label>
                <label>
                  <span class="section-label">Position Y</span>
                  <input
                    #setupControl
                    #cellYInput
                    class="setup-input"
                    type="number"
                    min="0"
                    [attr.data-setup-path]="cellPath(cellIndex, 'position.y')"
                    [attr.aria-invalid]="
                      setupHasError(cellPath(cellIndex, 'position.y'))
                    "
                    [attr.aria-describedby]="
                      setupDescribedBy(cellPath(cellIndex, 'position.y'))
                    "
                    [value]="cell.position.y"
                    (input)="
                      updateCellPosition(cellIndex, 'y', cellYInput.value)
                    "
                  />
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(cellPath(cellIndex, 'position.y'))
                    "
                    [messageId]="
                      setupDiagnosticId(cellPath(cellIndex, 'position.y'))
                    "
                  />
                </label>
              </div>
              <div class="button-row" aria-label="Add cell capability">
                @for (kind of cellCapabilityKinds; track kind) {
                  <button
                    class="secondary"
                    type="button"
                    (click)="addCellCapability(cellIndex, kind)"
                  >
                    Add {{ kind }}
                  </button>
                }
              </div>
              @for (
                capability of cell.capabilities;
                track $index;
                let capabilityIndex = $index
              ) {
                <fieldset
                  class="definition-choices capability-editor"
                  [attr.data-setup-path]="
                    cellCapabilityPath(cellIndex, capabilityIndex)
                  "
                  [attr.aria-invalid]="
                    setupHasError(
                      cellCapabilityPath(cellIndex, capabilityIndex)
                    )
                  "
                  [attr.aria-describedby]="
                    setupDescribedBy(
                      cellCapabilityPath(cellIndex, capabilityIndex)
                    )
                  "
                >
                  <legend class="section-label">
                    {{ capability.value.kind }} capability
                    {{ capabilityIndex + 1 }}
                  </legend>
                  <div class="setup-grid">
                    <label>
                      <span class="section-label">Capability ID</span>
                      <input
                        #setupControl
                        #cellCapabilityIdInput
                        class="setup-input"
                        [attr.data-setup-path]="
                          cellCapabilityFieldPath(
                            cellIndex,
                            capabilityIndex,
                            'id'
                          )
                        "
                        [attr.aria-invalid]="
                          setupHasError(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'id'
                            )
                          )
                        "
                        [attr.aria-describedby]="
                          setupDescribedBy(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'id'
                            )
                          )
                        "
                        [value]="capability.id"
                        (input)="
                          updateCellCapabilityText(
                            cellIndex,
                            capabilityIndex,
                            'id',
                            cellCapabilityIdInput.value
                          )
                        "
                      />
                      <arb-setup-diagnostics
                        [diagnostics]="
                          setupDiagnosticsFor(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'id'
                            )
                          )
                        "
                        [messageId]="
                          setupDiagnosticId(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'id'
                            )
                          )
                        "
                      />
                    </label>
                    <label>
                      <span class="section-label">Version</span>
                      <input
                        #setupControl
                        #cellCapabilityVersionInput
                        class="setup-input"
                        type="number"
                        min="1"
                        [attr.data-setup-path]="
                          cellCapabilityFieldPath(
                            cellIndex,
                            capabilityIndex,
                            'version'
                          )
                        "
                        [attr.aria-invalid]="
                          setupHasError(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'version'
                            )
                          )
                        "
                        [attr.aria-describedby]="
                          setupDescribedBy(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'version'
                            )
                          )
                        "
                        [value]="capability.version"
                        (input)="
                          updateCellCapabilityNumber(
                            cellIndex,
                            capabilityIndex,
                            'version',
                            cellCapabilityVersionInput.value
                          )
                        "
                      />
                      <arb-setup-diagnostics
                        [diagnostics]="
                          setupDiagnosticsFor(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'version'
                            )
                          )
                        "
                        [messageId]="
                          setupDiagnosticId(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'version'
                            )
                          )
                        "
                      />
                    </label>
                    <label>
                      <span class="section-label"
                        >Definition ID (optional)</span
                      >
                      <input
                        #setupControl
                        #cellDefinitionIdInput
                        class="setup-input"
                        [attr.data-setup-path]="
                          cellCapabilityFieldPath(
                            cellIndex,
                            capabilityIndex,
                            'definitionId'
                          )
                        "
                        [attr.aria-invalid]="
                          setupHasError(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'definitionId'
                            )
                          )
                        "
                        [attr.aria-describedby]="
                          setupDescribedBy(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'definitionId'
                            )
                          )
                        "
                        [value]="capability.definitionId ?? ''"
                        (input)="
                          updateCellCapabilityText(
                            cellIndex,
                            capabilityIndex,
                            'definitionId',
                            cellDefinitionIdInput.value
                          )
                        "
                      />
                      <arb-setup-diagnostics
                        [diagnostics]="
                          setupDiagnosticsFor(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'definitionId'
                            )
                          )
                        "
                        [messageId]="
                          setupDiagnosticId(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'definitionId'
                            )
                          )
                        "
                      />
                    </label>
                    @switch (capability.value.kind) {
                      @case ('traversal') {
                        <label>
                          <span class="section-label">Passable</span>
                          <select
                            #setupControl
                            #cellPassableSelect
                            class="setup-select"
                            [attr.data-setup-path]="
                              cellCapabilityFieldPath(
                                cellIndex,
                                capabilityIndex,
                                'value.passable'
                              )
                            "
                            [attr.aria-invalid]="
                              setupHasError(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.passable'
                                )
                              )
                            "
                            [attr.aria-describedby]="
                              setupDescribedBy(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.passable'
                                )
                              )
                            "
                            [value]="
                              capability.value.passable ? 'true' : 'false'
                            "
                            (change)="
                              updateCellCapabilityText(
                                cellIndex,
                                capabilityIndex,
                                'booleanValue',
                                cellPassableSelect.value
                              )
                            "
                          >
                            <option value="true">Yes</option>
                            <option value="false">No</option>
                          </select>
                          <arb-setup-diagnostics
                            [diagnostics]="
                              setupDiagnosticsFor(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.passable'
                                )
                              )
                            "
                            [messageId]="
                              setupDiagnosticId(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.passable'
                                )
                              )
                            "
                          />
                        </label>
                        <label>
                          <span class="section-label">Movement cost</span>
                          <input
                            #setupControl
                            #movementCostInput
                            class="setup-input"
                            type="number"
                            min="1"
                            [attr.data-setup-path]="
                              cellCapabilityFieldPath(
                                cellIndex,
                                capabilityIndex,
                                'value.movementCost'
                              )
                            "
                            [attr.aria-invalid]="
                              setupHasError(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.movementCost'
                                )
                              )
                            "
                            [attr.aria-describedby]="
                              setupDescribedBy(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.movementCost'
                                )
                              )
                            "
                            [value]="capability.value.movementCost"
                            (input)="
                              updateCellCapabilityNumber(
                                cellIndex,
                                capabilityIndex,
                                'value',
                                movementCostInput.value
                              )
                            "
                          />
                          <arb-setup-diagnostics
                            [diagnostics]="
                              setupDiagnosticsFor(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.movementCost'
                                )
                              )
                            "
                            [messageId]="
                              setupDiagnosticId(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.movementCost'
                                )
                              )
                            "
                          />
                        </label>
                      }
                      @case ('flag') {
                        <label>
                          <span class="section-label">Value</span>
                          <select
                            #setupControl
                            #cellFlagSelect
                            class="setup-select"
                            [attr.data-setup-path]="
                              cellCapabilityFieldPath(
                                cellIndex,
                                capabilityIndex,
                                'value.value'
                              )
                            "
                            [attr.aria-invalid]="
                              setupHasError(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                            [attr.aria-describedby]="
                              setupDescribedBy(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                            [value]="capability.value.value ? 'true' : 'false'"
                            (change)="
                              updateCellCapabilityText(
                                cellIndex,
                                capabilityIndex,
                                'booleanValue',
                                cellFlagSelect.value
                              )
                            "
                          >
                            <option value="true">True</option>
                            <option value="false">False</option>
                          </select>
                          <arb-setup-diagnostics
                            [diagnostics]="
                              setupDiagnosticsFor(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                            [messageId]="
                              setupDiagnosticId(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                          />
                        </label>
                      }
                      @case ('integer') {
                        <label>
                          <span class="section-label">Value</span>
                          <input
                            #setupControl
                            #cellIntegerInput
                            class="setup-input"
                            type="number"
                            [attr.data-setup-path]="
                              cellCapabilityFieldPath(
                                cellIndex,
                                capabilityIndex,
                                'value.value'
                              )
                            "
                            [attr.aria-invalid]="
                              setupHasError(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                            [attr.aria-describedby]="
                              setupDescribedBy(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                            [value]="capability.value.value"
                            (input)="
                              updateCellCapabilityNumber(
                                cellIndex,
                                capabilityIndex,
                                'value',
                                cellIntegerInput.value
                              )
                            "
                          />
                          <arb-setup-diagnostics
                            [diagnostics]="
                              setupDiagnosticsFor(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                            [messageId]="
                              setupDiagnosticId(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.value'
                                )
                              )
                            "
                          />
                        </label>
                      }
                      @case ('identifier') {
                        <label>
                          <span class="section-label">Value ID</span>
                          <input
                            #setupControl
                            #cellValueIdInput
                            class="setup-input"
                            [attr.data-setup-path]="
                              cellCapabilityFieldPath(
                                cellIndex,
                                capabilityIndex,
                                'value.valueId'
                              )
                            "
                            [attr.aria-invalid]="
                              setupHasError(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.valueId'
                                )
                              )
                            "
                            [attr.aria-describedby]="
                              setupDescribedBy(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.valueId'
                                )
                              )
                            "
                            [value]="capability.value.valueId"
                            (input)="
                              updateCellCapabilityText(
                                cellIndex,
                                capabilityIndex,
                                'valueId',
                                cellValueIdInput.value
                              )
                            "
                          />
                          <arb-setup-diagnostics
                            [diagnostics]="
                              setupDiagnosticsFor(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.valueId'
                                )
                              )
                            "
                            [messageId]="
                              setupDiagnosticId(
                                cellCapabilityFieldPath(
                                  cellIndex,
                                  capabilityIndex,
                                  'value.valueId'
                                )
                              )
                            "
                          />
                        </label>
                      }
                    }
                  </div>
                  <arb-setup-diagnostics
                    [diagnostics]="
                      setupDiagnosticsFor(
                        cellCapabilityPath(cellIndex, capabilityIndex)
                      )
                    "
                    [messageId]="
                      setupDiagnosticId(
                        cellCapabilityPath(cellIndex, capabilityIndex)
                      )
                    "
                    [showPath]="true"
                  />
                  <button
                    class="secondary"
                    type="button"
                    (click)="removeCellCapability(cellIndex, capabilityIndex)"
                  >
                    Remove capability
                  </button>
                </fieldset>
              }
            </section>
          }

          <label for="current-actor" class="section-label"
            >Starting actor</label
          >
          <select
            #setupControl
            #currentActorSelect
            id="current-actor"
            class="setup-select"
            data-setup-path="$.turn.currentActorId"
            [attr.aria-invalid]="setupHasError('$.turn.currentActorId')"
            [attr.aria-describedby]="setupDescribedBy('$.turn.currentActorId')"
            [value]="setup.turn.currentActorId"
            (change)="setCurrentActor(currentActorSelect.value)"
          >
            @for (participant of setup.participants; track $index) {
              <option [value]="participant.id">
                {{ participant.label || participant.id }}
              </option>
            }
          </select>
          <arb-setup-diagnostics
            [diagnostics]="setupDiagnosticsFor('$.turn.currentActorId')"
            [messageId]="setupDiagnosticId('$.turn.currentActorId')"
          />
          <p class="muted">
            Initiative follows the participant order shown above. Actions,
            targets, reactions, rolls, expected events, and winners are not part
            of setup.
          </p>
          <div
            #setupControl
            tabindex="-1"
            data-setup-path="$.turn.initiativeOrder"
            [attr.aria-invalid]="setupHasError('$.turn.initiativeOrder')"
            [attr.aria-describedby]="setupDescribedBy('$.turn.initiativeOrder')"
          >
            <arb-setup-diagnostics
              [diagnostics]="setupDiagnosticsFor('$.turn.initiativeOrder')"
              [messageId]="setupDiagnosticId('$.turn.initiativeOrder')"
            />
          </div>

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
  protected readonly participantCapabilityOwners = [
    'vitality',
    'stat',
    'defense',
    'resource',
    'modifier',
  ] as const;
  protected readonly cellCapabilityKinds = [
    'traversal',
    'flag',
    'integer',
    'identifier',
  ] as const;
  protected readonly store = createBrowserRulesetWorkspaceStore();
  protected readonly openDialogName = signal<DialogName>(null);
  protected readonly selectedActionId = signal<string | null>(null);
  protected readonly selectedOptions = signal<
    readonly AuthorityOptionSelection[]
  >([]);
  protected readonly setupDraft = signal<EncounterSetupRequestDto | null>(null);
  protected readonly setupDocumentName = signal<string | null>(null);
  protected readonly setupImportError = signal<string | null>(null);

  private readonly textFileInput = browserTextFileInput();

  private readonly rulesetRootInput =
    viewChild<ElementRef<HTMLInputElement>>('rulesetRootInput');
  private readonly boardPanel =
    viewChild<WorkbenchPanelComponent>('boardPanel');
  private readonly actionPanel =
    viewChild<WorkbenchPanelComponent>('actionPanel');
  private readonly outcomePanel =
    viewChild<WorkbenchPanelComponent>('outcomePanel');
  private readonly turnPanel = viewChild<WorkbenchPanelComponent>('turnPanel');
  private readonly reactionPanel =
    viewChild<ElementRef<HTMLElement>>('reactionPanel');
  private readonly gridCells =
    viewChildren<ElementRef<HTMLButtonElement>>('gridCell');
  private readonly setupControls =
    viewChildren<ElementRef<HTMLElement>>('setupControl');

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
    const gameplay = this.store.view()?.gameplay;
    const entities = gameplay?.entities ?? [];
    const action = this.selectedAction();
    const participantIds = new Set(action?.candidateIds ?? []);
    const cellIds = new Set(action?.cellIds ?? []);
    const cells: BoardCell[] = [];
    for (let y = 0; y < this.boardHeight(); y += 1) {
      for (let x = 0; x < this.boardWidth(); x += 1) {
        const entity =
          entities.find(
            (candidate) => candidate.x === x && candidate.y === y,
          ) ?? null;
        const authoredCell = gameplay?.board.cells.find(
          (candidate) => candidate.x === x && candidate.y === y,
        );
        const selection =
          entity !== null && participantIds.has(entity.id)
            ? { kind: 'participant' as const, id: entity.id }
            : authoredCell !== undefined && cellIds.has(authoredCell.id)
              ? { kind: 'cell' as const, id: authoredCell.id }
              : null;
        cells.push({
          x,
          y,
          entity,
          targetable: selection !== null,
          selection,
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

  protected readonly supportedRandomSources = computed<
    readonly EncounterRandomSourceDto[]
  >(() => {
    const view = this.store.view();
    if (view === null) return [];
    return view.supportedRandomSources.map((source) => ({ ...source }));
  });

  protected readonly setupDiagnostics = computed(() =>
    (this.store.view()?.diagnostics ?? []).filter(
      (diagnostic) => diagnostic.stage === 'setup',
    ),
  );

  protected readonly selectedOptionSummary = computed(() =>
    this.selectedOptions()
      .map((selection) => `${selection.kind} ${selection.id}`)
      .join(', '),
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
    effect(() => {
      const outcome = this.store.view()?.gameplay?.outcome;
      const panel = this.turnPanel();
      if (outcome?.status === 'completed' && panel !== undefined) panel.focus();
    });
    effect(() => {
      const diagnostics = this.setupDiagnostics();
      if (this.openDialogName() === 'encounter' && diagnostics.length > 0) {
        this.focusFirstSetupDiagnostic();
      }
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
    this.setupDocumentName.set(null);
    this.setupImportError.set(null);
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

  protected async loadSetupDocument(files: FileList | null): Promise<void> {
    const file = files?.item(0);
    if (file === null || file === undefined) return;
    try {
      const loaded = await this.textFileInput.readText(file);
      const parsed: unknown = JSON.parse(loaded.text);
      const setup = decodeEncounterSetupDocument(parsed);
      this.setupDraft.set(setup);
      this.setupDocumentName.set(loaded.name);
      this.setupImportError.set(null);
    } catch (error: unknown) {
      this.setupDocumentName.set(null);
      this.setupImportError.set(errorMessage(error));
    }
  }

  protected randomSourceKey(source: EncounterRandomSourceDto): string {
    return `${source.policyId}@${source.policyVersion}:${source.sourceId}@${source.sourceVersion}`;
  }

  protected randomSourceLabel(source: EncounterRandomSourceDto): string {
    return `${source.policyId}@${source.policyVersion} · ${source.sourceId}@${source.sourceVersion}`;
  }

  protected selectRandomSource(key: string): void {
    const source = this.supportedRandomSources().find(
      (candidate) => this.randomSourceKey(candidate) === key,
    );
    if (source === undefined) return;
    this.updateSetup((setup) => ({ ...setup, randomSource: { ...source } }));
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
        capabilities: [{ owner: 'vitality', value: { current: 10, max: 10 } }],
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

  protected addParticipantCapability(
    participantIndex: number,
    owner: EncounterInitialCapabilityDto['owner'],
  ): void {
    const capability = initialParticipantCapability(owner);
    this.updateParticipant(participantIndex, (participant) => ({
      ...participant,
      capabilities: [...participant.capabilities, capability],
    }));
  }

  protected removeParticipantCapability(
    participantIndex: number,
    capabilityIndex: number,
  ): void {
    this.updateParticipant(participantIndex, (participant) => ({
      ...participant,
      capabilities: participant.capabilities.filter(
        (_capability, index) => index !== capabilityIndex,
      ),
    }));
  }

  protected updateParticipantCapabilityText(
    participantIndex: number,
    capabilityIndex: number,
    field: 'id' | 'stackingGroup',
    value: string,
  ): void {
    this.updateParticipantCapability(
      participantIndex,
      capabilityIndex,
      (capability) => {
        if (field === 'id' && capability.owner !== 'vitality') {
          return { ...capability, id: value };
        }
        if (field === 'stackingGroup' && capability.owner === 'modifier') {
          return { ...capability, stackingGroup: value };
        }
        return capability;
      },
    );
  }

  protected updateParticipantCapabilityNumber(
    participantIndex: number,
    capabilityIndex: number,
    field: 'current' | 'max' | 'value' | 'remainingTurns',
    value: string,
  ): void {
    this.updateParticipantCapability(
      participantIndex,
      capabilityIndex,
      (capability) => {
        if (
          (capability.owner === 'vitality' ||
            capability.owner === 'resource') &&
          (field === 'current' || field === 'max')
        ) {
          return {
            ...capability,
            value: { ...capability.value, [field]: formInteger(value) },
          };
        }
        if (
          capability.owner !== 'vitality' &&
          capability.owner !== 'resource' &&
          field === 'value'
        ) {
          return { ...capability, value: formSignedInteger(value) };
        }
        if (capability.owner === 'modifier' && field === 'remainingTurns') {
          return { ...capability, remainingTurns: formInteger(value) };
        }
        return capability;
      },
    );
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
              capabilities: [],
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

  protected addCellCapability(
    cellIndex: number,
    kind: EncounterCellCapabilityDto['value']['kind'],
  ): void {
    const capability = initialCellCapability(kind);
    this.updateCell(cellIndex, (cell) => ({
      ...cell,
      capabilities: [...cell.capabilities, capability],
    }));
  }

  protected removeCellCapability(
    cellIndex: number,
    capabilityIndex: number,
  ): void {
    this.updateCell(cellIndex, (cell) => ({
      ...cell,
      capabilities: cell.capabilities.filter(
        (_capability, index) => index !== capabilityIndex,
      ),
    }));
  }

  protected updateCellCapabilityText(
    cellIndex: number,
    capabilityIndex: number,
    field: 'id' | 'definitionId' | 'booleanValue' | 'valueId',
    value: string,
  ): void {
    this.updateCellCapability(cellIndex, capabilityIndex, (capability) => {
      if (field === 'id') return { ...capability, id: value };
      if (field === 'definitionId') {
        return { ...capability, definitionId: value.trim() || null };
      }
      if (field === 'booleanValue' && capability.value.kind === 'traversal') {
        return {
          ...capability,
          value: { ...capability.value, passable: value === 'true' },
        };
      }
      if (field === 'booleanValue' && capability.value.kind === 'flag') {
        return {
          ...capability,
          value: { ...capability.value, value: value === 'true' },
        };
      }
      if (field === 'valueId' && capability.value.kind === 'identifier') {
        return {
          ...capability,
          value: { ...capability.value, valueId: value },
        };
      }
      return capability;
    });
  }

  protected updateCellCapabilityNumber(
    cellIndex: number,
    capabilityIndex: number,
    field: 'version' | 'value',
    value: string,
  ): void {
    this.updateCellCapability(cellIndex, capabilityIndex, (capability) => {
      if (field === 'version') {
        return { ...capability, version: formInteger(value) };
      }
      if (capability.value.kind === 'traversal') {
        return {
          ...capability,
          value: { ...capability.value, movementCost: formInteger(value) },
        };
      }
      if (capability.value.kind === 'integer') {
        return {
          ...capability,
          value: { ...capability.value, value: formSignedInteger(value) },
        };
      }
      return capability;
    });
  }

  protected setCurrentActor(currentActorId: string): void {
    this.updateSetup((setup) => ({
      ...setup,
      turn: { ...setup.turn, currentActorId },
    }));
  }

  protected participantPath(index: number, suffix: string): string {
    return `$.participants[${index}].${suffix}`;
  }

  protected capabilityPath(
    participantIndex: number,
    capabilityIndex: number,
  ): string {
    return `$.participants[${participantIndex}].capabilities[${capabilityIndex}]`;
  }

  protected capabilityFieldPath(
    participantIndex: number,
    capabilityIndex: number,
    suffix: string,
  ): string {
    return `${this.capabilityPath(participantIndex, capabilityIndex)}.${suffix}`;
  }

  protected cellPath(index: number, suffix: string): string {
    return `$.board.cells[${index}].${suffix}`;
  }

  protected cellCapabilityPath(
    cellIndex: number,
    capabilityIndex: number,
  ): string {
    return `$.board.cells[${cellIndex}].capabilities[${capabilityIndex}]`;
  }

  protected cellCapabilityFieldPath(
    cellIndex: number,
    capabilityIndex: number,
    suffix: string,
  ): string {
    return `${this.cellCapabilityPath(cellIndex, capabilityIndex)}.${suffix}`;
  }

  protected setupDiagnosticsFor(path: string): readonly RulesetDiagnosticDto[] {
    return this.setupDiagnostics().filter((diagnostic) =>
      diagnosticMatchesSetupPath(diagnostic.path, path),
    );
  }

  protected setupHasError(path: string): boolean {
    return this.setupDiagnosticsFor(path).length > 0;
  }

  protected setupExactDiagnosticsFor(
    path: string,
  ): readonly RulesetDiagnosticDto[] {
    return this.setupDiagnostics().filter(
      (diagnostic) => diagnostic.path === path,
    );
  }

  protected setupHasExactError(path: string): boolean {
    return this.setupExactDiagnosticsFor(path).length > 0;
  }

  protected setupDescribedBy(path: string): string | null {
    return this.setupHasError(path) ? this.setupDiagnosticId(path) : null;
  }

  protected setupExactDescribedBy(path: string): string | null {
    return this.setupHasExactError(path) ? this.setupDiagnosticId(path) : null;
  }

  protected setupDiagnosticId(path: string): string {
    return `setup-error-${path.replaceAll(/[^a-zA-Z0-9]+/g, '-')}`;
  }

  protected startEncounter(): void {
    const setup = this.setupDraft();
    if (setup === null) return;
    void this.store.startEncounter(setup).then((started) => {
      if (started) {
        this.selectedActionId.set(null);
        this.selectedOptions.set([]);
        this.closeDialog();
        if (this.store.view()?.gameplay?.outcome.status === 'completed') {
          this.turnPanel()?.focus();
        } else {
          this.boardPanel()?.focus();
        }
      } else {
        this.focusFirstSetupDiagnostic();
      }
    });
  }

  private updateSetup(
    update: (setup: EncounterSetupRequestDto) => EncounterSetupRequestDto,
  ): void {
    this.setupDraft.update((setup) => (setup === null ? null : update(setup)));
  }

  private focusFirstSetupDiagnostic(): void {
    const diagnostic = this.setupDiagnostics()[0];
    if (diagnostic === undefined) return;
    const candidates = this.setupControls()
      .map((control) => control.nativeElement)
      .map((control) => {
        const path = control.dataset['setupPath'];
        return {
          control,
          path,
          specificity:
            path === undefined
              ? null
              : setupPathMatchSpecificity(diagnostic.path, path),
        };
      })
      .filter(
        (
          candidate,
        ): candidate is {
          readonly control: HTMLElement;
          readonly path: string;
          readonly specificity: number;
        } => candidate.path !== undefined && candidate.specificity !== null,
      )
      .sort((left, right) => {
        if (left.specificity !== right.specificity) {
          return left.specificity - right.specificity;
        }
        return right.path.length - left.path.length;
      });
    candidates[0]?.control.focus();
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

  private updateParticipantCapability(
    participantIndex: number,
    capabilityIndex: number,
    update: (
      capability: EncounterInitialCapabilityDto,
    ) => EncounterInitialCapabilityDto,
  ): void {
    this.updateParticipant(participantIndex, (participant) => ({
      ...participant,
      capabilities: participant.capabilities.map((capability, index) =>
        capabilityIndex === index ? update(capability) : capability,
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

  private updateCellCapability(
    cellIndex: number,
    capabilityIndex: number,
    update: (
      capability: EncounterCellCapabilityDto,
    ) => EncounterCellCapabilityDto,
  ): void {
    this.updateCell(cellIndex, (cell) => ({
      ...cell,
      capabilities: cell.capabilities.map((capability, index) =>
        capabilityIndex === index ? update(capability) : capability,
      ),
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
    this.selectedOptions.set([]);
  }

  protected chooseGridCell(cell: BoardCell): void {
    const action = this.selectedAction();
    if (!cell.targetable || cell.selection === null || action === null) return;
    this.toggleOption(
      cell.selection.kind,
      cell.selection.id,
      action.maximumTargets,
    );
  }

  protected executeAction(): void {
    const gameplay = this.store.view()?.gameplay;
    const actionId = this.selectedActionId();
    const targetIds = authorityTargetIds(this.selectedOptions());
    if (
      gameplay === null ||
      gameplay === undefined ||
      actionId === null ||
      (targetIds.length === 0 && this.selectedAction()?.maximumTargets !== 0)
    ) {
      return;
    }
    this.selectedActionId.set(null);
    this.selectedOptions.set([]);
    void this.store.command({
      expectedRevision: gameplay.stateRevision,
      actionId,
      actorId: gameplay.actorId,
      targetIds: [...targetIds],
    });
  }

  protected executeTurnControl(kind: string): void {
    const gameplay = this.store.view()?.gameplay;
    if (gameplay === null || gameplay === undefined) return;
    this.selectedActionId.set(null);
    this.selectedOptions.set([]);
    void this.store.control({
      expectedRevision: gameplay.stateRevision,
      actorId: gameplay.actorId,
      kind,
    });
  }

  protected actionOptionCount(action: GameplayActionView): number {
    return (
      action.candidateIds.length + action.cellIds.length + action.areaIds.length
    );
  }

  protected isSelectionSelected(
    selection: AuthorityOptionSelection | null,
  ): boolean {
    return (
      selection !== null && this.isOptionSelected(selection.kind, selection.id)
    );
  }

  protected isOptionSelected(kind: AuthorityOptionKind, id: string): boolean {
    return this.selectedOptions().some(
      (selection) => selection.kind === kind && selection.id === id,
    );
  }

  protected optionDisabled(
    kind: AuthorityOptionKind,
    id: string,
    maximumTargets: number,
  ): boolean {
    return (
      this.store.busy() ||
      (!this.isOptionSelected(kind, id) &&
        maximumTargets > 1 &&
        this.selectedOptions().length >= maximumTargets)
    );
  }

  protected toggleOption(
    kind: AuthorityOptionKind,
    id: string,
    maximumTargets: number,
  ): void {
    this.selectedOptions.update((current) =>
      toggleAuthorityOption(current, { kind, id }, maximumTargets),
    );
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

function initialParticipantCapability(
  owner: EncounterInitialCapabilityDto['owner'],
): EncounterInitialCapabilityDto {
  if (owner === 'vitality') {
    return { owner, value: { current: 10, max: 10 } };
  }
  if (owner === 'resource') {
    return { owner, id: 'resource-id', value: { current: 0, max: 0 } };
  }
  if (owner === 'modifier') {
    return {
      owner,
      stackingGroup: 'modifier-group',
      id: 'modifier-id',
      value: 0,
      remainingTurns: 1,
    };
  }
  return { owner, id: `${owner}-id`, value: 0 };
}

function initialCellCapability(
  kind: EncounterCellCapabilityDto['value']['kind'],
): EncounterCellCapabilityDto {
  const shared = {
    id: `capability.${kind}`,
    version: 1,
    definitionId: null,
  };
  if (kind === 'traversal') {
    return {
      ...shared,
      value: { kind, passable: true, movementCost: 1 },
    };
  }
  if (kind === 'flag') {
    return { ...shared, value: { kind, value: false } };
  }
  if (kind === 'integer') {
    return { ...shared, value: { kind, value: 0 } };
  }
  return { ...shared, value: { kind, valueId: 'value-id' } };
}

function diagnosticMatchesSetupPath(
  diagnosticPath: string,
  controlPath: string,
): boolean {
  return setupPathMatchSpecificity(diagnosticPath, controlPath) !== null;
}

function setupPathMatchSpecificity(
  diagnosticPath: string,
  controlPath: string,
): number | null {
  if (diagnosticPath === controlPath) return 0;
  if (
    controlPath.startsWith(`${diagnosticPath}.`) ||
    controlPath.startsWith(`${diagnosticPath}[`)
  ) {
    return 1;
  }
  if (
    diagnosticPath.startsWith(`${controlPath}.`) ||
    diagnosticPath.startsWith(`${controlPath}[`)
  ) {
    return 2;
  }
  return null;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return 'The selected file was not valid JSON setup data.';
}
