import { normalizePackage } from "@asha-rpg/authoring";
import { describe, expect, it } from "vitest";

import {
  rulebenchActionBindings,
  rulebenchAuthoredRpgPackage,
} from "./representative-rpg-content.js";

describe("representative RPG content", () => {
  it("normalizes the downstream corpus without product authority callbacks", () => {
    const result = normalizePackage(rulebenchAuthoredRpgPackage);

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.artifact.actions.map((action) => action.id)).toEqual(
      rulebenchActionBindings
        .map((binding) => binding.actionId)
        .slice()
        .sort(),
    );
    expect(JSON.stringify(result.artifact)).not.toContain("function");
  });

  it("keeps product reaction orchestration outside the portable semantic artifact", () => {
    const reactionBindings = rulebenchActionBindings.filter(
      (binding) => binding.reaction !== null,
    );

    expect(reactionBindings.map((binding) => binding.actionId)).toEqual([
      "hexing_bolt",
    ]);
  });
});
