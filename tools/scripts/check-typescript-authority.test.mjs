import assert from "node:assert/strict";
import test from "node:test";
import { inspectTypeScriptAuthority } from "./check-typescript-authority.mjs";

test("TypeScript authority check accepts DTO projection and roll transport", () => {
  const source = `
    export function project(receipt: Receipt) {
      return {
        accepted: receipt.accepted,
        outcome: receipt.outcome === null ? null : receipt.outcome,
        damage: receipt.damage?.amount ?? null,
      };
    }
    export function parseRoll(value: string) {
      const damage = Number(value);
      return damage;
    }
  `;

  assert.deepEqual(inspectTypeScriptAuthority(source), []);
});

test("TypeScript authority check rejects client-side dice and semantic resolvers", () => {
  const source = `
    const roll = Math.floor(Math.random() * 20) + 1;
    const result = resolveAttack(roll, defense);
  `;

  const diagnostics = inspectTypeScriptAuthority(source);
  assert.ok(
    diagnostics.some((entry) => entry.includes("authority randomness")),
  );
  assert.ok(diagnostics.some((entry) => entry.includes("resolveAttack")));
});

test("TypeScript authority check rejects derived outcomes and damage", () => {
  const source = `
    const accepted = attackTotal >= defense;
    const damage = roll + modifier;
    return { accepted, damage: target.hitPoints.current - damage };
  `;

  const diagnostics = inspectTypeScriptAuthority(source);
  assert.ok(
    diagnostics.some((entry) => entry.includes("authoritative accepted")),
  );
  assert.ok(
    diagnostics.some((entry) => entry.includes("authoritative damage")),
  );
});

test("TypeScript authority check rejects returned gameplay-state mutation", () => {
  const source = `
    state.participants[0].hitPoints.current -= damage;
    state.accepted = true;
  `;

  const diagnostics = inspectTypeScriptAuthority(source);
  assert.equal(diagnostics.length, 2);
  assert.ok(diagnostics.every((entry) => entry.includes("may not mutate")));
});

test("content authoring accepts a pure combinator over published builders", () => {
  const source = `
    import { applyModifier, damage, sequence } from "@asha-rpg/authoring";
    import type { RpgIrFormula } from "@asha-rpg/ir";
    export const paired = (amount: RpgIrFormula) => sequence(
      damage({ amount, type: damageType("force") }),
      applyModifier(modifierOptions),
    );
  `;

  assert.deepEqual(
    inspectTypeScriptAuthority(
      source,
      "libs/content-authoring/src/pure-combinator.ts",
    ),
    [],
  );
});

test("content authoring rejects product imports callbacks capability stores and browser access", () => {
  const source = `
    import { store } from "@asha-rulebench/store";
    const operation = { execute: (gameplayContext) => gameplayContext.hp -= rollDice(6) };
    capabilityStore.get("vitality");
    window.fetch("/authority");
  `;

  const diagnostics = inspectTypeScriptAuthority(
    source,
    "libs/content-authoring/src/forbidden.ts",
  );
  assert.ok(
    diagnostics.some((entry) => entry.includes("@asha-rulebench/store")),
  );
  assert.ok(
    diagnostics.some((entry) => entry.includes("semantic callback execute")),
  );
  assert.ok(diagnostics.some((entry) => entry.includes("capabilityStore")));
  assert.ok(diagnostics.some((entry) => entry.includes("window")));
  assert.ok(diagnostics.some((entry) => entry.includes("rollDice")));
});

test("RPG policy accepts typed intent proposals and rejects authority-store access", () => {
  const accepted = `
    import type { RpgActionId } from "@asha-rpg/ir";
    import type { LiveSessionSnapshotDto } from "@asha-rulebench/protocol";
    export const proposeIntent = (view: LiveSessionSnapshotDto, actionId: RpgActionId) => ({
      actorId: view.currentActorId,
      actionId,
    });
  `;
  assert.deepEqual(
    inspectTypeScriptAuthority(accepted, "libs/rpg-policy/src/proposal.ts"),
    [],
  );

  const rejected = `
    import { store } from "@asha-rulebench/store";
    capabilityStore.get("vitality");
    resolveAttack(intent, authorityState);
  `;
  const diagnostics = inspectTypeScriptAuthority(
    rejected,
    "libs/rpg-policy/src/forbidden.ts",
  );
  assert.ok(
    diagnostics.some((entry) => entry.includes("@asha-rulebench/store")),
  );
  assert.ok(diagnostics.some((entry) => entry.includes("capabilityStore")));
  assert.ok(diagnostics.some((entry) => entry.includes("resolveAttack")));
});
