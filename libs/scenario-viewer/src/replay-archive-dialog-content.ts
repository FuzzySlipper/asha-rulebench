import { Component, computed, inject } from "@angular/core";
import type { OnInit } from "@angular/core";
import { ReplayReviewStore } from "@asha-rulebench/store";

@Component({
  selector: "arb-replay-archive-dialog-content",
  standalone: true,
  styles: [
    `
      :host {
        display: block;
      }
      .dialog-content {
        display: grid;
        gap: 14px;
        padding: 16px 44px;
      }
      .heading,
      .toolbar,
      .choice-row {
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
        display: grid;
        gap: 14px;
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }
      .evidence {
        border-top: 1px solid var(--arb-border);
        display: grid;
        gap: 7px;
        min-width: 0;
        padding-top: 8px;
      }
      .expected {
        border-top-color: #8b5e00;
      }
      .actual {
        border-top-color: #276749;
      }
      .trace {
        display: grid;
        gap: 6px;
        list-style: none;
        margin: 0;
        max-height: 280px;
        overflow: auto;
        padding: 0;
      }
      .trace li,
      .evidence-line {
        border-left: 3px solid var(--arb-border);
        overflow-wrap: anywhere;
        padding-left: 8px;
      }
      .participant-row {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
      }
      button,
      select {
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
      select {
        padding: 6px 8px;
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
        .dialog-content {
          padding: 14px 16px;
        }
        .evidence-grid {
          grid-template-columns: 1fr;
        }
      }
    `,
  ],
  template: `
    <section class="dialog-content" aria-label="Replay archive controls">
      <div class="heading">
        <div>
          <h2>Replay Review</h2>
          <p class="meta">Rust archive evidence</p>
        </div>
        <button type="button" (click)="refreshPackages()">
          Refresh archive
        </button>
      </div>

      @switch (packages().kind) {
        @case ("loading") {
          <p class="meta">Loading replay archive</p>
        }
        @case ("error") {
          <p role="alert">
            {{ packages().error.kind }} · {{ packages().error.message }}
          </p>
        }
        @case ("data") {
          <div class="choice-row" aria-label="Archived replay packages">
            @for (item of packages().value; track item.packageId) {
              <button
                type="button"
                [attr.aria-pressed]="selectedPackageId() === item.packageId"
                (click)="selectPackage(item.packageId)"
              >
                {{ item.packageId }} · {{ item.rulesetLabel }}
              </button>
            }
          </div>
        }
      }

      @switch (review().kind) {
        @case ("loading") {
          <p class="meta">Loading replay commands</p>
        }
        @case ("error") {
          <p role="alert">
            {{ review().error.kind }} · {{ review().error.message }}
          </p>
        }
        @case ("data") {
          <section class="detail" aria-label="Replay package detail">
            <div class="heading">
              <div>
                <h3>{{ review().value.title }}</h3>
                <p>{{ review().value.summary }}</p>
              </div>
              <p class="fingerprint">
                {{ review().value.finalFingerprintLabel }}
              </p>
            </div>
            <p class="meta">{{ review().value.provenanceLabel }}</p>
            @if (review().value.contentPackRootLabel !== null) {
              <section aria-label="Replay content pack provenance">
                <h4>Exact Content Pack Set</h4>
                <p>{{ review().value.contentPackRootLabel }}</p>
                <p class="fingerprint">{{ review().value.contentPackSetFingerprintLabel }}</p>
                <ul class="evidence-list">
                  @for (reference of review().value.contentPackReferenceLabels; track reference) {
                    <li>{{ reference }}</li>
                  }
                </ul>
              </section>
            } @else {
              <p class="meta">No authored content pack was bound to this replay.</p>
            }
            <div class="choice-row" aria-label="Replay commands">
              @for (
                command of review().value.commands;
                track command.sequence
              ) {
                <button
                  type="button"
                  [attr.aria-pressed]="
                    selectedCommandSequence() === command.sequence
                  "
                  (click)="selectCommand(command.sequence)"
                >
                  {{ command.sequenceLabel }} · {{ command.commandKindLabel }} ·
                  {{ command.id }}
                </button>
              }
            </div>
          </section>
        }
      }

      <div class="evidence-grid">
        <section class="evidence" aria-label="Replay verification">
          <div class="heading">
            <h3>Verification</h3>
            <button
              type="button"
              [disabled]="selectedPackageId() === null"
              (click)="verifySelected()"
            >
              Verify
            </button>
          </div>
          @if (verification().kind === "data") {
            <p
              [class.accepted]="verification().value.accepted"
              [class.rejected]="!verification().value.accepted"
            >
              {{ verification().value.decisionLabel }} ·
              {{ verification().value.finalizedLabel }}
            </p>
            <p>{{ verification().value.verifiedStepLabel }}</p>
            <p class="reason">
              {{ verification().value.mismatchLabel ?? "No mismatch" }}
            </p>
            <p class="fingerprint">
              {{
                verification().value.fingerprintLabel ?? "No final fingerprint"
              }}
            </p>
          }
          @if (verification().kind === "loading") {
            <p class="meta">Rust is verifying replay evidence</p>
          }
          @if (verification().kind === "error") {
            <p role="alert">{{ verification().error.message }}</p>
          }
        </section>
        <section class="evidence" aria-label="Replay comparison">
          <div class="heading">
            <h3>Comparison</h3>
            <div class="toolbar">
              <label
                >Actual
                <select
                  #actualPackage
                  [value]="comparisonPackageId() ?? ''"
                  (change)="selectComparisonPackage(actualPackage.value)"
                >
                  @if (packages().kind === "data") {
                    @for (item of packages().value; track item.packageId) {
                      <option
                        [value]="item.packageId"
                        [selected]="comparisonPackageId() === item.packageId"
                      >
                        {{ item.packageId }}
                      </option>
                    }
                  }
                </select></label
              ><button
                type="button"
                [disabled]="!canCompare()"
                (click)="compareSelected()"
              >
                Compare
              </button>
            </div>
          </div>
          @if (comparison().kind === "data") {
            <p
              [class.accepted]="comparison().value.matches"
              [class.rejected]="!comparison().value.matches"
            >
              {{ comparison().value.resultLabel }}
            </p>
            <p>
              {{ comparison().value.packageLabel }} ·
              {{ comparison().value.comparedCommandLabel }}
            </p>
            @if (comparison().value.firstDifference; as difference) {
              <div class="evidence-line">
                <strong>First difference · {{ difference.codeLabel }}</strong>
                <p class="reason">
                  {{ difference.commandLabel }} · {{ difference.path }}
                </p>
                <p>Expected: {{ difference.expectedSummary }}</p>
                <p>Actual: {{ difference.actualSummary }}</p>
              </div>
            }
          }
          @if (comparison().kind === "loading") {
            <p class="meta">Comparing Rust replay packages</p>
          }
          @if (comparison().kind === "error") {
            <p role="alert">{{ comparison().error.message }}</p>
          }
        </section>
      </div>

      @if (selectedCommand(); as command) {
        <section class="detail" aria-label="Replay command evidence">
          <div class="heading">
            <div>
              <h3>{{ command.sequenceLabel }} · {{ command.id }}</h3>
              <p>{{ command.summary }}</p>
            </div>
            <p class="meta">Supplied rolls · {{ command.suppliedRollLabel }}</p>
          </div>
          <div class="evidence-grid">
            <section
              class="evidence expected"
              aria-label="Expected replay evidence"
            >
              <h4>Expected</h4>
              <p
                [class.accepted]="command.expected.accepted"
                [class.rejected]="!command.expected.accepted"
              >
                {{ command.expected.decisionLabel }}
              </p>
              <p class="fingerprint">
                Before {{ command.expected.stateBeforeLabel }}
              </p>
              <p class="fingerprint">
                After {{ command.expected.stateAfterLabel }}
              </p>
              <h4>Events</h4>
              @for (event of command.expected.eventLabels; track $index) {
                <p class="evidence-line">{{ event }}</p>
              } @empty {
                <p class="meta">No accepted events</p>
              }
              <h4>Roll evidence</h4>
              @for (roll of command.expected.rollLabels; track $index) {
                <p class="evidence-line">{{ roll }}</p>
              } @empty {
                <p class="meta">No consumed rolls</p>
              }
              <h4>Audit</h4>
              @for (audit of command.expected.auditLabels; track $index) {
                <p class="evidence-line">{{ audit }}</p>
              } @empty {
                <p class="meta">No command audit entries</p>
              }
              <h4>Trace</h4>
              <ul class="trace">
                @for (entry of command.expected.trace; track $index) {
                  <li>
                    <strong
                      >{{ entry.sequenceLabel }} · {{ entry.phaseLabel }} ·
                      {{ entry.statusLabel }}</strong
                    >
                    <p>{{ entry.message }}</p>
                    <p class="reason">{{ entry.detail }}</p>
                  </li>
                } @empty {
                  <li class="meta">No trace entries</li>
                }
              </ul>
            </section>
            <section
              class="evidence actual"
              aria-label="Actual replay evidence"
            >
              <h4>Actual</h4>
              <p
                [class.accepted]="command.actual.accepted"
                [class.rejected]="!command.actual.accepted"
              >
                {{ command.actual.decisionLabel }}
              </p>
              <p class="fingerprint">
                Before {{ command.actual.stateBeforeLabel }}
              </p>
              <p class="fingerprint">
                After {{ command.actual.stateAfterLabel }}
              </p>
              <h4>Events</h4>
              @for (event of command.actual.eventLabels; track $index) {
                <p class="evidence-line">{{ event }}</p>
              } @empty {
                <p class="meta">No accepted events</p>
              }
              <h4>Roll evidence</h4>
              @for (roll of command.actual.rollLabels; track $index) {
                <p class="evidence-line">{{ roll }}</p>
              } @empty {
                <p class="meta">No consumed rolls</p>
              }
              <h4>Audit</h4>
              @for (audit of command.actual.auditLabels; track $index) {
                <p class="evidence-line">{{ audit }}</p>
              } @empty {
                <p class="meta">No command audit entries</p>
              }
              <h4>Trace</h4>
              <ul class="trace">
                @for (entry of command.actual.trace; track $index) {
                  <li>
                    <strong
                      >{{ entry.sequenceLabel }} · {{ entry.phaseLabel }} ·
                      {{ entry.statusLabel }}</strong
                    >
                    <p>{{ entry.message }}</p>
                    <p class="reason">{{ entry.detail }}</p>
                  </li>
                } @empty {
                  <li class="meta">No trace entries</li>
                }
              </ul>
            </section>
          </div>
          <section class="evidence" aria-label="Replay resulting state">
            <h4>Resulting state</h4>
            <p>
              Round {{ command.snapshot.roundLabel }} · turn
              {{ command.snapshot.turnLabel }} ·
              {{ command.snapshot.lifecycleLabel }}
            </p>
            <div class="participant-row">
              @for (
                participant of command.snapshot.participants;
                track participant.id
              ) {
                <span
                  >{{ participant.name }} · {{ participant.hitPointLabel }} ·
                  {{ participant.statusLabel }}</span
                >
              }
            </div>
            <h4>Combat log</h4>
            @for (entry of command.snapshot.combatLog; track entry.id) {
              <div class="evidence-line">
                <strong>{{ entry.sequenceLabel }} · {{ entry.title }}</strong>
                <p>{{ entry.summary }}</p>
              </div>
            } @empty {
              <p class="meta">No combat log entries</p>
            }
            <h4>Session audit</h4>
            @for (entry of command.snapshot.auditLog; track entry.id) {
              <p class="evidence-line">
                {{ entry.sequenceLabel }} · {{ entry.decisionLabel }} ·
                {{ entry.eventCount }} events · {{ entry.traceCount }} trace
                entries
              </p>
            } @empty {
              <p class="meta">No session audit entries</p>
            }
          </section>
        </section>
      }
    </section>
  `,
})
export class ReplayArchiveDialogContentComponent implements OnInit {
  private readonly store = inject(ReplayReviewStore);
  protected readonly packages = computed(() => this.store.packages());
  protected readonly review = computed(() => this.store.review());
  protected readonly verification = computed(() => this.store.verification());
  protected readonly comparison = computed(() => this.store.comparison());
  protected readonly selectedPackageId = computed(() =>
    this.store.selectedPackageId(),
  );
  protected readonly selectedCommandSequence = computed(() =>
    this.store.selectedCommandSequence(),
  );
  protected readonly comparisonPackageId = computed(() =>
    this.store.comparisonPackageId(),
  );
  protected readonly selectedCommand = computed(() => {
    const review = this.review();
    return review.kind === "data"
      ? (review.value.commands.find(
          (command) => command.sequence === this.selectedCommandSequence(),
        ) ?? null)
      : null;
  });
  protected readonly canCompare = computed(
    () =>
      this.selectedPackageId() !== null && this.comparisonPackageId() !== null,
  );

