import { describe, expect, it } from "vitest";

import { projectCapabilityManifest } from "./capabilities";

describe("projectCapabilityManifest", () => {
  it("keeps owner support levels explicit instead of collapsing them into one boolean", () => {
    const view = projectCapabilityManifest({
      manifestId: "asha-rulebench.capabilities",
      manifestVersion: 4,
      generatedArtifactSchema: "asha-rulebench.capabilities.ts@4",
      governedAshaRevision: "0123456789abcdef",
      operationVocabularyVersion: "2",
      effectVocabularyVersion: "1",
      protocolId: "asha-rulebench.protocol",
      protocolVersion: 5,
      host: {
        adapterId: "rulebench-process-host",
        storageMode: "filesystem",
        contentStorageAdapter: "versionedFilesystem",
        replayStorageAdapter: "versionedFilesystem",
        replayRecoveryMode: "finalizedArchive",
        sessionRecoveryMode: "none",
        authorityViewerMode: "liveAuthorityReadback",
      },
      providers: [
        {
          provider: { id: "provider.one", version: "1" },
          ruleset: { id: "ruleset.one", version: "1" },
          operationVocabularyVersion: "2",
          effectOperationVocabularyVersion: "1",
          capabilities: [{ id: "operation.damage", version: "1" }],
        },
      ],
      rulesets: [{ id: "ruleset.one", version: "1" }],
      packages: [{ id: "package.one", version: "1" }],
      scenarios: [{ id: "scenario.one", version: "registered" }],
      capabilities: [
        {
          id: "operation.damage",
          kind: "operation",
          version: "1",
          support: {
            declared: true,
            validationSupported: true,
            runtimeExecutable: true,
            protocolExposed: true,
            liveHostExposed: true,
            uiExposed: true,
            regressionCovered: false,
            durableAcrossRestart: true,
          },
          evidence: ["rulebench-combat.runtime-effect-operation-registry"],
        },
      ],
    });

    expect(view.hostLabel).toBe("rulebench-process-host · filesystem");
    expect(view.authorityViewerLabel).toBe(
      "Live authority readback; no checked-artifact fallback",
    );
    expect(view.capabilities[0]?.support.supportLabel).toBe(
      "UI exposed, regression gap",
    );
    expect(view.rulesetLabels).toEqual(["ruleset.one@1"]);
    expect(view.providers).toEqual([
      {
        providerLabel: "provider.one@1",
        rulesetLabel: "ruleset.one@1",
        compatibilityLabel: "pipeline 2 · effects 1",
        capabilityCount: 1,
      },
    ]);
  });
});
