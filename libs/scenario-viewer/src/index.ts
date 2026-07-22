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
  ScenarioCellCapabilityDto,
  ScenarioInitialCapabilityDto,
  ScenarioParticipantDto,
  ScenarioRandomSourceDto,
  ScenarioSetupRequestDto,
  ScenarioTemplateDto,
  PlayDiagnosticDto,
} from '@asha-rulebench/protocol';
import { decodeScenarioDocument } from '@asha-rulebench/protocol';
import { browserTextFileInput } from '@asha-rulebench/platform';
import { createBrowserPlayWorkspaceStore } from '@asha-rulebench/store';

type DialogName = 'playBundle' | 'scenario' | 'artifact' | 'replay' | null;

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
  public readonly diagnostics = input.required<readonly PlayDiagnosticDto[]>();
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
        grid-template-columns: repeat(
          var(--board-columns),
          minmax(4.75rem, 1fr)
        );
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

      .choice-row,
      .compatibility-result {
        border: 1px solid var(--arb-border);
        display: grid;
        gap: 0.45rem;
        padding: 0.7rem;
      }

      .choice-row {
        align-items: start;
        grid-template-columns: auto minmax(0, 1fr);
      }

      .choice-row > span,
      .compatibility-result {
        display: grid;
        gap: 0.25rem;
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
            minmax(18rem, 45vh) minmax(24rem, 58vh);
        }

        .battlefield-wrap {
          min-height: 19rem;
        }

        .battlefield {
          gap: 0.25rem;
          grid-template-columns: repeat(var(--board-columns), minmax(0, 1fr));
          min-width: 100%;
        }

        .grid-cell {
          min-height: 4.25rem;
          padding: 0.2rem;
        }

        .piece strong {
          display: none;
        }

        .piece-token {
          font-size: 0.72rem;
          height: 2rem;
          width: 2rem;
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
                    [style.--board-columns]="boardWidth()"
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
                  <strong>No active session</strong>
                  <p>{{ view.summary }}</p>
                  <button
                    type="button"
                    (click)="
                      openDialog(
                        view.scenarioSetupRequired ? 'scenario' : 'playBundle'
                      )
                    "
                  >
                    {{
                      view.scenarioSetupRequired
                        ? 'Create Scenario'
                        : 'Choose play content'
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
                <ul class="participant-list" aria-label="Session participants">
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
                        >Stats:
                        {{
                          entity.stats.length === 0
                            ? 'none'
                            : entity.stats.join(', ')
                        }}</span
                      >
                      <span class="muted"
                        >Defenses:
                        {{
                          entity.defenses.length === 0
                            ? 'none'
                            : entity.defenses.join(', ')
                        }}</span
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
                  Participants appear after an explicit Scenario is accepted.
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
                    <strong>Battle complete</strong>
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
                      >Start a new Scenario from the Session menu to continue
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
                      <strong>Battle complete</strong>
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
                            @if (action.description) {
                              <span>{{ action.description }}</span>
                            }
                            <code>{{ action.id }}</code>
                            @if (action.tags.length > 0) {
                              <span class="muted">{{
                                action.tags.join(' · ')
                              }}</span>
                            }
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
                          @if (action.description) {
                            <p>{{ action.description }}</p>
                          }
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
                              Target {{ participantLabel(candidate) }}
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
                  The action palette opens when a PlayBundle and Scenario own an
                  active session.
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
                <ul class="event-list" aria-label="Combat history">
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
      dialogId="play-bundle-dialog"
      dialogTitle="Choose play content"
      dialogDescription="Select one semantic Ruleset and its compatible authored Content Packs, then compile and activate their declared PlayBundle."
      [open]="openDialogName() === 'playBundle'"
      (closeRequested)="closeDialog()"
    >
      <div class="dialog-body">
        @if (store.view(); as view) {
          <p class="section-label">{{ view.statusLabel }}</p>
          <label for="configured-source-set" class="section-label"
            >Configured source set</label
          >
          <select
            #configuredSourceSetSelect
            id="configured-source-set"
            class="ruleset-select"
            [disabled]="store.busy()"
            [value]="configuredSourceSetId()"
            (change)="
              selectConfiguredSourceSet(configuredSourceSetSelect.value)
            "
          >
            <option value="">Choose a configured source set</option>
            @for (location of store.configuredSourceSets(); track location.id) {
              <option [value]="location.id">{{ location.label }}</option>
            }
          </select>
          @if (store.sourceSetConfigurationError(); as configurationError) {
            <div class="diagnostic" role="alert">
              <strong
                >Local source-set configuration could not be loaded</strong
              >
              <span>{{ configurationError }}</span>
            </div>
          } @else if (store.configuredSourceSets().length === 0) {
            <p class="muted">
              No local source sets are configured. Custom roots remain available
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
            placeholder="rulesets/my-ruleset"
            [value]="store.rulesetRoot()"
            (input)="store.selectRulesetRoot(rulesetRootInput.value)"
          />
          <p class="muted">
            Rulebench treats this as the one Ruleset source and infers
            <code>src/index.ts</code>.
          </p>
          <label for="additional-source-roots" class="section-label"
            >Additional authored source roots</label
          >
          <textarea
            #additionalSourceRootsInput
            id="additional-source-roots"
            class="ruleset-input"
            rows="3"
            [disabled]="store.busy()"
            placeholder="/home/dev/my-content-pack"
            [value]="store.additionalSourceRoots()"
            (input)="
              store.selectAdditionalSourceRoots(
                additionalSourceRootsInput.value
              )
            "
          ></textarea>
          <p class="muted">
            Optional, one Content Pack, PlayBundle, or Scenario root per line.
            Each root is an explicit source and import boundary; Rulebench does
            not scan parent directories.
          </p>
          <button
            class="secondary"
            type="button"
            [disabled]="store.busy() || !rootSelectionComplete()"
            (click)="inspectRuleset()"
          >
            Inspect source set
          </button>

          @if (store.rulesetCatalog(); as catalog) {
            <dl class="facts">
              <div>
                <dt>Ruleset</dt>
                <dd>
                  {{ catalog.ruleset.id }}&#64;{{ catalog.ruleset.version }}
                </dd>
              </div>
              <div>
                <dt>Sources</dt>
                <dd>
                  @for (source of catalog.sourceSet.entries; track source.id) {
                    <span>
                      {{ source.label }}: <code>{{ source.sourceRoot }}</code>
                    </span>
                  }
                </dd>
              </div>
            </dl>
            <fieldset>
              <legend>Content Packs</legend>
              @for (contentPack of catalog.contentPacks; track contentPack.id) {
                <label class="choice-row">
                  <input
                    #contentPackCheckbox
                    type="checkbox"
                    [disabled]="store.busy()"
                    [checked]="
                      store.selectedContentPackIds().includes(contentPack.id)
                    "
                    (change)="
                      setContentPackSelected(
                        contentPack.id,
                        contentPackCheckbox.checked
                      )
                    "
                  />
                  <span>
                    <strong>{{ contentPack.label }}</strong>
                    <code
                      >{{ contentPack.id }}&#64;{{ contentPack.version }}</code
                    >
                    <small>
                      Requires {{ contentPack.requirements.length }} declared
                      operation, capability, value, and numeric-domain
                      provisions.
                    </small>
                  </span>
                </label>
              }
            </fieldset>

            @if (selectedCatalogPlayBundle(); as selectedBundle) {
              <div
                class="compatibility-result"
                [class.diagnostic]="!selectedBundle.compatible"
                [attr.role]="selectedBundle.compatible ? 'status' : 'alert'"
              >
                <strong>
                  {{
                    selectedBundle.compatible
                      ? 'Compatible PlayBundle'
                      : 'Content requirements are not satisfied'
                  }}
                </strong>
                <span>
                  {{ selectedBundle.id }}&#64;{{ selectedBundle.version }}
                </span>
                @for (
                  diagnostic of selectedBundle.diagnostics;
                  track diagnostic.code + diagnostic.path
                ) {
                  <span>{{ diagnostic.code }} · {{ diagnostic.message }}</span>
                }
              </div>
            } @else if (store.selectedContentPackIds().length > 0) {
              <div class="diagnostic" role="alert">
                <strong>No declared PlayBundle matches this selection</strong>
                <span
                  >Choose a Content Pack combination published by the
                  configured source set.</span
                >
              </div>
            }
          }

          @for (
            diagnostic of store.catalogDiagnostics();
            track diagnostic.code + diagnostic.path
          ) {
            <div class="diagnostic" role="alert">
              <strong>{{ diagnostic.code }}</strong>
              <span>{{ diagnostic.path }} · {{ diagnostic.message }}</span>
            </div>
          }

          <div class="button-row" aria-label="PlayBundle lifecycle controls">
            <button
              type="button"
              [disabled]="
                store.busy() ||
                selectedCatalogPlayBundle() === null ||
                !selectedCatalogPlayBundle()?.compatible
              "
              (click)="compilePlayBundle()"
            >
              Compile selected PlayBundle
            </button>
            <button
              class="secondary"
              type="button"
              [disabled]="store.busy() || view.phase !== 'candidate'"
              (click)="activatePlayBundle()"
            >
              Activate compiled PlayBundle
            </button>
          </div>
          <dl class="facts">
            <div>
              <dt>Lifecycle</dt>
              <dd>{{ view.phase }}</dd>
            </div>
            <div>
              <dt>Active PlayBundle artifact</dt>
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
      dialogId="scenario-dialog"
      dialogTitle="Scenario setup"
      dialogDescription="Create or load setup-only data pinned to the active PlayBundle. Rust validates the complete Scenario before replacing any session."
      [open]="openDialogName() === 'scenario'"
      (closeRequested)="closeDialog()"
    >
      <div class="dialog-body">
        @if (setupDraft(); as setup) {
          @if (scenarioTemplates().length > 0) {
            <section>
              <p class="section-label">Scenario examples</p>
              <ul class="action-list" aria-label="Available Scenario examples">
                @for (
                  template of scenarioTemplates();
                  track template.identity.id
                ) {
                  <li>
                    <button
                      class="action-choice"
                      type="button"
                      [attr.aria-pressed]="
                        setupDocumentName() === template.presentation.label
                      "
                      (click)="useScenarioTemplate(template)"
                    >
                      <strong>{{ template.presentation.label }}</strong>
                      @if (template.presentation.description) {
                        <span>{{ template.presentation.description }}</span>
                      }
                      <span class="muted">
                        {{ template.participants.length }} participants ·
                        {{ template.board.width }} × {{ template.board.height }}
                        board
                      </span>
                    </button>
                  </li>
                }
              </ul>
              <p class="muted">
                Examples contain setup only. Every action, target, reaction,
                roll, and turn transition remains interactive.
              </p>
            </section>
          }
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
                  : 'Selected setup: ' + setupDocumentName()
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
            data-setup-path="$.playBundleId"
            [attr.aria-invalid]="setupHasError('$.playBundleId')"
            [attr.aria-describedby]="setupDescribedBy('$.playBundleId')"
          >
            <p class="section-label">PlayBundle binding</p>
            <code>{{ setup.playBundleId }}</code>
            <arb-setup-diagnostics
              [diagnostics]="setupDiagnosticsFor('$.playBundleId')"
              [messageId]="setupDiagnosticId('$.playBundleId')"
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

          @if (participantProfiles().length > 0) {
            <fieldset class="definition-choices">
              <legend class="section-label">
                Add from participant profile
              </legend>
              <ul
                class="action-list"
                aria-label="Available participant profiles"
              >
                @for (
                  profile of participantProfiles();
                  track profile.definitionId
                ) {
                  <li>
                    <button
                      class="action-choice"
                      type="button"
                      (click)="addParticipantFromProfile(profile.definitionId)"
                    >
                      <strong>{{ profile.label }}</strong>
                      <span>{{ profile.role }}</span>
                      @if (profile.description) {
                        <span class="muted">{{ profile.description }}</span>
                      }
                    </button>
                  </li>
                }
              </ul>
            </fieldset>
          }

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
                    #setupControl
                    class="secondary"
                    type="button"
                    [attr.data-setup-path]="
                      owner === 'vitality' &&
                      participantVitalityCount(participant) === 0
                        ? participantPath(participantIndex, 'capabilities')
                        : null
                    "
                    [attr.aria-invalid]="
                      owner === 'vitality' &&
                      participantVitalityCount(participant) === 0
                        ? setupHasExactError(
                            participantPath(participantIndex, 'capabilities')
                          )
                        : null
                    "
                    [attr.aria-describedby]="
                      owner === 'vitality' &&
                      participantVitalityCount(participant) === 0
                        ? setupExactDescribedBy(
                            participantPath(participantIndex, 'capabilities')
                          )
                        : null
                    "
                    [disabled]="
                      owner === 'vitality' &&
                      participantVitalityCount(participant) !== 0
                    "
                    (click)="addParticipantCapability(participantIndex, owner)"
                  >
                    Add {{ owner }}
                  </button>
                }
              </div>
              <arb-setup-diagnostics
                [diagnostics]="
                  setupExactDiagnosticsFor(
                    participantPath(participantIndex, 'capabilities')
                  )
                "
                [messageId]="
                  setupDiagnosticId(
                    participantPath(participantIndex, 'capabilities')
                  )
                "
              />
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
                    @if (
                      capability.owner === 'stat' ||
                      capability.owner === 'defense'
                    ) {
                      <label>
                        <span class="section-label">
                          {{ capability.owner === 'stat' ? 'Stat' : 'Defense' }}
                        </span>
                        <select
                          #setupControl
                          #capabilityValueSelect
                          class="setup-select"
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
                          (change)="
                            updateParticipantCapabilityText(
                              participantIndex,
                              capabilityIndex,
                              'id',
                              capabilityValueSelect.value
                            )
                          "
                        >
                          <option value="">Choose a named value</option>
                          @for (
                            value of rulesetValuesFor(capability.owner);
                            track value.id
                          ) {
                            <option [value]="value.id">
                              {{ value.label }} · {{ value.id }}
                            </option>
                          }
                        </select>
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
                    } @else if (capability.owner !== 'vitality') {
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
                    #setupControl
                    class="secondary"
                    type="button"
                    [attr.data-setup-path]="
                      isDuplicateVitalityCapability(
                        participant,
                        capabilityIndex
                      )
                        ? capabilityPath(participantIndex, capabilityIndex)
                        : null
                    "
                    [attr.aria-invalid]="
                      isDuplicateVitalityCapability(
                        participant,
                        capabilityIndex
                      )
                        ? setupHasError(
                            capabilityPath(participantIndex, capabilityIndex)
                          )
                        : null
                    "
                    [attr.aria-describedby]="
                      isDuplicateVitalityCapability(
                        participant,
                        capabilityIndex
                      )
                        ? setupDescribedBy(
                            capabilityPath(participantIndex, capabilityIndex)
                          )
                        : null
                    "
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
                    <span>
                      <strong>{{ definition.label }}</strong>
                      <code>{{ definition.id }}</code>
                      @if (definition.description) {
                        <small>{{ definition.description }}</small>
                      }
                    </span>
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
                    #setupControl
                    class="secondary"
                    type="button"
                    [attr.data-setup-path]="
                      isDuplicateTraversalCapability(cell, capabilityIndex)
                        ? cellCapabilityFieldPath(
                            cellIndex,
                            capabilityIndex,
                            'value'
                          )
                        : null
                    "
                    [attr.aria-invalid]="
                      isDuplicateTraversalCapability(cell, capabilityIndex)
                        ? setupHasExactError(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'value'
                            )
                          )
                        : null
                    "
                    [attr.aria-describedby]="
                      isDuplicateTraversalCapability(cell, capabilityIndex)
                        ? setupExactDescribedBy(
                            cellCapabilityFieldPath(
                              cellIndex,
                              capabilityIndex,
                              'value'
                            )
                          )
                        : null
                    "
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
              (click)="startScenario()"
            >
              Validate and start Scenario
            </button>
            <button class="secondary" type="button" (click)="closeDialog()">
              Cancel
            </button>
          </div>
        } @else {
          <p>Activate a compiled PlayBundle before creating a Scenario.</p>
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
              <dt>PlayBundle</dt>
              <dd>{{ artifact.playBundle }}</dd>
            </div>
            <div>
              <dt>Ruleset</dt>
              <dd>{{ artifact.ruleset }}</dd>
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
          <p class="section-label">Exact Content Packs</p>
          <ul class="detail-list">
            @for (source of artifact.contentPacks; track source.identity) {
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
  protected readonly store = createBrowserPlayWorkspaceStore();
  protected readonly openDialogName = signal<DialogName>(null);
  protected readonly selectedActionId = signal<string | null>(null);
  protected readonly selectedOptions = signal<
    readonly AuthorityOptionSelection[]
  >([]);
  protected readonly setupDraft = signal<ScenarioSetupRequestDto | null>(null);
  protected readonly setupDocumentName = signal<string | null>(null);
  protected readonly setupImportError = signal<string | null>(null);

  private readonly textFileInput = browserTextFileInput();

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

  protected readonly configuredSourceSetId = computed(
    () => this.store.configuredSourceSetId() ?? '',
  );

  protected readonly selectedCatalogPlayBundle = computed(() => {
    const catalog = this.store.rulesetCatalog();
    if (catalog === null) return null;
    const selected = this.store.selectedContentPackIds();
    return (
      catalog.playBundles.find((bundle) =>
        sameStringSet(bundle.contentPackIds, selected),
      ) ?? null
    );
  });

  protected readonly scenarioTemplates = computed(() => {
    const catalog = this.store.rulesetCatalog();
    const activePlayBundle = this.store.view()?.artifact?.playBundle;
    if (catalog === null || activePlayBundle === undefined) return [];
    return catalog.scenarios.filter(
      (template) =>
        `${template.playBundle.id}@${template.playBundle.version}` ===
        activePlayBundle,
    );
  });

  protected readonly participantProfiles = computed(
    () => this.store.view()?.artifact?.participantProfiles ?? [],
  );

  protected readonly actionDefinitions = computed(() =>
    (this.store.view()?.artifact?.definitions ?? []).filter((definition) =>
      definition.contract.startsWith('action'),
    ),
  );

  protected readonly supportedRandomSources = computed<
    readonly ScenarioRandomSourceDto[]
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
        id: 'play-bundle',
        label: 'Play',
        items: [
          {
            id: 'choose-play-bundle',
            label: 'Choose Ruleset and Content Packs…',
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
            id: 'setup-scenario',
            label: this.store.view()?.gameplayAvailable
              ? 'Start new Scenario…'
              : 'Create Scenario…',
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
      if (this.openDialogName() === 'scenario' && diagnostics.length > 0) {
        this.focusFirstSetupDiagnostic();
      }
    });
  }

  public ngOnInit(): void {
    void this.store.refresh();
    void this.store.refreshConfiguredSourceSets();
  }

  protected handleMenuItem(item: ApplicationMenuItem): void {
    if (item.id === 'choose-play-bundle') {
      this.openDialog('playBundle');
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
    if (item.id === 'setup-scenario') {
      this.prepareScenarioDraft();
      this.openDialog('scenario');
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
    this.openDialog('playBundle');
    void this.store.inspectSelectedRuleset();
  }

  protected openDialog(dialog: Exclude<DialogName, null>): void {
    this.openDialogName.set(dialog);
  }

  protected closeDialog(): void {
    this.openDialogName.set(null);
  }

  protected inspectRuleset(): void {
    void this.store.inspectSelectedRuleset();
  }

  protected compilePlayBundle(): void {
    void this.store.compileSelectedPlayBundle();
  }

  protected selectConfiguredSourceSet(locationId: string): void {
    const location = this.store
      .configuredSourceSets()
      .find((candidate) => candidate.id === locationId);
    this.store.selectConfiguredSourceSet(location ?? null);
    if (location !== undefined) void this.store.inspectSelectedRuleset();
  }

  protected setContentPackSelected(
    contentPackId: string,
    selected: boolean,
  ): void {
    this.store.setContentPackSelected(contentPackId, selected);
  }

  protected activatePlayBundle(): void {
    void this.store.activatePlayBundle().then(() => {
      if (this.store.view()?.phase === 'active') {
        this.prepareScenarioDraft();
        this.openDialog('scenario');
      }
    });
  }

  protected prepareScenarioDraft(): void {
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
      schema: { id: 'asha.rpg.scenario', version: 1 },
      playBundleId: view.activeArtifactId,
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

  protected useScenarioTemplate(template: ScenarioTemplateDto): void {
    const activeArtifactId = this.store.view()?.activeArtifactId;
    if (activeArtifactId === null || activeArtifactId === undefined) return;
    this.setupDraft.set({
      schema: { id: 'asha.rpg.scenario', version: 1 },
      playBundleId: activeArtifactId,
      board: {
        ...template.board,
        cells: template.board.cells.map((cell) => ({
          ...cell,
          position: { ...cell.position },
          capabilities: cell.capabilities.map((capability) => ({
            ...capability,
            value: { ...capability.value },
          })),
        })),
      },
      participants: template.participants.map((participant) => ({
        ...participant,
        position: { ...participant.position },
        definitionIds: [...participant.definitionIds],
        capabilities: participant.capabilities.map(cloneInitialCapability),
      })),
      turn: {
        ...template.turn,
        initiativeOrder: [...template.turn.initiativeOrder],
      },
      randomSource: { ...template.randomSource },
    });
    this.setupDocumentName.set(template.presentation.label);
    this.setupImportError.set(null);
  }

  protected async loadSetupDocument(files: FileList | null): Promise<void> {
    const file = files?.item(0);
    if (file === null || file === undefined) return;
    try {
      const loaded = await this.textFileInput.readText(file);
      const parsed: unknown = JSON.parse(loaded.text);
      const setup = decodeScenarioDocument(parsed);
      this.setupDraft.set(setup);
      this.setupDocumentName.set(loaded.name);
      this.setupImportError.set(null);
    } catch (error: unknown) {
      this.setupDocumentName.set(null);
      this.setupImportError.set(errorMessage(error));
    }
  }

  protected randomSourceKey(source: ScenarioRandomSourceDto): string {
    return `${source.policyId}@${source.policyVersion}:${source.sourceId}@${source.sourceVersion}`;
  }

  protected randomSourceLabel(source: ScenarioRandomSourceDto): string {
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
      const participant: ScenarioParticipantDto = {
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

  protected addParticipantFromProfile(definitionId: string): void {
    const profile = this.participantProfiles().find(
      (candidate) => candidate.definitionId === definitionId,
    );
    if (profile === undefined) return;
    this.updateSetup((setup) => {
      const profileCount = setup.participants.filter((participant) =>
        participant.definitionIds.includes(profile.definitionId),
      ).length;
      const suffix = profileCount === 0 ? '' : `-${profileCount + 1}`;
      const id = `${profile.profileId}${suffix}`;
      const participant: ScenarioParticipantDto = {
        id,
        label: `${profile.label}${profileCount === 0 ? '' : ` ${profileCount + 1}`}`,
        teamId: profile.role === 'player' ? 'players' : 'creatures',
        position: firstAvailablePosition(setup),
        definitionIds: [profile.definitionId, ...profile.definitionIds],
        capabilities: profile.capabilities.map(cloneInitialCapability),
      };
      const participants = [...setup.participants, participant];
      return {
        ...setup,
        participants,
        turn: {
          ...setup.turn,
          initiativeOrder: participants.map((entry) => entry.id),
          currentActorId: setup.turn.currentActorId || id,
        },
      };
    });
  }

  protected rulesetValuesFor(owner: 'stat' | 'defense') {
    return (this.store.view()?.artifact?.rulesetValues ?? []).filter(
      (value) => value.kind === owner,
    );
  }

  protected participantLabel(participantId: string): string {
    const entity = this.store
      .view()
      ?.gameplay?.entities.find((candidate) => candidate.id === participantId);
    return entity === undefined
      ? participantId
      : `${entity.label} (${participantId})`;
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
    owner: ScenarioInitialCapabilityDto['owner'],
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
    kind: ScenarioCellCapabilityDto['value']['kind'],
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

  protected participantVitalityCount(
    participant: ScenarioParticipantDto,
  ): number {
    return participant.capabilities.filter(
      (capability) => capability.owner === 'vitality',
    ).length;
  }

  protected isDuplicateVitalityCapability(
    participant: ScenarioParticipantDto,
    capabilityIndex: number,
  ): boolean {
    const capability = participant.capabilities[capabilityIndex];
    if (capability?.owner !== 'vitality') return false;
    return participant.capabilities
      .slice(0, capabilityIndex)
      .some((candidate) => candidate.owner === 'vitality');
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

  protected isDuplicateTraversalCapability(
    cell: ScenarioSetupRequestDto['board']['cells'][number],
    capabilityIndex: number,
  ): boolean {
    const capability = cell.capabilities[capabilityIndex];
    if (capability?.value.kind !== 'traversal') return false;
    return cell.capabilities
      .slice(0, capabilityIndex)
      .some((candidate) => candidate.value.kind === 'traversal');
  }

  protected cellCapabilityFieldPath(
    cellIndex: number,
    capabilityIndex: number,
    suffix: string,
  ): string {
    return `${this.cellCapabilityPath(cellIndex, capabilityIndex)}.${suffix}`;
  }

  protected setupDiagnosticsFor(path: string): readonly PlayDiagnosticDto[] {
    return this.setupDiagnostics().filter((diagnostic) =>
      diagnosticMatchesSetupPath(diagnostic.path, path),
    );
  }

  protected setupHasError(path: string): boolean {
    return this.setupDiagnosticsFor(path).length > 0;
  }

  protected setupExactDiagnosticsFor(
    path: string,
  ): readonly PlayDiagnosticDto[] {
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

  protected startScenario(): void {
    const setup = this.setupDraft();
    if (setup === null) return;
    void this.store.startScenario(setup).then((started) => {
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
    update: (setup: ScenarioSetupRequestDto) => ScenarioSetupRequestDto,
  ): void {
    this.setupDraft.update((setup) => (setup === null ? null : update(setup)));
  }

  private focusFirstSetupDiagnostic(): void {
    const diagnostic = this.setupDiagnostics()[0];
    if (diagnostic === undefined) return;
    const candidate = this.setupControls()
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
      .reduce<
        | {
            readonly control: HTMLElement;
            readonly path: string;
            readonly specificity: number;
          }
        | undefined
      >((best, current) => {
        if (best === undefined || current.specificity < best.specificity) {
          return current;
        }
        return best;
      }, undefined);
    candidate?.control.focus();
  }

  private updateParticipant(
    index: number,
    update: (participant: ScenarioParticipantDto) => ScenarioParticipantDto,
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
      capability: ScenarioInitialCapabilityDto,
    ) => ScenarioInitialCapabilityDto,
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
      cell: ScenarioSetupRequestDto['board']['cells'][number],
    ) => ScenarioSetupRequestDto['board']['cells'][number],
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
      capability: ScenarioCellCapabilityDto,
    ) => ScenarioCellCapabilityDto,
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

function sameStringSet(
  left: readonly string[],
  right: readonly string[],
): boolean {
  if (left.length !== right.length) return false;
  const normalizedLeft = [...left].sort((first, second) =>
    first.localeCompare(second),
  );
  const normalizedRight = [...right].sort((first, second) =>
    first.localeCompare(second),
  );
  return normalizedLeft.every(
    (value, index) => value === normalizedRight[index],
  );
}

function firstAvailablePosition(
  setup: ScenarioSetupRequestDto,
): ScenarioParticipantDto['position'] {
  for (let y = 0; y < setup.board.height; y += 1) {
    for (let x = 0; x < setup.board.width; x += 1) {
      const occupied = setup.participants.some(
        (participant) =>
          participant.position.x === x && participant.position.y === y,
      );
      if (!occupied) return { x, y };
    }
  }
  return { x: 0, y: 0 };
}

function cloneInitialCapability(
  capability: ScenarioInitialCapabilityDto,
): ScenarioInitialCapabilityDto {
  if (capability.owner === 'vitality' || capability.owner === 'resource') {
    return { ...capability, value: { ...capability.value } };
  }
  return { ...capability };
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
  owner: ScenarioInitialCapabilityDto['owner'],
): ScenarioInitialCapabilityDto {
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
  kind: ScenarioCellCapabilityDto['value']['kind'],
): ScenarioCellCapabilityDto {
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
    diagnosticPath.startsWith(`${controlPath}.`) ||
    diagnosticPath.startsWith(`${controlPath}[`)
  ) {
    return 2;
  }
  if (isExplicitSetupParentRoute(diagnosticPath, controlPath)) return 1;
  return null;
}

function isExplicitSetupParentRoute(
  diagnosticPath: string,
  controlPath: string,
): boolean {
  if (diagnosticPath === '$.board') {
    return controlPath === '$.board.width' || controlPath === '$.board.height';
  }
  if (diagnosticPath === '$.turn') {
    return controlPath === '$.turn.round' || controlPath === '$.turn.turn';
  }
  if (diagnosticPath.endsWith('.position')) {
    return (
      controlPath === `${diagnosticPath}.x` ||
      controlPath === `${diagnosticPath}.y`
    );
  }
  if (/\.capabilities\[\d+\]$/.test(diagnosticPath)) {
    return controlPath.startsWith(`${diagnosticPath}.`);
  }
  if (diagnosticPath.endsWith('.capabilities')) {
    return /^\[\d+\]$/.test(controlPath.slice(diagnosticPath.length));
  }
  if (/\.capabilities\[\d+\]\.value$/.test(diagnosticPath)) {
    return controlPath === `${diagnosticPath}.movementCost`;
  }
  return false;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return 'The selected file was not valid JSON setup data.';
}
