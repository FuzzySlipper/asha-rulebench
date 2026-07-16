import type {
  RulebenchReplayArchiveMetadataDto,
  RulebenchReplayComparisonReadoutDto,
  RulebenchReplayPackageReviewDto,
  RulebenchReplayStepEvidenceDto,
  RulebenchReplayVerificationReadoutDto,
} from "@asha-rulebench/protocol";
import { projectLiveSessionSnapshot, type RulebenchLiveSessionView } from "./live-combat";

export interface RulebenchReplayArchiveItemView {
  readonly packageId: string;
  readonly sessionLabel: string;
  readonly scenarioLabel: string;
  readonly rulesetLabel: string;
  readonly completedLabel: string;
}

export interface RulebenchReplayReviewView {
  readonly packageId: string;
  readonly title: string;
  readonly summary: string;
  readonly provenanceLabel: string;
  readonly contentPackRootLabel: string | null;
  readonly contentPackSetFingerprintLabel: string | null;
  readonly contentPackReferenceLabels: readonly string[];
  readonly authoredActionBindingLabel: string | null;
  readonly finalFingerprintLabel: string;
  readonly commands: readonly RulebenchReplayCommandView[];
}

export interface RulebenchReplayCommandView {
  readonly sequence: number;
  readonly sequenceLabel: string;
  readonly id: string;
  readonly commandKindLabel: string;
  readonly summary: string;
  readonly suppliedRollLabel: string;
  readonly expected: RulebenchReplayEvidenceView;
  readonly actual: RulebenchReplayEvidenceView;
  readonly snapshot: RulebenchLiveSessionView;
}

export interface RulebenchReplayEvidenceView {
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly stateBeforeLabel: string;
  readonly stateAfterLabel: string;
  readonly eventLabels: readonly string[];
  readonly auditLabels: readonly string[];
  readonly rollLabels: readonly string[];
  readonly trace: readonly RulebenchReplayTraceView[];
}

export interface RulebenchReplayTraceView {
  readonly sequenceLabel: string;
  readonly phaseLabel: string;
  readonly statusLabel: string;
  readonly message: string;
  readonly detail: string;
}

export interface RulebenchReplayVerificationView {
  readonly accepted: boolean;
  readonly decisionLabel: string;
  readonly verifiedStepLabel: string;
  readonly finalizedLabel: string;
  readonly mismatchLabel: string | null;
  readonly fingerprintLabel: string | null;
}

export interface RulebenchReplayComparisonView {
  readonly matches: boolean;
  readonly resultLabel: string;
  readonly packageLabel: string;
  readonly comparedCommandLabel: string;
  readonly firstDifference: RulebenchReplayDifferenceView | null;
  readonly differences: readonly RulebenchReplayDifferenceView[];
}

export interface RulebenchReplayDifferenceView {
  readonly codeLabel: string;
  readonly path: string;
  readonly commandLabel: string;
  readonly expectedSummary: string;
  readonly actualSummary: string;
}

export function projectReplayArchiveItem(
  item: RulebenchReplayArchiveMetadataDto,
): RulebenchReplayArchiveItemView {
  return {
    packageId: item.packageId,
    sessionLabel: item.sessionId,
    scenarioLabel: item.scenarioId,
    rulesetLabel: `${item.rulesetId} · ${item.rulesetVersion}`,
    completedLabel: item.completedAt,
  };
}

export function projectReplayReview(
  review: RulebenchReplayPackageReviewDto,
): RulebenchReplayReviewView {
  return {
    packageId: review.packageId,
    title: review.narrationTitle ?? review.packageId,
    summary: review.narrationSummary ?? "No replay narration supplied.",
    provenanceLabel: `${review.scenarioId} · ${review.rulesetId} ${review.rulesetVersion} · package ${review.packageVersion}`,
    contentPackRootLabel:
      review.contentPackRoot === null
        ? null
        : contentPackReferenceLabel(review.contentPackRoot),
    contentPackSetFingerprintLabel:
      review.contentPackSetFingerprint === null
        ? null
        : `${review.contentPackSetFingerprint.algorithm}:${review.contentPackSetFingerprint.value}`,
    contentPackReferenceLabels: review.contentPackReferences.map(
      contentPackReferenceLabel,
    ),
    authoredActionBindingLabel:
      review.authoredActionBinding === null
        ? null
        : `${review.authoredActionBinding.actionId} · ${review.authoredActionBinding.actorId} · ${review.authoredActionBinding.actionDefinitionFingerprint.algorithm}:${review.authoredActionBinding.actionDefinitionFingerprint.value}`,
    finalFingerprintLabel: `${review.finalStateFingerprint.algorithm}:${review.finalStateFingerprint.value}`,
    commands: review.commands.map((command) => ({
      sequence: command.sequence,
      sequenceLabel: String(command.sequence + 1),
      id: command.id,
      commandKindLabel: labelCode(command.commandKind),
      summary: command.narrationSummary ?? command.id,
      suppliedRollLabel:
        command.suppliedRollStream.length === 0
          ? "No supplied rolls"
          : command.suppliedRollStream.join(", "),
      expected: projectEvidence(command.expected),
      actual: projectEvidence(command.actual),
      snapshot: projectLiveSessionSnapshot(command.snapshot),
    })),
  };
}

