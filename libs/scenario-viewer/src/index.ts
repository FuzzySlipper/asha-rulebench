import { Component, computed, inject, signal } from "@angular/core";
import type { OnInit } from "@angular/core";
import { LiveCombatStore, SessionStore } from "@asha-rulebench/store";
import type {
  RulebenchCommandOutcomeClassDto,
} from "@asha-rulebench/protocol";
import {
  ApplicationDialogComponent,
  type ApplicationMenuGroup,
  type ApplicationMenuItem,
} from "@asha-rulebench/components";
import { ContentPacksDialogContentComponent } from "./content-packs-dialog-content";
import { CapabilityManifestDialogContentComponent } from "./capability-manifest-dialog-content";
import { LiveCombatSetupDialogContentComponent } from "./live-combat-setup-dialog-content";
import { ReplayArchiveDialogContentComponent } from "./replay-archive-dialog-content";
import { PolicyLaboratoryDialogContentComponent } from "./policy-laboratory-dialog-content";
import { WorkbenchShellComponent } from "./workbench-shell.component";

@Component({
  imports: [
    ApplicationDialogComponent,
    CapabilityManifestDialogContentComponent,
    ContentPacksDialogContentComponent,
    LiveCombatSetupDialogContentComponent,
    PolicyLaboratoryDialogContentComponent,
    ReplayArchiveDialogContentComponent,
    WorkbenchShellComponent,
  ],
  selector: "arb-scenario-viewer-feature",
  standalone: true,
  styles: [
    `
      :host {
        display: block;
        min-height: 100vh;
      }

      .viewer {
        display: grid;
      }

      .catalog,
      .session {
        display: grid;
        gap: 10px;
        padding: 0.75rem 0;
      }

      .session {
        border-bottom: 1px solid var(--arb-border);
      }

      .catalog h2 {
        font-size: 0.95rem;
        margin: 0;
      }

      .session h2,
      .session h3 {
        font-size: 0.95rem;
        margin: 0;
      }

      .catalog-row {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .session-row,
      .session-actions {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .scenario-button,
      .step-button,
      .control-button {
        background: var(--arb-surface);
        border: 1px solid var(--arb-border);
        border-radius: 6px;
        color: var(--arb-text);
        cursor: pointer;
      }

      .scenario-button,
      .step-button {
        display: grid;
        gap: 3px;
        min-width: 190px;
        padding: 10px 12px;
        text-align: left;
      }

      .control-button {
        min-height: 34px;
        padding: 7px 12px;
      }

      .scenario-button[aria-pressed="true"],
      .step-button[aria-pressed="true"] {
        border-color: var(--arb-accent);
        box-shadow: inset 3px 0 0 var(--arb-accent);
      }

      .scenario-title,
      .step-title {
        font-weight: 700;
      }

      .scenario-meta,
      .step-meta,
      .catalog-status,
      .session-status {
        color: var(--arb-muted);
        font-size: 0.85rem;
      }

      h1,
      h2,
      h3,
      h4,
      p {
        margin: 0;
      }
    `,
  ],
  template: `
    <div class="viewer">
      <arb-workbench-shell
        [additionalMenuGroups]="applicationMenuGroups"
        [deterministicMode]="viewerMode()"
        (applicationCommand)="handleApplicationCommand($event)"
      />

      <arb-application-dialog
        dialogId="capability-manifest-dialog"
        dialogTitle="Runtime capabilities"
        dialogDescription="Inspect executable support derived from Rust registries and current host composition."
        [open]="activeDialog() === 'capabilities'"
        (closeRequested)="closeDialog()"
      >
        <arb-capability-manifest-dialog-content />
      </arb-application-dialog>

      <arb-application-dialog
        dialogId="content-packs-dialog"
        dialogTitle="Content packs"
        dialogDescription="Import, review, activate, and audit Rust-owned authored content."
        [open]="activeDialog() === 'content'"
        (closeRequested)="closeDialog()"
      >
        <arb-content-packs-dialog-content />
      </arb-application-dialog>

      <arb-application-dialog
        dialogId="live-combat-dialog"
        dialogTitle="Live combat setup"
        dialogDescription="Connect to the Rust authority, configure a scenario, and operate the current session."
        [open]="activeDialog() === 'live'"
        (closeRequested)="closeDialog()"
      >
        <arb-live-combat-setup-dialog-content />
      </arb-application-dialog>

      <arb-application-dialog
        dialogId="policy-laboratory-dialog"
        dialogTitle="Deterministic policy laboratory"
        dialogDescription="Configure bounded Rust policy matrices, monitor trials, cancel work, compare evidence, and open archived replays."
        [open]="activeDialog() === 'laboratory'"
        (closeRequested)="closeDialog()"
      >
        <arb-policy-laboratory-dialog-content />
      </arb-application-dialog>

      <arb-application-dialog
        dialogId="replay-review-dialog"
        dialogTitle="Replay archive"
        dialogDescription="Select, verify, and compare Rust replay packages."
        [open]="activeDialog() === 'replay'"
        (closeRequested)="closeDialog()"
      >
        <arb-replay-archive-dialog-content />
      </arb-application-dialog>

      <arb-application-dialog
        dialogId="scenario-cases-dialog"
        dialogTitle="Live authority viewer"
        dialogDescription="Inspect scenario and session evidence read directly from the running Rust authority host."
        [open]="activeDialog() === 'scenario'"
        (closeRequested)="closeDialog()"
      >
        <section class="session" aria-label="Combat session">
          <h2>Authority Session Evidence</h2>
          @switch (sessionCatalog().kind) {
            @case ("data") {
              @for (session of sessionCatalog().value; track session.id) {
                <div class="session-row">
                  @for (step of session.steps; track step.id) {
                    <button
                      class="step-button"
                      type="button"
                      [attr.aria-pressed]="
                        selectedSessionId() === session.id &&
                        selectedSessionStepId() === step.id &&
                        viewerMode() === 'session'
                      "
                      (click)="selectSessionStep(session.id, step.id)"
                    >
                      <span class="step-title"
                        >{{ step.logIndex }} · {{ step.title }}</span
                      >
                      <span class="step-meta">{{
                        sessionOutcomeClassLabel(step.outcomeClass)
                      }}</span>
                    </button>
                  }
                </div>
                <div
                  class="session-actions"
                  aria-label="Combat session controls"
                >
                  <button
                    class="control-button"
                    type="button"
                    (click)="previousSessionStep()"
                  >
                    Previous
                  </button>
                  <button
                    class="control-button"
                    type="button"
                    (click)="nextSessionStep()"
                  >
                    Next
                  </button>
                </div>
              }
            }
            @case ("loading") {
              <p class="session-status">Loading combat session</p>
            }
            @case ("error") {
              <p class="session-status">{{ sessionCatalog().error.message }}</p>
              <button class="control-button" type="button" (click)="retrySessionCatalog()">
                Retry sessions
              </button>
            }
            @case ("idle") {
              <p class="session-status">Combat session idle</p>
            }
          }
        </section>

        <section class="catalog" aria-label="Scenario catalog">
          <h2>Authority Scenario Evidence</h2>
          @switch (catalog().kind) {
            @case ("data") {
              <div class="catalog-row">
                @for (summary of catalog().value; track summary.id) {
                  <button
                    class="scenario-button"
                    type="button"
                    [attr.aria-pressed]="
                      viewerMode() === 'scenario' &&
                      selectedScenarioId() === summary.id
                    "
                    (click)="selectScenario(summary.id)"
                  >
                    <span class="scenario-title">{{ summary.title }}</span>
                    <span class="scenario-meta"
                      >{{ outcomeClassLabel(summary.outcomeClass) }} ·
                      {{ summary.seedLabel }}</span
                    >
                  </button>
                }
              </div>
            }
            @case ("loading") {
              <p class="catalog-status">Loading scenario catalog</p>
            }
            @case ("error") {
              <p class="catalog-status">{{ catalog().error.message }}</p>
              <button class="control-button" type="button" (click)="retryCatalog()">
                Retry scenarios
              </button>
            }
            @case ("idle") {
              <p class="catalog-status">Scenario catalog idle</p>
            }
          }
        </section>
      </arb-application-dialog>
    </div>
  `,
})
export class ScenarioViewerFeatureComponent implements OnInit {
  private readonly sessionStore = inject(SessionStore);
  private readonly liveStore = inject(LiveCombatStore);
  protected readonly activeDialog = signal<
    "capabilities" | "content" | "scenario" | "live" | "replay" | "laboratory" | null
  >(null);
  protected readonly applicationMenuGroups: readonly ApplicationMenuGroup[] = [
    {
      id: "view",
      label: "View",
      items: [{ id: "open-runtime-capabilities", label: "Runtime capabilities" }],
    },
    {
      id: "file",
      label: "File",
      items: [{ id: "open-content-packs", label: "Content packs" }],
    },
    {
      id: "scenario",
      label: "Scenario",
      items: [
        { id: "open-scenario-cases", label: "Scenario cases" },
        { id: "open-live-combat", label: "Live combat setup" },
      ],
    },
    {
      id: "replay",
      label: "Replay",
      items: [
        { id: "open-replay-review", label: "Replay archive" },
        { id: "open-policy-laboratory", label: "Policy laboratory" },
      ],
    },
  ];
  protected readonly viewerMode = signal<"session" | "scenario">("session");
  protected readonly catalog = computed(() => this.sessionStore.catalog());
  protected readonly selectedScenarioId = computed(() =>
    this.sessionStore.selectedScenarioId(),
  );
  protected readonly sessionCatalog = computed(() =>
    this.sessionStore.sessionCatalog(),
  );
  protected readonly selectedSessionId = computed(() =>
    this.sessionStore.selectedSessionId(),
  );
  protected readonly selectedSessionStepId = computed(() =>
    this.sessionStore.selectedSessionStepId(),
  );

