import type {
  RulebenchReplayArchiveErrorDto,
  RulebenchReplayArchiveMetadataDto,
  RulebenchReplayComparisonReadoutDto,
  RulebenchReplayPackageReviewDto,
  RulebenchReplayVerificationReadoutDto,
} from "@asha-rulebench/protocol";

export type ReplayReviewResult<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: RulebenchReplayArchiveErrorDto };

export interface ReplayReviewTransport {
  listReplayPackages(): Promise<ReplayReviewResult<readonly RulebenchReplayArchiveMetadataDto[]>>;
  loadReplayPackage(packageId: string): Promise<ReplayReviewResult<RulebenchReplayPackageReviewDto>>;
  loadReplayVerification(packageId: string): Promise<ReplayReviewResult<RulebenchReplayVerificationReadoutDto>>;
  compareReplayPackages(expectedPackageId: string, actualPackageId: string): Promise<ReplayReviewResult<RulebenchReplayComparisonReadoutDto>>;
}

export interface ReplayReviewFixtureCatalog {
  readonly packages: readonly RulebenchReplayArchiveMetadataDto[];
  readonly reviews: Readonly<Record<string, RulebenchReplayPackageReviewDto>>;
  readonly verifications: Readonly<Record<string, RulebenchReplayVerificationReadoutDto>>;
  readonly comparisons: Readonly<Record<string, RulebenchReplayComparisonReadoutDto>>;
}

export const createReplayReviewTransport = (catalog: ReplayReviewFixtureCatalog): ReplayReviewTransport => ({
  listReplayPackages: async () => ({ ok: true, value: catalog.packages }),
  loadReplayPackage: async (packageId) => valueOrNotFound(catalog.reviews[packageId], packageId),
  loadReplayVerification: async (packageId) => valueOrNotFound(catalog.verifications[packageId], packageId),
  compareReplayPackages: async (expectedPackageId, actualPackageId) => valueOrNotFound(
    catalog.comparisons[comparisonKey(expectedPackageId, actualPackageId)], actualPackageId,
  ),
});

export const comparisonKey = (expectedPackageId: string, actualPackageId: string): string =>
  `${expectedPackageId}::${actualPackageId}`;

const valueOrNotFound = <T>(value: T | undefined, packageId: string): ReplayReviewResult<T> =>
  value === undefined
    ? { ok: false, error: { kind: "notFound", code: "unknownReplayPackage", message: `Replay package ${packageId} was not found.`, retryable: false } }
    : { ok: true, value };