  ngOnInit(): void {
    void this.initialize();
  }
  protected refreshPackages(): void {
    void this.initialize();
  }
  protected selectCommand(sequence: number): void {
    this.store.selectCommand(sequence);
  }
  protected selectComparisonPackage(packageId: string): void {
    this.store.selectComparisonPackage(packageId);
  }
  protected selectPackage(packageId: string): void {
    void this.loadPackage(packageId);
  }
  protected verifySelected(): void {
    const packageId = this.selectedPackageId();
    if (packageId !== null) void this.store.loadVerification(packageId);
  }
  protected compareSelected(): void {
    const expectedPackageId = this.selectedPackageId();
    const actualPackageId = this.comparisonPackageId();
    if (expectedPackageId !== null && actualPackageId !== null)
      void this.store.compare(expectedPackageId, actualPackageId);
  }
  private async initialize(): Promise<void> {
    await this.store.loadPackages();
    const packageId = this.store.selectedPackageId();
    if (packageId !== null) await this.loadPackage(packageId);
  }
  private async loadPackage(packageId: string): Promise<void> {
    await Promise.all([
      this.store.loadReview(packageId),
      this.store.loadVerification(packageId),
    ]);
    const packages = this.store.packages();
    if (
      packages.kind === "data" &&
      this.store.comparisonPackageId() === packageId
    ) {
      this.store.selectComparisonPackage(
        packages.value.find((item) => item.packageId !== packageId)
          ?.packageId ?? packageId,
      );
    }
    this.compareSelected();
  }
}
