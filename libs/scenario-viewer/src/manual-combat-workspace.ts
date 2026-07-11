import { Component, computed, inject, signal } from "@angular/core";
import type { OnInit } from "@angular/core";
import { LiveCombatStore } from "@asha-rulebench/store";

@Component({
  selector: "arb-manual-combat-workspace",
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
      .workspace {
        border-bottom: 1px solid var(--arb-border);
        display: grid;
        gap: 14px;
        padding: 16px 44px;
      }
      .heading,
      .toolbar,
      .choice-row,
      .participant-list,
      .evidence-grid,
      .field-row {
        align-items: center;
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }
      .heading {
        justify-content: space-between;
      }
      .detail {
        border-left: 3px solid var(--arb-border);
        display: grid;
        gap: 10px;
        padding-left: 12px;
      }
      .evidence-grid {
        align-items: start;
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
      }
      .evidence {
        border-top: 1px solid var(--arb-border);
        display: grid;
        gap: 6px;
        min-width: 0;
        padding-top: 8px;
      }
      .participant {
        border-left: 3px solid var(--arb-border);
        display: grid;
        gap: 2px;
        min-width: 180px;
        padding-left: 8px;
      }
      .log {
        display: grid;
        gap: 7px;
        list-style: none;
        margin: 0;
        padding: 0;
      }
      .log li {
        border-left: 3px solid var(--arb-border);
        padding-left: 8px;
      }
      button,
      input {
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        color: var(--arb-text);
        min-height: 34px;
      }
      button {
        cursor: pointer;
        padding: 7px 10px;
      }
      button:disabled {
        cursor: default;
        opacity: 0.5;
      }
      button[aria-pressed="true"] {
        border-color: var(--arb-accent);
        box-shadow: inset 3px 0 0 var(--arb-accent);
      }
      input {
        padding: 6px 8px;
        width: 150px;
      }
      input.roll {
        width: 74px;
      }
      h2,
      h3,
      h4,
      p {
        margin: 0;
      }
      h2 {
        font-size: 0.95rem;
      }
      h3,
      h4 {
        font-size: 0.86rem;
      }
      .meta,
      .state,
      .reason,
      .fingerprint {
        color: var(--arb-muted);
        font-size: 0.82rem;
        overflow-wrap: anywhere;
      }
      .accepted {
        color: #276749;
        font-weight: 700;
      }
      .rejected,
      [role="alert"] {
        color: #9b2c2c;
        font-weight: 700;
      }
      @media (max-width: 860px) {
        .workspace {
          padding: 14px 16px;
        }
        .evidence-grid {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <section class="workspace" aria-label="Live combat controls">
      <div class="heading">
        <div>
          <h2>Live Combat</h2>
          <p class="meta">Rust authority session</p>
        </div>
        <div class="toolbar">
          <button
            type="button"
            [disabled]="connection().kind === 'loading'"
            (click)="connect()"
          >
            Connect
          </button>
          <button
            type="button"
            [disabled]="connection().kind !== 'data'"
            (click)="refreshSessions()"
          >
            Refresh sessions
          </button>
          <button
            type="button"
            [disabled]="connection().kind === 'idle'"
            (click)="disconnect()"
          >
            Disconnect
          </button>
        </div>
      </div>

      @switch (connection().kind) {
        @case ("idle") {
          <p class="state">Live authority disconnected</p>
        }
        @case ("loading") {
          <p class="state">Connecting to Rust authority</p>
        }
        @case ("error") {
          <p role="alert">
            {{ connection().error.code }} · {{ connection().error.message }}
          </p>
        }
        @case ("data") {
          <p class="meta">
            {{ connection().value.authoritySurface }} · protocol
            {{ connection().value.protocolVersion }}
          </p>
          <div class="detail">
            <h3>Scenario and session</h3>
            @switch (scenarios().kind) {
              @case ("loading") {
                <p class="state">Loading live scenarios</p>
              }
              @case ("error") {
                <p role="alert">
                  {{ scenarios().error.code }} · {{ scenarios().error.message }}
                </p>
              }
              @case ("data") {
                <div class="choice-row" aria-label="Live scenario choices">
                  @for (scenario of scenarios().value; track scenario.id) {
                    <button
                      type="button"
                      [attr.aria-pressed]="selectedScenarioId() === scenario.id"
                      (click)="selectScenario(scenario.id)"
                    >
                      {{ scenario.title }}
                    </button>
                  }
                </div>
              }
            }
            <div class="field-row">
              <label
                >Session
                <input
                  #sessionId
                  [value]="sessionIdInput()"
                  (input)="setSessionId(sessionId.value)"
              /></label>
              <button
                type="button"
                [disabled]="!canCreateSession()"
                (click)="createSession()"
              >
                Create session
              </button>
            </div>
            @if (selectedScenario(); as scenario) {
              <section class="evidence-grid" aria-label="Scenario setup">
                <section class="evidence">
                  <h4>Ruleset</h4>
                  <p>
                    {{ scenario.rulesetId }} · {{ scenario.rulesetVersion }}
                  </p>
                </section>
                <section class="evidence">
                  <h4>Content</h4>
                  <p>
                    {{
                      scenario.contentPackId === null
                        ? "Built-in fixture content"
                        : scenario.contentPackId +
                          " · " +
                          scenario.contentPackVersion
                    }}
                  </p>
                </section>
                <section class="evidence">
                  <h4>Participant order</h4>
                  @for (
                    participantId of participantOrder();
                    track participantId;
                    let index = $index
                  ) {
                    @if (participantById(participantId); as participant) {
                      <div class="choice-row">
                        <span
                          >{{ index + 1 }} · {{ participant.name }} ·
                          {{ participant.sideId }} · initiative
                          {{ participant.initiative }}</span
                        ><button
                          type="button"
                          [attr.aria-label]="participant.name + ' earlier'"
                          [disabled]="index === 0"
                          (click)="moveParticipant(index, -1)"
                        >
                          Earlier</button
                        ><button
                          type="button"
                          [attr.aria-label]="participant.name + ' later'"
                          [disabled]="index === participantOrder().length - 1"
                          (click)="moveParticipant(index, 1)"
                        >
                          Later
                        </button>
                      </div>
                    }
                  }
                </section>
              </section>
            }
            @if (sessions().kind === "data" && sessions().value.length > 0) {
              <div class="choice-row" aria-label="Live sessions">
                @for (session of sessions().value; track session.sessionId) {
                  <button
                    type="button"
                    [attr.aria-pressed]="
                      selectedSessionId() === session.sessionId
                    "
                    (click)="selectSession(session.sessionId)"
                  >
                    {{ session.sessionId }} · {{ session.lifecycleLabel }}
                  </button>
                }
              </div>
            }
          </div>
        }
      }

      @switch (snapshot().kind) {
        @case ("loading") {
          <p class="state">Loading live session</p>
        }
        @case ("error") {
          <p role="alert">
            {{ snapshot().error.code }} · {{ snapshot().error.message }}
          </p>
        }
        @case ("data") {
          <section class="detail" aria-label="Live session state">
            <div class="heading">
              <div>
                <h3>
                  {{ snapshot().value.sessionId }} ·
                  {{ snapshot().value.lifecycleLabel }}
                </h3>
                <p class="fingerprint">
                  {{ snapshot().value.fingerprintLabel }}
                </p>
              </div>
              <div class="toolbar" aria-label="Live lifecycle controls">
                <button
                  type="button"
                  [disabled]="
                    busy() || snapshot().value.lifecycleLabel !== 'Ready'
                  "
                  (click)="controlCombat('explicitStart')"
                >
                  Start
                </button>
                <button
                  type="button"
                  [disabled]="
                    busy() || snapshot().value.lifecycleLabel !== 'In Progress'
                  "
                  (click)="controlCombat('advanceTurn')"
                >
                  Advance turn
                </button>
                <button
                  type="button"
                  [disabled]="
                    busy() || snapshot().value.lifecycleLabel === 'Ended'
                  "
                  (click)="controlCombat('explicitEnd')"
                >
                  End
                </button>
                <button
                  type="button"
                  [disabled]="
                    busy() || snapshot().value.lifecycleLabel !== 'Ended'
                  "
                  (click)="closeSession()"
                >
                  Close
                </button>
              </div>
            </div>
            <p class="meta">
              Round {{ snapshot().value.roundLabel }} · turn
              {{ snapshot().value.turnLabel }} · actor
              {{ snapshot().value.currentActorId ?? "none" }}
            </p>
          </section>

          <section class="detail" aria-label="Automatic combat controls">
            <div class="heading">
              <div>
                <h3>Automatic control</h3>
                <p class="meta">
                  firstAcceptedCandidate v1 · Rust selects each operation
                </p>
              </div>
              <div class="toolbar">
                <button
                  type="button"
                  [attr.aria-pressed]="noCandidateBehavior() === 'advanceTurn'"
                  (click)="noCandidateBehavior.set('advanceTurn')"
                >
                  Advance turn</button
                ><button
                  type="button"
                  [attr.aria-pressed]="noCandidateBehavior() === 'stopRun'"
                  (click)="noCandidateBehavior.set('stopRun')"
                >
                  Stop run
                </button>
              </div>
            </div>
            <div class="field-row">
              <label
                >Max steps
                <input
                  class="roll"
                  #maxSteps
                  type="number"
                  min="1"
                  [value]="maxStepsInput()"
                  (input)="maxStepsInput.set(maxSteps.value)"
              /></label>
              <label
                >Roll stream
                <input
                  #autoRolls
                  [value]="automaticRollInput()"
                  (input)="automaticRollInput.set(autoRolls.value)"
              /></label>
              <button
                type="button"
                [disabled]="!canRunAutomatic()"
                (click)="runAutomaticStep()"
              >
                Run step
              </button>
              <button
                type="button"
                [disabled]="!canRunAutomatic()"
                (click)="runAutomaticCombat()"
              >
                Run bounded
              </button>
              <button
                type="button"
                [disabled]="!automationBusy()"
                (click)="cancelAutomation()"
              >
                Stop
              </button>
            </div>
            <p class="reason">
              Replay verification: the current live session run is not archived
            </p>
            <div class="evidence-grid">
              <section class="evidence" aria-label="Automatic next decision">
                <h4>Next decision</h4>
                @if (automaticStep().kind === "data") {
                  <p
                    [class.accepted]="automaticStep().value.accepted"
                    [class.rejected]="!automaticStep().value.accepted"
                  >
                    {{ automaticStep().value.decisionLabel }} ·
                    {{ automaticStep().value.operationLabel ?? "No operation" }}
                  </p>
                  <p class="reason">
                    {{ automaticStep().value.selectedActionId ?? "No action" }}
                    →
                    {{ automaticStep().value.selectedTargetId ?? "No target" }}
                  </p>
                  <p class="reason">{{ automaticStep().value.reason }}</p>
                }
                @if (automaticStep().kind === "loading") {
                  <p class="state">Rust is selecting one operation</p>
                }
                @if (automaticStep().kind === "error") {
                  <p role="alert">{{ automaticStep().error.message }}</p>
                }
              </section>
              <section class="evidence" aria-label="Automatic run status">
                <h4>Bounded run</h4>
                @if (automaticRun().kind === "data") {
                  <p
                    [class.accepted]="automaticRun().value.accepted"
                    [class.rejected]="!automaticRun().value.accepted"
                  >
                    {{ automaticRun().value.decisionLabel }}
                  </p>
                  <p>
                    {{ automaticRun().value.executedStepCount }}/{{
                      automaticRun().value.maxSteps
                    }}
                    steps · {{ automaticRun().value.finalLifecycleLabel }}
                  </p>
                  <p class="reason">{{ automaticRun().value.reason }}</p>
                  <p class="fingerprint">
                    {{ automaticRun().value.finalFingerprintLabel }}
                  </p>
                }
                @if (automaticRun().kind === "loading") {
                  <p class="state">Rust is running within the step guard</p>
                }
                @if (automaticRun().kind === "error") {
                  <p role="alert">{{ automaticRun().error.message }}</p>
                }
              </section>
              <section class="evidence" aria-label="Automatic policy status">
                <h4>Policy configuration</h4>
                <p>firstAcceptedCandidate v1</p>
                <p class="reason">
                  No candidate:
                  {{
                    noCandidateBehavior() === "advanceTurn"
                      ? "Advance turn"
                      : "Stop run"
                  }}
                </p>
                <p class="reason">Guard: {{ maxStepsInput() }} steps</p>
              </section>
            </div>
          </section>
        }
      }
    </section>
  `,
})
export class ManualCombatWorkspaceComponent implements OnInit {
  private readonly store = inject(LiveCombatStore);
  protected readonly connection = computed(() => this.store.connection());
  protected readonly scenarios = computed(() => this.store.scenarios());
  protected readonly sessions = computed(() => this.store.sessions());
  protected readonly snapshot = computed(() => this.store.snapshot());
  protected readonly options = computed(() => this.store.options());
  protected readonly candidates = computed(() => this.store.candidates());
  protected readonly preflight = computed(() => this.store.preflight());
  protected readonly submission = computed(() => this.store.submission());
  protected readonly automaticStep = computed(() => this.store.automaticStep());
  protected readonly automaticRun = computed(() => this.store.automaticRun());
  protected readonly selectedScenarioId = computed(() =>
    this.store.selectedScenarioId(),
  );
  protected readonly selectedScenario = computed(() => {
    const scenarios = this.scenarios();
    return scenarios.kind === "data"
      ? (scenarios.value.find(
          (scenario) => scenario.id === this.selectedScenarioId(),
        ) ?? null)
      : null;
  });
  protected readonly selectedSessionId = computed(() =>
    this.store.selectedSessionId(),
  );
  protected readonly sessionIdInput = signal("manual-session");
  protected readonly automaticRollInput = signal("17,5,2,5,17,5");
  protected readonly maxStepsInput = signal("8");
  protected readonly noCandidateBehavior = signal<"advanceTurn" | "stopRun">(
    "advanceTurn",
  );
  protected readonly participantOrder = signal<readonly string[]>([]);
  private commandSequence = 0;
  protected readonly busy = computed(() =>
    [
      this.snapshot(),
      this.options(),
      this.candidates(),
      this.preflight(),
      this.submission(),
      this.store.control(),
    ].some((state) => state.kind === "loading"),
  );
  protected readonly canCreateSession = computed(
    () =>
      this.connection().kind === "data" &&
      this.selectedScenarioId() !== null &&
      this.sessionIdInput().trim().length > 0,
  );
  protected readonly automationBusy = computed(
    () =>
      this.automaticStep().kind === "loading" ||
      this.automaticRun().kind === "loading",
  );
  protected readonly canRunAutomatic = computed(() => {
    const snapshot = this.snapshot();
    return (
      snapshot.kind === "data" &&
      snapshot.value.lifecycleLabel === "In Progress" &&
      !this.automationBusy() &&
      this.automaticRollStream() !== null &&
      this.maxSteps() !== null
    );
  });

  ngOnInit(): void {
    void this.initialize();
  }
  protected connect(): void {
    void this.initialize();
  }
  protected disconnect(): void {
    this.store.disconnect();
  }
  protected refreshSessions(): void {
    void this.store.loadSessions();
  }
  protected selectScenario(id: string): void {
    this.store.selectScenario(id);
    this.syncParticipantOrder();
  }
  protected setSessionId(value: string): void {
    this.sessionIdInput.set(value);
  }
  protected participantById(participantId: string) {
    return (
      this.selectedScenario()?.participants.find(
        (participant) => participant.id === participantId,
      ) ?? null
    );
  }
  protected moveParticipant(index: number, offset: -1 | 1): void {
    const next = [...this.participantOrder()];
    const targetIndex = index + offset;
    const current = next[index];
    const target = next[targetIndex];
    if (current === undefined || target === undefined) return;
    next[index] = target;
    next[targetIndex] = current;
    this.participantOrder.set(next);
  }
  protected createSession(): void {
    const scenarioId = this.selectedScenarioId();
    if (scenarioId === null) return;
    void this.store
      .createSession(
        this.sessionIdInput().trim(),
        scenarioId,
        this.participantOrder(),
      )
      .then(() => this.refreshEvidence());
  }
  protected selectSession(sessionId: string): void {
    void this.store.selectSession(sessionId).then(() => this.refreshEvidence());
  }
  protected controlCombat(
    kind: "explicitStart" | "explicitEnd" | "advanceTurn",
  ): void {
    void this.store.submitControl(kind).then(() => this.refreshEvidence());
  }
  protected closeSession(): void {
    void this.store.closeSession();
  }
  protected refreshEvidence(): void {
    void Promise.all([
      this.store.refreshOptions(),
      this.store.refreshCandidates(),
    ]);
  }
  protected runAutomaticStep(): void {
    const rollStream = this.automaticRollStream();
    if (rollStream === null) return;
    this.commandSequence += 1;
    void this.store
      .runAutomaticStep({
        id: `automatic-step-${this.commandSequence}`,
        title: "Automatic step",
        summary: "One Rust-selected automatic operation.",
        rollStream,
        policy: this.automationPolicy(),
      })
      .then(() => this.refreshEvidence());
  }
  protected runAutomaticCombat(): void {
    const rollStream = this.automaticRollStream();
    const maxSteps = this.maxSteps();
    if (rollStream === null || maxSteps === null) return;
    this.commandSequence += 1;
    void this.store
      .runAutomaticCombat({
        id: `automatic-run-${this.commandSequence}`,
        title: "Bounded automatic run",
        summary: "Rust-selected operations within the configured step guard.",
        maxSteps,
        rollStream,
        policy: this.automationPolicy(),
      })
      .then(() => this.refreshEvidence());
  }
  protected cancelAutomation(): void {
    this.store.cancelAutomation();
  }
  private async initialize(): Promise<void> {
    await this.store.connect();
    if (this.store.connection().kind !== "data") return;
    await Promise.all([this.store.loadScenarios(), this.store.loadSessions()]);
    this.syncParticipantOrder();
  }
  private automaticRollStream(): readonly number[] | null {
    const values = this.automaticRollInput()
      .split(",")
      .map((value) => Number(value.trim()));
    return values.length > 0 && values.every(Number.isInteger) ? values : null;
  }
  private maxSteps(): number | null {
    const value = Number(this.maxStepsInput());
    return Number.isInteger(value) && value > 0 ? value : null;
  }
  private automationPolicy(): {
    readonly id: string;
    readonly version: number;
    readonly noCandidateBehavior: "advanceTurn" | "stopRun";
  } {
    return {
      id: "firstAcceptedCandidate",
      version: 1,
      noCandidateBehavior: this.noCandidateBehavior(),
    };
  }
  private syncParticipantOrder(): void {
    this.participantOrder.set(
      this.selectedScenario()?.participants.map(
        (participant) => participant.id,
      ) ?? [],
    );
  }
}
