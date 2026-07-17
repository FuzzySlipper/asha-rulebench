import { canonicalRpgJson, normalizePackage } from "@asha-rpg/authoring";

import {
  rulebenchActionBindings,
  rulebenchAuthoredRpgPackage,
} from "../../libs/content-authoring/src/representative-rpg-content.ts";

export const RPG_CONTENT_ARTIFACT_SCHEMA = "asha-rulebench.rpg-content@1";

export function renderRpgContentArtifact() {
  const result = normalizePackage(rulebenchAuthoredRpgPackage);
  if (!result.ok) {
    const diagnostics = result.diagnostics
      .map(
        (diagnostic) =>
          `${diagnostic.code} ${diagnostic.path}: ${diagnostic.message}`,
      )
      .join("\n");
    throw new Error(`Rulebench RPG content did not normalize:\n${diagnostics}`);
  }

  const normalizedIr = JSON.parse(canonicalRpgJson(result.artifact));
  return `${JSON.stringify(
    {
      _generated: {
        emitter: "libs/content-authoring",
        schema: RPG_CONTENT_ARTIFACT_SCHEMA,
      },
      normalizedIr,
      bindings: rulebenchActionBindings,
    },
    null,
    2,
  )}\n`;
}