  ngOnInit(): void {
    void this.loadInitialScenario();
  }

  protected handleApplicationCommand(item: ApplicationMenuItem): void {
    switch (item.id) {
      case "open-content-packs":
        this.activeDialog.set("content");
        return;
      case "open-scenario-cases":
        this.activeDialog.set("scenario");
        return;
      case "open-live-combat":
        this.activeDialog.set("live");
        return;
      case "open-replay-review":
        this.activeDialog.set("replay");
        return;
      case "open-policy-laboratory":
        this.activeDialog.set("laboratory");
        void Promise.all([
          this.liveStore.loadScenarios(),
          this.liveStore.loadAutomationPolicies(),
          this.liveStore.loadExperiments(),
        ]);
        return;
      case "open-runtime-capabilities":
        this.activeDialog.set("capabilities");
        void this.liveStore.loadCapabilities();
        return;
    }
  }

  protected closeDialog(): void {
    this.activeDialog.set(null);
  }

  protected selectScenario(scenarioId: string): void {
    this.viewerMode.set("scenario");
    void this.sessionStore.selectScenario(scenarioId);
  }

  protected selectSessionStep(sessionId: string, stepId: string): void {
    this.viewerMode.set("session");
    void this.sessionStore.selectSessionStep(sessionId, stepId);
  }

