import type {
  RulebenchCapabilityEntryDto,
  RulebenchCapabilityKindDto,
  RulebenchCapabilityManifestDto,
  RulebenchCapabilitySupportDto,
} from "@asha-rulebench/protocol";

export interface RulebenchCapabilitySupportView {
  readonly declared: boolean;
  readonly validationSupported: boolean;
  readonly runtimeExecutable: boolean;
  readonly protocolExposed: boolean;
  readonly liveHostExposed: boolean;
  readonly uiExposed: boolean;
  readonly regressionCovered: boolean;
  readonly durableAcrossRestart: boolean;
  readonly supportLabel: string;
}

export interface RulebenchCapabilityEntryView {
  readonly id: string;
  readonly kind: RulebenchCapabilityKindDto;
  readonly kindLabel: string;
  readonly version: string;
  readonly support: RulebenchCapabilitySupportView;
  readonly evidence: readonly string[];
}

export interface RulebenchRulesetProviderView {
  readonly providerLabel: string;
  readonly rulesetLabel: string;
  readonly compatibilityLabel: string;
  readonly capabilityCount: number;
}

export interface RulebenchCapabilityManifestView {
  readonly manifestId: string;
  readonly manifestVersion: number;
  readonly generatedArtifactSchema: string;
  readonly governedAshaRevision: string;
  readonly operationVocabularyVersion: string;
  readonly effectVocabularyVersion: string;
  readonly protocolLabel: string;
  readonly hostLabel: string;
  readonly recoveryLabel: string;
  readonly providers: readonly RulebenchRulesetProviderView[];
  readonly rulesetLabels: readonly string[];
  readonly packageLabels: readonly string[];
  readonly scenarioCount: number;
  readonly capabilities: readonly RulebenchCapabilityEntryView[];
}

export function projectCapabilityManifest(
  dto: RulebenchCapabilityManifestDto,
): RulebenchCapabilityManifestView {
  return {
    manifestId: dto.manifestId,
    manifestVersion: dto.manifestVersion,
    generatedArtifactSchema: dto.generatedArtifactSchema,
    governedAshaRevision: dto.governedAshaRevision,
    operationVocabularyVersion: dto.operationVocabularyVersion,
    effectVocabularyVersion: dto.effectVocabularyVersion,
    protocolLabel: `${dto.protocolId} v${dto.protocolVersion}`,
    hostLabel: `${dto.host.adapterId} · ${dto.host.storageMode}`,
    recoveryLabel: `Replay: ${dto.host.replayRecoveryMode}; session: ${dto.host.sessionRecoveryMode}`,
    providers: dto.providers.map((provider) => ({
      providerLabel: identityLabel(provider.provider),
      rulesetLabel: identityLabel(provider.ruleset),
      compatibilityLabel: `pipeline ${provider.operationVocabularyVersion} · effects ${provider.effectOperationVocabularyVersion}`,
      capabilityCount: provider.capabilities.length,
    })),
    rulesetLabels: dto.rulesets.map(identityLabel),
    packageLabels: dto.packages.map(identityLabel),
    scenarioCount: dto.scenarios.length,
    capabilities: dto.capabilities.map(projectCapabilityEntry),
  };
}

function projectCapabilityEntry(
  dto: RulebenchCapabilityEntryDto,
): RulebenchCapabilityEntryView {
  return {
    id: dto.id,
    kind: dto.kind,
    kindLabel: capabilityKindLabel(dto.kind),
    version: dto.version,
    support: {
      ...dto.support,
      supportLabel: supportLabel(dto.support),
    },
    evidence: dto.evidence,
  };
}

function identityLabel(identity: Readonly<{ id: string; version: string }>): string {
  return `${identity.id}@${identity.version}`;
}

function capabilityKindLabel(kind: RulebenchCapabilityKindDto): string {
  switch (kind) {
    case "operation":
      return "Operation";
    case "targeting":
      return "Targeting";
    case "policy":
      return "Policy";
    case "content":
      return "Content";
    case "replay":
      return "Replay";
    case "session":
      return "Session";
  }
}

function supportLabel(support: RulebenchCapabilitySupportDto): string {
  if (!support.declared) return "Not declared";
  if (!support.validationSupported) return "Declared only";
  if (!support.runtimeExecutable) return "Validated, not executable";
  if (!support.protocolExposed) return "Rust runtime only";
  if (!support.liveHostExposed) return "Protocol only";
  if (!support.uiExposed) return "Live host only";
  if (!support.regressionCovered) return "UI exposed, regression gap";
  return support.durableAcrossRestart
    ? "Durable and regression covered"
    : "Regression covered, not restart durable";
}
