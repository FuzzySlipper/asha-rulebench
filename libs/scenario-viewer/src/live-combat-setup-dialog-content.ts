import { Component, computed, inject, signal } from "@angular/core";
import type { OnInit } from "@angular/core";
import { LiveCombatStore } from "@asha-rulebench/store";

@Component({
  selector: "arb-live-combat-setup-dialog-content",
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
      .setup {
        display: grid;
        gap: 0.9rem;
      }
      .heading,
      .toolbar,
      .choice-row,
      .field-row {
        align-items: center;
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
      }
      .heading {
        justify-content: space-between;
      }
      .detail,
      .setup-section {
        border-left: 3px solid var(--arb-border);
        display: grid;
        gap: 0.65rem;
        padding-left: 0.75rem;
      }
      .setup-grid {
        display: grid;
        gap: 0.75rem;
        grid-template-columns: repeat(3, minmax(0, 1fr));
      }
      button,
      input {
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        color: var(--arb-text);
        min-height: 2.15rem;
      }
      button {
        cursor: pointer;
        padding: 0.45rem 0.65rem;
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
        padding: 0.4rem 0.5rem;
        width: 10rem;
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
      .state {
        color: var(--arb-muted);
        font-size: 0.82rem;
        overflow-wrap: anywhere;
      }
      [role="alert"] {
        color: var(--arb-danger);
        font-weight: 700;
      }
      @media (max-width: 52rem) {
        .setup-grid {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <section class="setup" aria-label="Live combat setup controls">
      <div class="heading">
        <div>
          <h2>Live combat setup</h2>
          <p class="meta">Rust authority connection and session selection</p>
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
          <p class="state" aria-busy="true">Connecting to Rust authority</p>
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
                <p class="state" aria-busy="true">Loading live scenarios</p>
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
              <div class="setup-grid" aria-label="Scenario setup">
                <section class="setup-section">
                  <h4>Ruleset</h4>
                  <p>
                    {{ scenario.rulesetId }} · {{ scenario.rulesetVersion }}
                  </p>
                </section>
                <section class="setup-section">
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
                <section class="setup-section">
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
                        >
                        <button
                          type="button"
                          [attr.aria-label]="participant.name + ' earlier'"
                          [disabled]="index === 0"
                          (click)="moveParticipant(index, -1)"
                        >
                          Earlier
                        </button>
                        <button
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
              </div>
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
      @if (snapshot().kind === "loading") {
        <p class="state" aria-busy="true">Creating or loading live session</p>
      }
      @if (snapshot().kind === "error") {
        <p role="alert">
          {{ snapshot().error.code }} · {{ snapshot().error.message }}
        </p>
      }
    </section>
  `,
})
export class LiveCombatSetupDialogContentComponent implements OnInit {
  private readonly store = inject(LiveCombatStore);
  protected readonly connection = computed(() => this.store.connection());
  protected readonly scenarios = computed(() => this.store.scenarios());
  protected readonly sessions = computed(() => this.store.sessions());
  protected readonly snapshot = computed(() => this.store.snapshot());
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
  protected readonly participantOrder = signal<readonly string[]>([]);
  protected readonly canCreateSession = computed(
    () =>
      this.connection().kind === "data" &&
      this.selectedScenarioId() !== null &&
      this.sessionIdInput().trim().length > 0,
  );

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
    void this.store.createSession(
      this.sessionIdInput().trim(),
      scenarioId,
      this.participantOrder(),
    );
  }

  protected selectSession(sessionId: string): void {
    void this.store.selectSession(sessionId);
  }

  private async initialize(): Promise<void> {
    await this.store.connect();
    if (this.store.connection().kind !== "data") return;
    await Promise.all([this.store.loadScenarios(), this.store.loadSessions()]);
    this.syncParticipantOrder();
  }

  private syncParticipantOrder(): void {
    this.participantOrder.set(
      this.selectedScenario()?.participants.map(
        (participant) => participant.id,
      ) ?? [],
    );
  }
}
