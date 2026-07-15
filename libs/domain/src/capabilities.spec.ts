import { describe, expect, it } from "vitest";

import { projectCapabilityManifest } from "./capabilities";

describe("projectCapabilityManifest", () => {
  it("keeps owner support levels explicit instead of collapsing them into one boolean", () => {
    const view = projectCapabilityManifest({
      manifestId: "asha-rulebench.capabilities",
      manifestVersion: 1,
      generatedArtifactSchema: "asha-rulebench.capabilities.ts@1",
      governedAshaRevision: "0123456789abcdef",
      operationVocabularyVersion: "2",
      effectVocabularyVersion: "1",
      protocolId: "asha-rulebench.protocol",
      protocolVersion: 3,
      host: {
        adapterId: "rulebench-process-host",
        storageMode: "filesystem",
        contentStorageAdapter: "versionedFilesystem",
        replayStorageAdapter: "versionedFilesystem",
        replayRecoveryMode: "finalizedArchive",
        sessionRecoveryMode: "none",
      },
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
    expect(view.capabilities[0]?.support.supportLabel).toBe(
      "UI exposed, regression gap",
    );
    expect(view.rulesetLabels).toEqual(["ruleset.one@1"]);
  });
});