function contentPackReferenceLabel(
  reference: RulebenchReplayPackageReviewDto["contentPackReferences"][number],
): string {
  return `${reference.id}@${reference.version} · ${reference.fingerprint.algorithm}:${reference.fingerprint.value}`;
}

export function projectReplayVerification(
  verification: RulebenchReplayVerificationReadoutDto,
): RulebenchReplayVerificationView {
  return {
    accepted: verification.accepted,
    decisionLabel: labelCode(verification.decisionKind),
    verifiedStepLabel: `${verification.verifiedStepCount} steps verified`,
    finalizedLabel: verification.finalized ? "Finalized" : "Not finalized",
    mismatchLabel:
      verification.mismatch === null
        ? null
        : `${labelCode(verification.mismatch.dimension)} · ${verification.mismatch.reason}`,
    fingerprintLabel:
      verification.finalStateFingerprint === null
        ? null
        : `${verification.finalStateFingerprint.algorithm}:${verification.finalStateFingerprint.value}`,
  };
}

export function projectReplayComparison(
  comparison: RulebenchReplayComparisonReadoutDto,
): RulebenchReplayComparisonView {
  return {
    matches: comparison.matches,
    resultLabel: comparison.matches ? "Matches" : "Differences found",
    packageLabel: `${comparison.expectedPackageId} vs ${comparison.actualPackageId}`,
    comparedCommandLabel: `${comparison.comparedCommandCount} commands compared`,
    firstDifference:
      comparison.firstDifference === null
        ? null
        : projectDifference(comparison.firstDifference),
    differences: comparison.differences.map(projectDifference),
  };
}

function projectEvidence(
  evidence: RulebenchReplayStepEvidenceDto,
): RulebenchReplayEvidenceView {
  return {
    accepted: evidence.accepted,
    decisionLabel: labelCode(evidence.decisionCode),
    stateBeforeLabel: `${evidence.stateBeforeFingerprint.algorithm}:${evidence.stateBeforeFingerprint.value}`,
    stateAfterLabel: `${evidence.stateAfterFingerprint.algorithm}:${evidence.stateAfterFingerprint.value}`,
    eventLabels: evidence.acceptedEvents.map(
      (event) => `${labelCode(event.kind)} · ${event.summary}`,
    ),
    auditLabels: evidence.commandAudit.map(
      (entry) =>
        `${entry.sequence + 1} · ${labelCode(entry.decisionKind)} · ${entry.eventCount} events · ${entry.traceCount} trace entries`,
    ),
    rollLabels: evidence.rolls.map(
      (roll) =>
        `${roll.sequence + 1} · ${labelCode(roll.requestKind)} · ${roll.suppliedValue ?? "none"} · ${roll.consumed ? "consumed" : "unused"}`,
    ),
    trace: evidence.trace.map((entry) => ({
      sequenceLabel: String(entry.sequence + 1),
      phaseLabel: labelCode(entry.phase),
      statusLabel: labelCode(entry.status),
      message: entry.message,
      detail: entry.detail,
    })),
  };
}

function projectDifference(
  difference: RulebenchReplayComparisonReadoutDto["differences"][number],
): RulebenchReplayDifferenceView {
  return {
    codeLabel: labelCode(difference.code),
    path: difference.path,
    commandLabel:
      difference.commandSequence === null
        ? "Package"
        : `Command ${difference.commandSequence + 1} · ${difference.commandId ?? "unknown"}`,
    expectedSummary: difference.expectedSummary,
    actualSummary: difference.actualSummary,
  };
}

function labelCode(value: string): string {
  return value
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/^./, (character) => character.toUpperCase());
}
