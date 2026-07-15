import type {
  RulebenchContentAuditEntryDto,
  RulebenchContentDefinitionChangeDto,
  RulebenchContentImportAttemptDto,
  RulebenchContentImportDiagnosticDto,
  RulebenchContentPackDiffDto,
  RulebenchContentPackReferenceDto,
  RulebenchContentPackReviewDto,
  RulebenchContentWorkspaceDto,
  RulebenchStoredContentPackSummaryDto,
} from "@asha-rulebench/protocol";

export interface RulebenchContentReferenceView {
  readonly reference: RulebenchContentPackReferenceDto;
  readonly identityLabel: string;
  readonly fingerprintLabel: string;
}

export interface RulebenchStoredContentPackView
  extends RulebenchContentReferenceView {
  readonly title: string;
  readonly summary: string;
  readonly active: boolean;
  readonly statusLabel: string;
  readonly provenanceLabel: string;
  readonly rulesetLabel: string;
  readonly dependencies: readonly RulebenchContentReferenceView[];
  readonly definitions: readonly {
    readonly kind: string;
    readonly id: string;
  }[];
}

export interface RulebenchContentAuditView {
  readonly sequenceLabel: string;
  readonly operationLabel: string;
  readonly packLabel: string;
  readonly detail: string;
}

export interface RulebenchContentWorkspaceView {
  readonly packs: readonly RulebenchStoredContentPackView[];
  readonly audit: readonly RulebenchContentAuditView[];
}

export interface RulebenchContentDiagnosticView {
  readonly severityLabel: string;
  readonly code: string;
  readonly locationLabel: string;
  readonly message: string;
}

export interface RulebenchContentImportAttemptView {
  readonly accepted: boolean;
  readonly statusLabel: string;
  readonly packLabel: string;
  readonly errorMessage: string | null;
  readonly diagnostics: readonly RulebenchContentDiagnosticView[];
  readonly review: RulebenchContentPackReviewView | null;
}

export interface RulebenchContentPackReviewView {
  readonly pack: RulebenchStoredContentPackView;
  readonly authoredPayload: string;
  readonly diagnostics: readonly RulebenchContentDiagnosticView[];
}

export interface RulebenchContentDiffView {
  readonly beforeLabel: string;
  readonly afterLabel: string;
  readonly changed: boolean;
  readonly summaryLabel: string;
  readonly metadataChanges: readonly string[];
  readonly definitionChanges: readonly RulebenchContentDefinitionChangeDto[];
}

export function projectContentWorkspace(
  workspace: RulebenchContentWorkspaceDto,
): RulebenchContentWorkspaceView {
  return {
    packs: workspace.packs.map(projectStoredPack),
    audit: workspace.audit.map(projectAudit),
  };
}

export function projectContentImportAttempt(
  attempt: RulebenchContentImportAttemptDto,
): RulebenchContentImportAttemptView {
  return {
    accepted: attempt.accepted,
    statusLabel: attempt.accepted ? "Accepted" : "Rejected",
    packLabel: `${attempt.pack.id}@${attempt.pack.version}`,
    errorMessage: attempt.errorMessage,
    diagnostics: attempt.diagnostics.map(projectDiagnostic),
    review:
      attempt.outcome === null
        ? null
        : projectContentPackReview(attempt.outcome.review),
  };
}

export function projectContentPackReview(
  review: RulebenchContentPackReviewDto,
): RulebenchContentPackReviewView {
  return {
    pack: projectStoredPack(review.pack),
    authoredPayload: review.authoredPayload,
    diagnostics: review.diagnostics.map(projectDiagnostic),
  };
}

export function projectContentDiff(
  diff: RulebenchContentPackDiffDto,
): RulebenchContentDiffView {
  const metadataCount = diff.metadataChanges.length;
  const definitionCount = diff.definitionChanges.length;
  return {
    beforeLabel: referenceLabel(diff.before),
    afterLabel: referenceLabel(diff.after),
    changed: diff.changed,
    summaryLabel: diff.changed
      ? `${metadataCount} metadata changes · ${definitionCount} definition changes`
      : "No canonical changes",
    metadataChanges: diff.metadataChanges,
    definitionChanges: diff.definitionChanges,
  };
}

function projectStoredPack(
  pack: RulebenchStoredContentPackSummaryDto,
): RulebenchStoredContentPackView {
  const reference = projectReference(pack.reference);
  return {
    ...reference,
    title: pack.title,
    summary: pack.summary,
    active: pack.active,
    statusLabel: pack.active ? "Active" : "Inactive",
    provenanceLabel: `${pack.sourceKind} · ${pack.sourceId}${pack.authoredBy === null ? "" : ` · ${pack.authoredBy}`}`,
    rulesetLabel: `${pack.rulesetId}@${pack.rulesetVersion}`,
    dependencies: pack.dependencies.map(projectReference),
    definitions: pack.definitions,
  };
}

function projectReference(
  reference: RulebenchContentPackReferenceDto,
): RulebenchContentReferenceView {
  return {
    reference,
    identityLabel: `${reference.id}@${reference.version}`,
    fingerprintLabel: `${reference.fingerprint.algorithm}:${reference.fingerprint.value}`,
  };
}

function referenceLabel(reference: RulebenchContentPackReferenceDto): string {
  return `${reference.id}@${reference.version}#${reference.fingerprint.value}`;
}

function projectAudit(entry: RulebenchContentAuditEntryDto): RulebenchContentAuditView {
  return {
    sequenceLabel: `#${entry.sequence}`,
    operationLabel: humanize(entry.operation),
    packLabel: referenceLabel(entry.reference),
    detail: entry.detail,
  };
}

function projectDiagnostic(
  diagnostic: RulebenchContentImportDiagnosticDto,
): RulebenchContentDiagnosticView {
  const definition =
    diagnostic.definitionKind === null ? "" : ` / ${diagnostic.definitionKind}`;
  const reference =
    diagnostic.referenceId === null ? "" : ` / ${diagnostic.referenceId}`;
  return {
    severityLabel: diagnostic.severity === "error" ? "Error" : "Warning",
    code: diagnostic.code,
    locationLabel: `${diagnostic.path}${definition}${reference}`,
    message: diagnostic.message,
  };
}

function humanize(value: string): string {
  return value.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/^./, (letter) =>
    letter.toUpperCase(),
  );
}
