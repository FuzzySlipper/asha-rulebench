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
import { NgTemplateOutlet } from "@angular/common";
import type { ElementRef } from "@angular/core";
import {
  ApplicationDialogComponent,
  ApplicationMenubarComponent,
  type ApplicationMenuGroup,
  type ApplicationMenuItem,
  WorkbenchPanelComponent,
} from "@asha-rulebench/components";
import {
  LiveCombatStore,
  ReplayReviewStore,
  SessionStore,
} from "@asha-rulebench/store";

type EvidenceTab = "combat" | "events" | "trace" | "audit" | "state" | "replay";
type InitiativePosition = "Current" | "Next" | "Queued" | "Complete";

@Component({
  selector: "arb-workbench-shell",
  imports: [
    ApplicationDialogComponent,
    ApplicationMenubarComponent,
    WorkbenchPanelComponent,
    NgTemplateOutlet,
  ],
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
    "replay",
  ];
  private readonly liveStore = inject(LiveCombatStore);
  private readonly sessionStore = inject(SessionStore);
  private readonly replayStore = inject(ReplayReviewStore);
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
  protected readonly selectedActionCheckKind = computed(() => {
    const options = this.options();
    if (options.kind !== "data") return null;
    return (
      options.value.actions.find(
        (action) => action.actionId === this.intent().actionId,
      )?.checkKind ?? null
    );
  });
  protected readonly primaryRollLabel = computed(() => {
    switch (this.selectedActionCheckKind()) {
      case "savingThrow":
        return "Saving throw roll";
      case "contested":
        return "Actor contest roll";
      default:
        return "Attack roll";
    }
  });
  protected readonly secondaryRollLabel = computed(() =>
    this.selectedActionCheckKind() === "contested"
      ? "Target contest roll"
      : "Damage roll",
  );
  protected readonly candidates = computed(() => this.liveStore.candidates());
  protected readonly preflight = computed(() => this.liveStore.preflight());
  protected readonly submission = computed(() => this.liveStore.submission());
  protected readonly reaction = computed(() => this.liveStore.reaction());
  protected readonly reactionWindow = computed(() => {
    const snapshot = this.snapshot();
    return snapshot.kind === "data" ? snapshot.value.reactionWindow : null;
  });
  protected readonly automaticStep = computed(() =>
    this.liveStore.automaticStep(),
  );
  protected readonly automaticRun = computed(() =>
    this.liveStore.automaticRun(),
  );
  protected readonly automationPolicies = computed(() =>
    this.liveStore.automationPolicies(),
  );
  protected readonly selectedAutomationPolicyId = signal("");
  protected readonly selectedAutomationPolicy = computed(() => {
    const policies = this.automationPolicies();
    if (policies.kind !== "data") return null;
    return (
      policies.value.find(
        (policy) => policy.id === this.selectedAutomationPolicyId(),
      ) ??
      policies.value.find((policy) => policy.id === "firstAcceptedCandidate") ??
      policies.value[0] ??
      null
    );
  });
  protected readonly deterministicStep = computed(() =>
    this.sessionStore.sessionStep(),
  );
  protected readonly replayReview = computed(() => this.replayStore.review());
  protected readonly replayVerification = computed(() =>
    this.replayStore.verification(),
  );
  protected readonly replayComparison = computed(() =>
    this.replayStore.comparison(),
  );
  protected readonly selectedReplayCommand = computed(() => {
    const review = this.replayReview();
    const sequence = this.replayStore.selectedCommandSequence();
    return review.kind === "data"
      ? (review.value.commands.find(
          (command) => command.sequence === sequence,
        ) ?? null)
      : null;
  });
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
  protected readonly defaultRollMode = computed(() =>
    this.liveStore.defaultRollMode(),
  );
  protected readonly attackRollInput = signal("17");
  protected readonly damageRollInput = signal("5");
  protected readonly additionalRollInput = signal("");
  protected readonly automationConfigOpen = signal(false);
  protected readonly automaticRollInput = signal("17,5,2,5,17,5");
  protected readonly maxStepsInput = signal("8");
  protected readonly noCandidateBehavior = signal<"advanceTurn" | "stopRun">(
    "advanceTurn",
  );
  private commandSequence = 0;
  protected readonly commandBusy = computed(() =>
    [
      this.options(),
      this.candidates(),
      this.preflight(),
      this.submission(),
      this.reaction(),
    ].some((state) => state.kind === "loading"),
  );
  protected readonly rollInputError = computed(() =>
    this.defaultRollMode() === "supplied" && this.rollStream() === null
      ? "Rolls must be integers, with additional rolls separated by commas."
      : null,
  );
  protected readonly canSubmit = computed(
    () =>
      this.intent().actorId.length > 0 &&
      this.intent().actionId.length > 0 &&
      (this.intent().targetId.length > 0 ||
        (this.intent().targetIds?.length ?? 0) > 0 ||
        this.intent().targetCell !== undefined ||
        this.intent().destinationCell !== undefined) &&
      (this.defaultRollMode() === "authorityGenerated" ||
        this.rollStream() !== null) &&
      !this.commandBusy(),
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
      (this.defaultRollMode() === "authorityGenerated" ||
        this.automaticRollStream() !== null) &&
      this.maxSteps() !== null
    );
  });
  protected readonly automationValidation = computed(() => {
    if (this.maxSteps() === null)
      return "Max steps must be a positive integer.";
    if (
      this.defaultRollMode() === "supplied" &&
      this.automaticRollStream() === null
    )
      return "Roll stream must be a comma-separated list of integers.";
    return null;
  });
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

  protected retryAuthorityEvidence(): void {
    if (this.deterministicMode() === "scenario") {
      void this.sessionStore.retryScenario();
      return;
    }
    void this.sessionStore.retrySessionStep();
  }
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
            { id: "configure-automation", label: "Configure automatic run" },
            {
              id: "run-policy-step",
              label: "Run one policy step",
              disabled: !this.canRunAutomatic(),
            },
            {
              id: "run-bounded-combat",
              label: "Run bounded combat",
              disabled: !this.canRunAutomatic(),
            },
            {
              id: "stop-automatic-run",
              label: "Stop current run",
              disabled: !this.automationBusy(),
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
        {
          id: "preferences",
          label: "Preferences",
          items: [
            {
              id: "roll-mode-supplied",
              label: `Supplied rolls${this.defaultRollMode() === "supplied" ? " (current)" : ""}`,
            },
            {
              id: "roll-mode-generated",
              label: `Authority-generated rolls${this.defaultRollMode() === "authorityGenerated" ? " (current)" : ""}`,
            },
          ],
        },
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
      case "replay":
        return "Replay review";
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
    if (this.invokePreferenceCommand(item.id)) return;
    if (this.invokeAutomationCommand(item.id)) return;
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

  protected liveGridCellLabel(
    x: number,
    y: number,
    terrainLabels: readonly string[],
    occupantIds: readonly string[],
  ): string {
    const occupantNames = occupantIds
      .map((occupantId) => this.participantById(occupantId)?.name)
      .filter((name) => name !== undefined);
    const terrain = terrainLabels.join(", ") || "open";
    const occupants = occupantNames.length === 0
      ? "unoccupied"
      : `occupied by ${occupantNames.join(", ")}`;
    return `Coordinate ${x}, ${y}; ${terrain}; ${occupants}`;
  }

  protected liveCellOperation(x: number, y: number): "destination" | "target" | "area" | null {
    const options = this.options();
    const snapshot = this.snapshot();
    if (options.kind !== "data" || snapshot.kind !== "data") return null;
    const action = options.value.actions.find(
      (candidate) => candidate.actionId === this.intent().actionId,
    );
    if (action === undefined || !action.available) return null;
    if (action.targetMode === "cell") {
      if (
        action.targetSets.some(
          (targetSet) => targetSet.targetCell?.x === x && targetSet.targetCell.y === y,
        )
      ) {
        return "area";
      }
      return action.destinations.some((cell) => cell.x === x && cell.y === y)
        ? "destination"
        : null;
    }
    if (action.targetMode !== "entity") return null;
    const boardCell = snapshot.value.board.cells.find((cell) => cell.x === x && cell.y === y);
    return boardCell?.occupantIds.some((id) => action.targets.some((target) => target.id === id)) === true
      ? "target"
      : null;
  }

  protected selectLiveGridCell(x: number, y: number): void {
    const operation = this.liveCellOperation(x, y);
    if (operation === "destination") {
      this.liveStore.selectCellTarget({ x, y });
      return;
    }
    if (operation === "area") {
      const options = this.options();
      if (options.kind !== "data") return;
      const action = options.value.actions.find(
        (candidate) => candidate.actionId === this.intent().actionId,
      );
      const targetSet = action?.targetSets.find(
        (candidate) => candidate.targetCell?.x === x && candidate.targetCell.y === y,
      );
      if (targetSet !== undefined) {
        this.liveStore.selectTargetSet(targetSet.targetIds, targetSet.targetCell);
      }
      return;
    }
    if (operation !== "target") return;
    const snapshot = this.snapshot();
    const options = this.options();
    if (snapshot.kind !== "data" || options.kind !== "data") return;
    const action = options.value.actions.find((candidate) => candidate.actionId === this.intent().actionId);
    const cell = snapshot.value.board.cells.find((candidate) => candidate.x === x && candidate.y === y);
    const targetId = cell?.occupantIds.find((id) => action?.targets.some((target) => target.id === id));
    if (targetId !== undefined) this.liveStore.selectEntityTarget(targetId);
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

  protected setAdditionalRolls(value: string): void {
    this.additionalRollInput.set(value);
  }

  protected setRollMode(mode: "supplied" | "authorityGenerated"): void {
    this.liveStore.setDefaultRollMode(mode);
  }

  protected selectAction(actorId: string | null, actionId: string): void {
    if (actorId === null) return;
    this.liveStore.selectAction(actionId);
  }

  protected selectTarget(targetId: string): void {
    this.liveStore.selectEntityTarget(targetId);
  }

  protected selectTargetSet(
    targetIds: readonly string[],
    targetCell: Readonly<{ x: number; y: number }> | null,
  ): void {
    this.liveStore.selectTargetSet(targetIds, targetCell);
  }

  protected targetSetLabel(targetIds: readonly string[]): string {
    return targetIds
      .map((targetId) => this.participantById(targetId)?.name ?? targetId)
      .join(", ");
  }

  protected targetSetSelected(
    targetIds: readonly string[],
    targetCell: Readonly<{ x: number; y: number }> | null,
  ): boolean {
    const intent = this.intent();
    const sameIds =
      (intent.targetIds?.length ?? 0) === targetIds.length &&
      targetIds.every((targetId, index) => intent.targetIds?.[index] === targetId);
    const sameCell =
      targetCell === null
        ? intent.targetCell === undefined
        : intent.targetCell?.x === targetCell.x && intent.targetCell.y === targetCell.y;
    return sameIds && sameCell;
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
    if (rollStream === null && this.defaultRollMode() === "supplied") return;
    this.commandSequence += 1;
    void this.liveStore
      .submitIntent({
        id: `panel-command-${this.commandSequence}`,
        title: "Manual command",
        summary: "Submitted from the Rulebench available actions panel.",
        rollStream: rollStream ?? [],
      })
      .then(async () => {
        await this.refreshEvidence();
        this.focusCommandFeedback();
      });
  }

  protected respondToReaction(
    responseKind: "pass" | "accept",
    optionId: string | null,
  ): void {
    const snapshot = this.snapshot();
    if (snapshot.kind !== "data") return;
    const window = snapshot.value.reactionWindow;
    if (window === null || window.currentReactorId === null) return;
    void this.liveStore
      .submitReaction({
        windowId: window.id,
        reactorId: window.currentReactorId,
        responseKind,
        optionId,
      })
      .then(async () => {
        await this.refreshEvidence();
        this.focusCommandFeedback();
      });
  }

  protected setAutomaticRolls(value: string): void {
    this.automaticRollInput.set(value);
  }

  protected setMaxSteps(value: string): void {
    this.maxStepsInput.set(value);
  }

  protected closeAutomationConfig(): void {
    this.automationConfigOpen.set(false);
  }

  protected setDefaultRollMode(mode: "supplied" | "authorityGenerated"): void {
    this.liveStore.setDefaultRollMode(mode);
    this.menuStatus.set(
      mode === "supplied"
        ? "Supplied rolls selected"
        : "Authority-generated rolls selected",
    );
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

  private invokeAutomationCommand(commandId: string): boolean {
    switch (commandId) {
      case "configure-automation":
        this.automationConfigOpen.set(true);
        void this.liveStore.loadAutomationPolicies();
        this.menuStatus.set("Opened automatic run configuration");
        return true;
      case "run-policy-step":
        void this.runAutomaticStep();
        return true;
      case "run-bounded-combat":
        void this.runAutomaticCombat();
        return true;
      case "stop-automatic-run":
        this.liveStore.cancelAutomation();
        this.menuStatus.set("Stopped current automatic run");
        return true;
      default:
        return false;
    }
  }

  private invokePreferenceCommand(commandId: string): boolean {
    if (commandId === "roll-mode-supplied") {
      this.setDefaultRollMode("supplied");
      return true;
    }
    if (commandId === "roll-mode-generated") {
      this.setDefaultRollMode("authorityGenerated");
      return true;
    }
    return false;
  }

  private async runAutomaticStep(): Promise<void> {
    const rollStream = this.automaticRollStream();
    if (rollStream === null && this.defaultRollMode() === "supplied") return;
    this.commandSequence += 1;
    await this.liveStore.runAutomaticStep({
      id: `panel-automatic-step-${this.commandSequence}`,
      title: "Automatic policy step",
      summary: "One Rust-selected deterministic policy operation.",
      rollStream: rollStream ?? [],
      policy: this.automationPolicy(),
    });
    await this.refreshEvidence();
  }

  private async runAutomaticCombat(): Promise<void> {
    const rollStream = this.automaticRollStream();
    const maxSteps = this.maxSteps();
    if (
      (rollStream === null && this.defaultRollMode() === "supplied") ||
      maxSteps === null
    )
      return;
    this.commandSequence += 1;
    await this.liveStore.runAutomaticCombat({
      id: `panel-automatic-run-${this.commandSequence}`,
      title: "Bounded automatic policy run",
      summary:
        "Rust-selected deterministic operations within the configured guard.",
      maxSteps,
      rollStream: rollStream ?? [],
      policy: this.automationPolicy(),
    });
    await this.refreshEvidence();
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
    if (!Number.isInteger(attack) || !Number.isInteger(damage)) return null;

    const additionalText = this.additionalRollInput().trim();
    if (additionalText.length === 0) return [attack, damage];

    const additional = additionalText
      .split(",")
      .map((value) => Number(value.trim()));
    return additional.every(Number.isInteger)
      ? [attack, damage, ...additional]
      : null;
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
    const selected = this.selectedAutomationPolicy();
    return {
      id: selected?.id ?? "firstAcceptedCandidate",
      version: selected?.version ?? 1,
      noCandidateBehavior: this.noCandidateBehavior(),
    };
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
