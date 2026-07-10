import { describe, expect, it } from "vitest";
import type { RulebenchContentImportReadoutDto } from "@asha-rulebench/protocol";
import { projectContentImportReadout } from "./index";

describe("projectContentImportReadout", () => {
  it("formats Rust diagnostic identity without deriving compatibility semantics", () => {
    const readout: RulebenchContentImportReadoutDto = {
      exampleId: "error",
      pack: { id: "pack.error", version: "1.0.0", fingerprint: null },
      accepted: false,
      errorCount: 1,
      warningCount: 0,
      diagnostics: [
        {
          severity: "error",
          code: "missingContentPackDependency",
          path: "pack",
          referenceId: "pack.dependency",
          definitionKind: null,
          message: "Dependency is not available.",
        },
      ],
    };

    expect(projectContentImportReadout(readout)).toEqual({
      exampleId: "error",
      packLabel: "pack.error@1.0.0",
      fingerprintLabel: "Not accepted",
      statusLabel: "Rejected",
      errorCount: 1,
      warningCount: 0,
      diagnostics: [
        {
          severityLabel: "Error",
          code: "missingContentPackDependency",
          locationLabel: "pack / pack.dependency",
          message: "Dependency is not available.",
        },
      ],
    });
  });
});
