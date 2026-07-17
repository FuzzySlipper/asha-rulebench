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
