import { ChangeDetectionStrategy, Component, computed, inject, signal } from "@angular/core";
import { LiveCombatStore, ReplayReviewStore } from "@asha-rulebench/store";

@Component({
  selector: "arb-policy-laboratory-dialog-content",
  standalone: true,
  templateUrl: "./policy-laboratory-dialog-content.html",
  styleUrl: "./policy-laboratory-dialog-content.css",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PolicyLaboratoryDialogContentComponent {
  private readonly store = inject(LiveCombatStore);
  private readonly replayStore = inject(ReplayReviewStore);

  protected readonly scenarios = computed(() => this.store.scenarios());
  protected readonly policies = computed(() => this.store.automationPolicies());
  protected readonly experiments = computed(() => this.store.experiments());
  protected readonly comparison = computed(() => this.store.experimentComparison());
  protected readonly replayReview = computed(() => this.replayStore.review());
  protected readonly selectedScenario = signal("");
  protected readonly primaryPolicy = signal("");
  protected readonly comparisonPolicy = signal("");
  protected readonly seedInput = signal("7,11");
  protected readonly maxStepsInput = signal("8");
  protected readonly noCandidateBehavior = signal<"advanceTurn" | "stopRun">(
    "advanceTurn",
  );
  protected readonly selectedExperimentId = signal("");
  protected readonly statusMessage = signal("");

  protected readonly effectiveScenarioId = computed(() => {
    if (this.selectedScenario().length > 0) return this.selectedScenario();
    const scenarios = this.scenarios();
    return scenarios.kind === "data" ? (scenarios.value[0]?.id ?? "") : "";
  });

  protected readonly compatiblePolicies = computed(() => {
    const scenarios = this.scenarios();
    const policies = this.policies();
    if (scenarios.kind !== "data" || policies.kind !== "data") return [];
    const scenario = scenarios.value.find(
      (candidate) => candidate.id === this.effectiveScenarioId(),
    );
    if (scenario === undefined) return [];
    return policies.value.filter((policy) =>
      policy.compatibility.some(
        (entry) =>
          entry.rulesetId === scenario.rulesetId &&
          entry.rulesetVersion === scenario.rulesetVersion &&
          entry.compatible,
      ),
    );
  });

  protected readonly effectivePrimaryPolicyId = computed(() =>
    this.compatiblePolicies().some(
      (policy) => policy.id === this.primaryPolicy(),
    )
      ? this.primaryPolicy()
      : (this.compatiblePolicies()[0]?.id ?? ""),
  );

  protected readonly activeExperiment = computed(() => {
    const experiments = this.experiments();
    if (experiments.kind !== "data") return null;
    const selected = experiments.value.find(
      (experiment) => experiment.id === this.selectedExperimentId(),
    );
    return selected ?? experiments.value.at(-1) ?? null;
  });

  protected readonly validationMessage = computed(() => {
    if (this.effectiveScenarioId().length === 0) return "Select a scenario.";
    if (this.effectivePrimaryPolicyId().length === 0)
      return "No compatible Rust policy is available for this ruleset.";
    if (this.seeds() === null)
      return "Seeds must be comma-separated unsigned integers.";
    if (this.maxSteps() === null) return "Max steps must be between 1 and 64.";
    const policyCount = this.comparisonPolicy().length > 0 ? 2 : 1;
    const trialCount = (this.seeds()?.length ?? 0) * policyCount;
    return trialCount > 16 ? "The matrix may contain at most 16 trials." : null;
  });

  protected selectScenario(value: string): void {
    this.selectedScenario.set(value);
    this.primaryPolicy.set("");
    this.comparisonPolicy.set("");
  }

  protected setSeedInput(value: string): void {
    this.seedInput.set(value);
  }

  protected setMaxStepsInput(value: string): void {
    this.maxStepsInput.set(value);
  }

  protected async refresh(): Promise<void> {
    await Promise.all([
      this.store.loadScenarios(),
      this.store.loadAutomationPolicies(),
      this.store.loadExperiments(),
    ]);
  }

  protected async createExperiment(): Promise<void> {
    const seeds = this.seeds();
    const maxSteps = this.maxSteps();
    if (seeds === null || maxSteps === null || this.validationMessage() !== null)
      return;
    const policies = [
      this.effectivePrimaryPolicyId(),
      ...(this.comparisonPolicy().length > 0 ? [this.comparisonPolicy()] : []),
    ].filter((id, index, values) => values.indexOf(id) === index);
    const experimentId = this.nextExperimentId();
    this.selectedExperimentId.set(experimentId);
    await this.store.createExperiment({
      id: experimentId,
      scenarioIds: [this.effectiveScenarioId()],
      policies: policies.map((id) => {
        const registration = this.compatiblePolicies().find(
          (policy) => policy.id === id,
        );
        return {
          id,
          version: registration?.version ?? 1,
          noCandidateBehavior: this.noCandidateBehavior(),
        };
      }),
      seeds,
      maxSteps,
    });
    this.statusMessage.set(`Created bounded matrix ${experimentId}.`);
  }

  protected async advanceExperiment(experimentId: string): Promise<void> {
    this.selectedExperimentId.set(experimentId);
    await this.store.advanceExperiment(experimentId);
    this.statusMessage.set(`Advanced ${experimentId} by one trial.`);
  }

  protected async cancelExperiment(experimentId: string): Promise<void> {
    await this.store.cancelExperiment(experimentId);
    this.statusMessage.set(`Cancelled ${experimentId}.`);
  }

  protected async compareCompletedTrials(): Promise<void> {
    const experiment = this.activeExperiment();
    if (experiment === null || experiment.trials.length < 2) return;
    const expected = experiment.trials[0];
    const actual = experiment.trials.at(-1);
    if (expected === undefined || actual === undefined) return;
    await this.store.compareExperimentTrials({
      expectedExperimentId: experiment.id,
      expectedTrialId: expected.id,
      actualExperimentId: experiment.id,
      actualTrialId: actual.id,
    });
  }

  protected async openReplay(packageId: string): Promise<void> {
    await this.replayStore.loadReview(packageId);
    await this.replayStore.loadVerification(packageId);
    this.statusMessage.set(`Opened replay package ${packageId}.`);
  }

  private seeds(): readonly number[] | null {
    const values = this.seedInput()
      .split(",")
      .map((value) => Number(value.trim()));
    return values.length > 0 &&
      values.every(
        (value) => Number.isInteger(value) && value >= 0 && value <= 4_294_967_295,
      )
      ? values
      : null;
  }

  private maxSteps(): number | null {
    const value = Number(this.maxStepsInput());
    return Number.isInteger(value) && value >= 1 && value <= 64 ? value : null;
  }

  private nextExperimentId(): string {
    const experiments = this.experiments();
    const ids =
      experiments.kind === "data"
        ? new Set(experiments.value.map((experiment) => experiment.id))
        : new Set<string>();
    let sequence = ids.size + 1;
    while (ids.has(`policy-lab-${sequence}`)) sequence += 1;
    return `policy-lab-${sequence}`;
  }
}
