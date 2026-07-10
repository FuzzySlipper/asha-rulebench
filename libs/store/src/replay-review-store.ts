import { Injectable, signal } from "@angular/core";
import type { Signal } from "@angular/core";
import type {
  ClassifiedError,
  RulebenchReplayArchiveMetadataDto,
  RulebenchReplayComparisonReadoutDto,
  RulebenchReplayPackageReviewDto,
  RulebenchReplayVerificationReadoutDto,
} from "@asha-rulebench/protocol";
import type { ReplayReviewTransport } from "@asha-rulebench/transport";
import type { ClockPort } from "@asha-rulebench/platform";
import type { AsyncState } from "./async-state";

@Injectable()
export class ReplayReviewStore {
  private readonly _packages = signal<AsyncState<readonly RulebenchReplayArchiveMetadataDto[]>>({ kind: "idle" });
  readonly packages: Signal<AsyncState<readonly RulebenchReplayArchiveMetadataDto[]>> = this._packages.asReadonly();
  private readonly _review = signal<AsyncState<RulebenchReplayPackageReviewDto>>({ kind: "idle" });
  readonly review: Signal<AsyncState<RulebenchReplayPackageReviewDto>> = this._review.asReadonly();
  private readonly _verification = signal<AsyncState<RulebenchReplayVerificationReadoutDto>>({ kind: "idle" });
  readonly verification: Signal<AsyncState<RulebenchReplayVerificationReadoutDto>> = this._verification.asReadonly();
  private readonly _comparison = signal<AsyncState<RulebenchReplayComparisonReadoutDto>>({ kind: "idle" });
  readonly comparison: Signal<AsyncState<RulebenchReplayComparisonReadoutDto>> = this._comparison.asReadonly();

  constructor(private readonly transport: ReplayReviewTransport, private readonly clock: ClockPort) {}

  async loadPackages(): Promise<void> {
    this._packages.set({ kind: "loading" });
    const result = await this.transport.listReplayPackages();
    this._packages.set(result.ok ? { kind: "data", value: result.value } : { kind: "error", error: classified(result.error) });
    this.clock.now();
  }

  async loadReview(packageId: string): Promise<void> {
    this._review.set({ kind: "loading" });
    const result = await this.transport.loadReplayPackage(packageId);
    this._review.set(result.ok ? { kind: "data", value: result.value } : { kind: "error", error: classified(result.error) });
    this.clock.now();
  }

  async loadVerification(packageId: string): Promise<void> {
    this._verification.set({ kind: "loading" });
    const result = await this.transport.loadReplayVerification(packageId);
    this._verification.set(result.ok ? { kind: "data", value: result.value } : { kind: "error", error: classified(result.error) });
    this.clock.now();
  }

  async compare(expectedPackageId: string, actualPackageId: string): Promise<void> {
    this._comparison.set({ kind: "loading" });
    const result = await this.transport.compareReplayPackages(expectedPackageId, actualPackageId);
    this._comparison.set(result.ok ? { kind: "data", value: result.value } : { kind: "error", error: classified(result.error) });
    this.clock.now();
  }
}

const classified = (error: { readonly kind: string; readonly message: string }): ClassifiedError =>
  error.kind === "notFound"
    ? { kind: "not-found", message: error.message, retryable: false }
    : { kind: "unknown", message: error.message, retryable: false };
