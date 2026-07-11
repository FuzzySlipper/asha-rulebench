import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  input,
  output,
  signal,
  viewChild,
} from "@angular/core";
import type { ElementRef } from "@angular/core";
import {
  ApplicationMenubarComponent,
  type ApplicationMenuGroup,
  type ApplicationMenuItem,
  WorkbenchPanelComponent,
} from "@asha-rulebench/components";
import { LiveCombatStore, SessionStore } from "@asha-rulebench/store";

type EvidenceTab = "combat" | "events" | "trace" | "audit" | "state";
type InitiativePosition = "Current" | "Next" | "Queued" | "Complete";

@Component({
  selector: "arb-workbench-shell",
  imports: [ApplicationMenubarComponent, WorkbenchPanelComponent],
  templateUrl: "./workbench-shell.component.html",
  styleUrl: "./workbench-shell.component.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchShellComponent {
  private readonly evidenceTabOrder: readonly EvidenceTab[] = [
    "combat",
    "events",
    "trace",
    "audit",
    "state",
  ];
  private readonly liveStore = inject(LiveCombatStore);
  private readonly sessionStore = inject(SessionStore);
  private readonly gridPanel =
    viewChild.required<WorkbenchPanelComponent>("gridPanel");
  private readonly initiativePanel =
    viewChild.required<WorkbenchPanelComponent>("initiativePanel");
  private readonly statusPanel =
    viewChild.required<WorkbenchPanelComponent>("statusPanel");
  private readonly logPanel =
    viewChild.required<WorkbenchPanelComponent>("logPanel");
  private readonly actionsPanel =
    viewChild.required<WorkbenchPanelComponent>("actionsPanel");
  private readonly unitsPanel =
    viewChild.required<WorkbenchPanelComponent>("unitsPanel");
  private readonly commandFeedback =
    viewChild<ElementRef<HTMLElement>>("commandFeedback");

  readonly additionalMenuGroups = input<readonly ApplicationMenuGroup[]>([]);
  readonly deterministicMode = input<"session" | "scenario">("session");
  readonly applicationCommand = output<ApplicationMenuItem>();

  protected readonly menuStatus = signal("");
  protected readonly connection = computed(() => this.liveStore.connection());
  protected readonly snapshot = computed(() => this.liveStore.snapshot());
  protected readonly control = computed(() => this.liveStore.control());
  protected readonly options = computed(() => this.liveStore.options());
  protected readonly candidates = computed(() => this.liveStore.candidates());
  protected readonly preflight = computed(() => this.liveStore.preflight());
  protected readonly submission = computed(() => this.liveStore.submission());
  protected readonly deterministicStep = computed(() =>
    this.sessionStore.sessionStep(),
  );
  protected readonly deterministicScenario = computed(() => {
    if (this.deterministicMode() === "scenario") {
      return this.sessionStore.scenario();
    }
    const step = this.deterministicStep();
    if (step.kind === "data")
      return { kind: "data" as const, value: step.value.scenario };
    if (step.kind === "error")
      return { kind: "error" as const, error: step.error };
    return step.kind === "loading"
      ? { kind: "loading" as const }
      : { kind: "idle" as const };
  });
  protected readonly intent = computed(() => this.liveStore.intent());
  protected readonly attackRollInput = signal("17");
  protected readonly damageRollInput = signal("5");
  private commandSequence = 0;
  protected readonly commandBusy = computed(() =>
    [
      this.options(),
      this.candidates(),
      this.preflight(),
      this.submission(),
    ].some((state) => state.kind === "loading"),
  );
  protected readonly rollInputError = computed(() =>
    this.rollStream() === null
      ? "Attack and damage rolls must be integers."
      : null,
  );
  protected readonly canSubmit = computed(
    () =>
      this.intent().actorId.length > 0 &&
      this.intent().actionId.length > 0 &&
      this.intent().targetId.length > 0 &&
      this.rollStream() !== null &&
      !this.commandBusy(),
  );
  protected readonly currentParticipantIndex = computed(() => {
    const snapshot = this.snapshot();
    return snapshot.kind === "data"
      ? snapshot.value.participantOrderIds.findIndex(
          (id) => id === snapshot.value.currentActorId,
        )
      : -1;
  });
  protected readonly globalAnnouncement = computed(() => {
    const connection = this.connection();
    const snapshot = this.snapshot();
    const control = this.control();
    if (connection.kind === "error")
      return `Authority error: ${connection.error.message}`;
    if (snapshot.kind === "error")
      return `Session error: ${snapshot.error.message}`;
    if (control.kind === "error")
      return `Lifecycle command error: ${control.error.message}`;
    if (snapshot.kind === "data") {
      return `${snapshot.value.sessionId}, ${snapshot.value.lifecycleLabel}, round ${snapshot.value.roundLabel}, turn ${snapshot.value.turnLabel}, combat end ${snapshot.value.combatEndLabel}, finalization ${snapshot.value.finalizationLabel ?? "not finalized"}`;
    }
    return connection.kind === "data"
      ? "Authority connected; no session selected"
      : "Authority disconnected";
  });
  private readonly panelMenuGroups = computed<readonly ApplicationMenuGroup[]>(
    () => {
      const snapshot = this.snapshot();
      const lifecycle =
        snapshot.kind === "data" ? snapshot.value.lifecycleLabel : null;
      const busy = this.control().kind === "loading";
      return [
        { id: "file", label: "File", items: [] },
        {
          id: "scenario",
          label: "Scenario",
          items: [
            { id: "focus-grid", label: "Combat grid" },
            { id: "focus-initiative", label: "Initiative" },
          ],
        },
        {
          id: "run",
          label: "Run",
          items: [
            {
              id: "start-combat",
              label: "Start combat",
              disabled: busy || lifecycle !== "Ready",
            },
            {
              id: "advance-turn",
              label: "Advance turn",
              disabled: busy || lifecycle !== "In Progress",
            },
            {
              id: "end-combat",
              label: "End combat",
              disabled: busy || lifecycle === null || lifecycle === "Ended",
            },
            {
              id: "close-session",
              label: "Close session",
              disabled: busy || lifecycle !== "Ended",
            },
            { id: "focus-status", label: "Turn status" },
            { id: "focus-actions", label: "Available actions" },
            {
              id: "focus-current-actor",
              label: "Current actor",
              disabled: true,
            },
          ],
        },
        {
          id: "replay",
          label: "Replay",
          items: [{ id: "focus-log", label: "Evidence log" }],
        },
        {
          id: "view",
          label: "View",
          items: [{ id: "focus-units", label: "Active units" }],
        },
        { id: "preferences", label: "Preferences", items: [] },
      ];
    },
  );
  protected readonly menuGroups = computed(() =>
    this.panelMenuGroups().map((panelGroup) => {
      const additionalGroup = this.additionalMenuGroups().find(
        (group) => group.id === panelGroup.id,
      );
      return {
        ...panelGroup,
        items: [...(additionalGroup?.items ?? []), ...panelGroup.items],
      };
    }),
  );
  protected readonly selectedEvidenceTab = signal<EvidenceTab>("combat");
  protected readonly evidenceAnnouncement = computed(() => {
    const snapshot = this.snapshot();
    if (snapshot.kind !== "data") return "Deterministic evidence selected";
    return `${snapshot.value.combatLog.length} combat log entries and ${snapshot.value.auditLog.length} command audit entries available`;
  });
  protected readonly evidenceTabLabel = computed(() => {
    switch (this.selectedEvidenceTab()) {
      case "combat":
        return "Combat log";
      case "events":
        return "Accepted DomainEvents";
      case "trace":
        return "Rule resolution trace";
      case "audit":
        return "Command audit";
      case "state":
        return "State review";
    }
  });

  protected selectEvidenceTab(tab: EvidenceTab): void {
    this.selectedEvidenceTab.set(tab);
  }

  protected handleEvidenceTabKeydown(event: KeyboardEvent): void {
    if (!(event.currentTarget instanceof HTMLButtonElement)) return;
    const tablist = event.currentTarget.parentElement;
    if (tablist === null) return;
    const tabs = Array.from(
      tablist.querySelectorAll<HTMLButtonElement>('[role="tab"]'),
    );
    const currentIndex = tabs.indexOf(event.currentTarget);
    if (currentIndex < 0) return;

    let nextIndex: number | null = null;
    if (event.key === "ArrowRight")
      nextIndex = (currentIndex + 1) % tabs.length;
    if (event.key === "ArrowLeft")
      nextIndex = (currentIndex - 1 + tabs.length) % tabs.length;
    if (event.key === "Home") nextIndex = 0;
    if (event.key === "End") nextIndex = tabs.length - 1;
    if (nextIndex === null) return;

    event.preventDefault();
    const nextTab = this.evidenceTabOrder[nextIndex];
    if (nextTab !== undefined) this.selectedEvidenceTab.set(nextTab);
    tabs[nextIndex]?.focus();
  }

  protected invokeMenuItem(item: ApplicationMenuItem): void {
    if (this.invokeLifecycleCommand(item.id)) return;
    const target = this.panelForCommand(item.id);
    if (target !== null) {
      target.focus();
      this.menuStatus.set(`Focused ${item.label}`);
      return;
    }
    this.applicationCommand.emit(item);
    this.menuStatus.set(`Opened ${item.label}`);
  }

  protected participantById(participantId: string) {
    const snapshot = this.snapshot();
    return snapshot.kind === "data"
      ? (snapshot.value.participants.find(
          (participant) => participant.id === participantId,
        ) ?? null)
      : null;
  }

  protected combatantById(combatantId: string) {
    const scenario = this.deterministicScenario();
    return scenario.kind === "data"
      ? (scenario.value.combatants.find(
          (combatant) => combatant.id === combatantId,
        ) ?? null)
      : null;
  }

  protected gridCellLabel(
    x: number,
    y: number,
    terrainLabel: string,
    occupantIds: readonly string[],
  ): string {
    const occupantNames = occupantIds
      .map((occupantId) => this.combatantById(occupantId)?.name)
      .filter((name) => name !== undefined);
    const occupants =
      occupantNames.length === 0
        ? "unoccupied"
        : `occupied by ${occupantNames.join(", ")}`;
    return `Coordinate ${x}, ${y}; ${terrainLabel}; ${occupants}`;
  }

  protected occupantStateLabel(combatantId: string): string {
    const scenario = this.deterministicScenario();
    const combatant = this.combatantById(combatantId);
    if (scenario.kind !== "data" || combatant === null) return "Occupant";
    if (combatant.isActor) return "Selected actor";
    if (combatant.name === scenario.value.selectedTarget.targetLabel)
      return "Selected target";
    return "Occupant";
  }

  protected setAttackRoll(value: string): void {
    this.attackRollInput.set(value);
  }

  protected setDamageRoll(value: string): void {
    this.damageRollInput.set(value);
  }

  protected selectAction(actorId: string | null, actionId: string): void {
    this.liveStore.setIntent({
      actorId: actorId ?? "",
      actionId,
      targetId: this.intent().targetId,
    });
  }

  protected selectTarget(targetId: string): void {
    this.liveStore.setIntent({ ...this.intent(), targetId });
  }

  protected refreshCommandEvidence(): void {
    void this.refreshEvidence();
  }

  protected preflightIntent(): void {
    void this.liveStore
      .preflightIntent()
      .then(() => this.focusCommandFeedback());
  }

  protected submitIntent(): void {
    const rollStream = this.rollStream();
    if (rollStream === null) return;
    this.commandSequence += 1;
    void this.liveStore
      .submitIntent({
        id: `panel-command-${this.commandSequence}`,
        title: "Manual command",
        summary: "Submitted from the Rulebench available actions panel.",
        rollStream,
      })
      .then(async () => {
        await this.refreshEvidence();
        this.focusCommandFeedback();
      });
  }

  protected initiativePositionLabel(index: number): InitiativePosition {
    const snapshot = this.snapshot();
    if (snapshot.kind !== "data") return "Queued";
    if (snapshot.value.lifecycleLabel === "Ended") return "Complete";
    if (index === this.currentParticipantIndex()) return "Current";
    if (this.currentParticipantIndex() < 0) return "Queued";
    const nextIndex =
      (this.currentParticipantIndex() + 1) %
      snapshot.value.participantOrderIds.length;
    return index === nextIndex ? "Next" : "Queued";
  }

  private invokeLifecycleCommand(commandId: string): boolean {
    switch (commandId) {
      case "start-combat":
        void this.runLifecycleCommand("explicitStart");
        return true;
      case "advance-turn":
        void this.runLifecycleCommand("advanceTurn");
        return true;
      case "end-combat":
        void this.runLifecycleCommand("explicitEnd");
        return true;
      case "close-session":
        void this.liveStore.closeSession();
        return true;
      default:
        return false;
    }
  }

  private async runLifecycleCommand(
    kind: "explicitStart" | "advanceTurn" | "explicitEnd",
  ): Promise<void> {
    await this.liveStore.submitControl(kind);
    await this.refreshEvidence();
  }

  private async refreshEvidence(): Promise<void> {
    await Promise.all([
      this.liveStore.refreshOptions(),
      this.liveStore.refreshCandidates(),
    ]);
  }

  private rollStream(): readonly number[] | null {
    const attack = Number(this.attackRollInput());
    const damage = Number(this.damageRollInput());
    return Number.isInteger(attack) && Number.isInteger(damage)
      ? [attack, damage]
      : null;
  }

  private focusCommandFeedback(): void {
    this.commandFeedback()?.nativeElement.focus();
  }

  private panelForCommand(commandId: string): WorkbenchPanelComponent | null {
    switch (commandId) {
      case "focus-grid":
        return this.gridPanel();
      case "focus-initiative":
        return this.initiativePanel();
      case "focus-status":
        return this.statusPanel();
      case "focus-log":
        return this.logPanel();
      case "focus-actions":
        return this.actionsPanel();
      case "focus-units":
        return this.unitsPanel();
      default:
        return null;
    }
  }
}
