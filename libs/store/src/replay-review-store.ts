import { Injectable, signal } from "@angular/core";
import type { Signal } from "@angular/core";
import type {
  ClassifiedError,
} from "@asha-rulebench/protocol";
import {
  projectReplayArchiveItem,
  projectReplayComparison,
  projectReplayReview,
  projectReplayVerification,
  type RulebenchReplayArchiveItemView,
  type RulebenchReplayComparisonView,
  type RulebenchReplayReviewView,
  type RulebenchReplayVerificationView,
} from "@asha-rulebench/domain";
import type { ReplayReviewTransport } from "@asha-rulebench/transport";
import type { ClockPort } from "@asha-rulebench/platform";
import type { AsyncState } from "./async-state";

@Injectable()
export class ReplayReviewStore {
  private readonly _packages = signal<AsyncState<readonly RulebenchReplayArchiveItemView[]>>({ kind: "idle" });
  readonly packages: Signal<AsyncState<readonly RulebenchReplayArchiveItemView[]>> = this._packages.asReadonly();
  private readonly _review = signal<AsyncState<RulebenchReplayReviewView>>({ kind: "idle" });
  readonly review: Signal<AsyncState<RulebenchReplayReviewView>> = this._review.asReadonly();
  private readonly _verification = signal<AsyncState<RulebenchReplayVerificationView>>({ kind: "idle" });
  readonly verification: Signal<AsyncState<RulebenchReplayVerificationView>> = this._verification.asReadonly();
  private readonly _comparison = signal<AsyncState<RulebenchReplayComparisonView>>({ kind: "idle" });
  readonly comparison: Signal<AsyncState<RulebenchReplayComparisonView>> = this._comparison.asReadonly();
  private readonly _selectedPackageId = signal<string | null>(null);
  readonly selectedPackageId = this._selectedPackageId.asReadonly();
  private readonly _selectedCommandSequence = signal<number | null>(null);
  readonly selectedCommandSequence = this._selectedCommandSequence.asReadonly();
  private readonly _comparisonPackageId = signal<string | null>(null);
  readonly comparisonPackageId = this._comparisonPackageId.asReadonly();
  private generation = 0;

  constructor(private readonly transport: ReplayReviewTransport, private readonly clock: ClockPort) {}

  async loadPackages(): Promise<void> {
    const generation = ++this.generation;
    this._packages.set({ kind: "loading" });
    const result = await this.transport.listReplayPackages();
    if (generation !== this.generation) return;
    if (result.ok) {
      const packages = result.value.map(projectReplayArchiveItem);
      this._packages.set({ kind: "data", value: packages });
      this._selectedPackageId.set(this._selectedPackageId() ?? packages[0]?.packageId ?? null);
      this._comparisonPackageId.set(
        this._comparisonPackageId() ?? packages.find((item) => item.packageId !== this._selectedPackageId())?.packageId ?? null,
      );
    } else {
      this._packages.set({ kind: "error", error: classified(result.error) });
    }
    this.clock.now();
  }

  async loadReview(packageId: string): Promise<void> {
    this._selectedPackageId.set(packageId);
    const generation = ++this.generation;
    this._review.set({ kind: "loading" });
    this._comparison.set({ kind: "idle" });
    const result = await this.transport.loadReplayPackage(packageId);
    if (generation !== this.generation) return;
    if (result.ok) {
      const review = projectReplayReview(result.value);
      this._review.set({ kind: "data", value: review });
      this._selectedCommandSequence.set(review.commands[0]?.sequence ?? null);
    } else {
      this._review.set({ kind: "error", error: classified(result.error) });
    }
    this.clock.now();
  }

  async loadVerification(packageId: string): Promise<void> {
    const generation = this.generation;
    this._verification.set({ kind: "loading" });
    const result = await this.transport.loadReplayVerification(packageId);
    if (generation !== this.generation) return;
    this._verification.set(result.ok ? { kind: "data", value: projectReplayVerification(result.value) } : { kind: "error", error: classified(result.error) });
    this.clock.now();
  }

  async compare(expectedPackageId: string, actualPackageId: string): Promise<void> {
    this._comparisonPackageId.set(actualPackageId);
    const generation = this.generation;
    this._comparison.set({ kind: "loading" });
    const result = await this.transport.compareReplayPackages(expectedPackageId, actualPackageId);
    if (generation !== this.generation) return;
    this._comparison.set(result.ok ? { kind: "data", value: projectReplayComparison(result.value) } : { kind: "error", error: classified(result.error) });
    this.clock.now();
  }

  selectCommand(sequence: number): void {
    this._selectedCommandSequence.set(sequence);
    this.clock.now();
  }

  selectComparisonPackage(packageId: string): void {
    this._comparisonPackageId.set(packageId);
    this.clock.now();
  }

  clear(): void {
    this.generation += 1;
    this._packages.set({ kind: "idle" });
    this._review.set({ kind: "idle" });
    this._verification.set({ kind: "idle" });
    this._comparison.set({ kind: "idle" });
    this._selectedPackageId.set(null);
    this._selectedCommandSequence.set(null);
    this._comparisonPackageId.set(null);
    this.clock.now();
  }
}

const classified = (error: { readonly kind: string; readonly message: string }): ClassifiedError =>
  error.kind === "notFound"
    ? { kind: "not-found", message: error.message, retryable: false }
    : { kind: "unknown", message: error.message, retryable: false };