  protected nextSessionStep(): void {
    this.viewerMode.set("session");
    void this.sessionStore.nextSessionStep();
  }

  protected previousSessionStep(): void {
    this.viewerMode.set("session");
    void this.sessionStore.previousSessionStep();
  }

  protected retryCatalog(): void {
    void this.sessionStore.retryCatalog();
  }

  protected retrySessionCatalog(): void {
    void this.sessionStore.retrySessionCatalog();
  }

  protected outcomeClassLabel(
    outcomeClass: RulebenchCommandOutcomeClassDto,
  ): string {
    switch (outcomeClass) {
      case "acceptedHit":
        return "Accepted hit";
      case "acceptedMiss":
        return "Accepted miss";
      case "rejectedTargetLegality":
        return "Rejected target";
      case "rejectedInvalidCommand":
        return "Rejected invalid command";
    }
  }

  protected sessionOutcomeClassLabel(
    outcomeClass: RulebenchCommandOutcomeClassDto,
  ): string {
    switch (outcomeClass) {
      case "acceptedHit":
        return "Accepted hit";
      case "acceptedMiss":
        return "Accepted miss";
      case "rejectedTargetLegality":
        return "Rejected target";
      case "rejectedInvalidCommand":
        return "Rejected invalid command";
    }
  }

  private async loadInitialScenario(): Promise<void> {
    await this.sessionStore.loadSessionCatalog();
    await this.sessionStore.loadSessionStep();
    await this.sessionStore.loadCatalog();
    await this.sessionStore.loadScenario();
  }
}
