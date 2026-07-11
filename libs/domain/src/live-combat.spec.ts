import { describe, expect, it } from "vitest";
import type {
  RulebenchLiveCommandExecutionDto,
  RulebenchLiveSessionSnapshotDto,
} from "@asha-rulebench/protocol";
import {
  projectLiveCommandExecution,
  projectLiveSessionSnapshot,
} from "./live-combat";

describe("live combat domain projections", () => {
  it("maps authority snapshot evidence into display labels without changing facts", () => {
    const view = projectLiveSessionSnapshot(snapshot(9, "state-1"));

    expect(view.lifecycleLabel).toBe("In Progress");
    expect(view.fingerprintLabel).toBe("test:state-1");
    expect(view.participants[1]).toEqual({
      id: "entity-raider",
      name: "Raider",
      hitPointLabel: "9/18 HP",
      temporaryVitalityLabel: null,
      statusLabel: "Active",
      conditionLabels: ["rattled"],
      position: { x: 4, y: 1 },
      movementLabel: "0/0",
    });
    expect(view.options.actions[0]?.targets[0]?.id).toBe("entity-raider");
  });

  it("projects accepted and rejected command evidence from Rust fingerprints", () => {
    expect(projectLiveCommandExecution(execution(true))).toMatchObject({
      accepted: true,
      stateChanged: true,
      eventLabels: ["Damage Applied"],
    });
    expect(projectLiveCommandExecution(execution(false))).toMatchObject({
      accepted: false,
      stateChanged: false,
      rejectionLabel: "Invalid Target",
    });
  });
});

function snapshot(
  raiderHitPoints = 18,
  fingerprint = "state-0",
): RulebenchLiveSessionSnapshotDto {
  return {
    sessionId: "live-session",
    nextStepIndex: 0,
    lifecyclePhase: "inProgress",
    startedAtStep: 0,
    endedAtStep: null,
    roundNumber: 1,
    turnIndex: 0,
    participantOrder: ["entity-adept", "entity-raider"],
    currentActorId: "entity-adept",
    participants: [
      {
        id: "entity-adept",
        name: "Adept",
        currentHitPoints: 24,
        maxHitPoints: 24,
        temporaryVitality: 0,
        defeated: false,
        conditions: [],
        position: { x: 1, y: 1 },
        movementRemaining: 0,
        movementMaximum: 0,
      },
      {
        id: "entity-raider",
        name: "Raider",
        currentHitPoints: raiderHitPoints,
        maxHitPoints: 18,
        temporaryVitality: 0,
        defeated: false,
        conditions: raiderHitPoints < 18 ? ["rattled"] : [],
        position: { x: 4, y: 1 },
        movementRemaining: 0,
        movementMaximum: 0,
      },
    ],
    board: {
      id: "two-combatant-hexing-bolt",
      width: 6,
      height: 4,
      cells: [
        {
          position: { x: 1, y: 1 },
          terrainTags: ["clear"],
          blocksMovement: false,
          occupantIds: ["entity-adept"],
        },
      ],
    },
    options: {
      roundNumber: 1,
      turnIndex: 0,
      lifecyclePhase: "inProgress",
      currentActorId: "entity-adept",
      currentActorDefeated: false,
      available: true,
      unavailableReason: null,
      actions: [
        {
          actionId: "hexing_bolt",
          abilityId: "ability.hexing-bolt",
          actionName: "Hexing Bolt",
          available: true,
          unavailableReason: null,
          resourceCosts: [],
          resourceStates: [],
          targetMode: "entity",
          destinations: [],
          targets: [
            {
              targetId: "entity-raider",
              targetName: "Raider",
              currentHitPoints: raiderHitPoints,
              maxHitPoints: 18,
              reason: "Target is legal.",
            },
          ],
        },
      ],
    },
    combatEnd: {
      shouldEnd: false,
      conditionKind: "ongoing",
      outcomeKind: "ongoing",
      activeSides: ["ally", "enemy"],
      defeatedSides: [],
      winningSides: [],
      reason: "Combat continues.",
    },
    finalization: null,
    combatLog: [],
    auditLog: [],
    stateFingerprint: { algorithm: "test", value: fingerprint },
  };
}

function execution(accepted: boolean): RulebenchLiveCommandExecutionDto {
  return {
    step: {
      sessionId: "live-session",
      stepId: accepted ? "accepted" : "rejected",
      stepIndex: 0,
      title: "Command",
      summary: "Authority result.",
      outcomeClass: accepted ? "acceptedHit" : "rejectedInvalidCommand",
      accepted,
      decisionKind: accepted ? "acceptedByResolver" : "rejectedByPreflight",
      rejectionCode: accepted ? null : "invalidTarget",
      intent: {
        actorId: "entity-adept",
        actionId: "hexing_bolt",
        targetId: "entity-raider",
        destinationCell: null,
      },
      rolls: [],
      events: accepted
        ? [{ kind: "damageApplied", summary: "Raider took damage." }]
        : [],
      trace: [],
      stateBeforeFingerprint: { algorithm: "test", value: "state-0" },
      stateAfterFingerprint: {
        algorithm: "test",
        value: accepted ? "state-1" : "state-0",
      },
    },
    snapshot: snapshot(accepted ? 9 : 18, accepted ? "state-1" : "state-0"),
  };
}
