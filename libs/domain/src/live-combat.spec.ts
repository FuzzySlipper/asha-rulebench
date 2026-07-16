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
    expect(view.authoredActionBinding).toMatchObject({
      actionId: "action.binding-glyph",
      actorId: "entity-adept",
      contentPackRootLabel: "pack.authored.v3@3.0.0",
      actionFingerprintLabel: "action:exact-action",
    });
    expect(view.fingerprintLabel).toBe("test:state-1");
    expect(view.participants[1]).toEqual({
      id: "entity-raider",
      name: "Raider",
      hitPointLabel: "9/18 HP",
      temporaryVitalityLabel: null,
      statusLabel: "Active",
      conditionLabels: ["rattled"],
      position: { x: 4, y: 1 },
      coordinateLabel: "4,1",
      movementLabel: "0/0",
    });
    expect(view.options.actions[0]?.targets[0]?.id).toBe("entity-raider");
    expect(view.options.actions[0]?.targetSets[0]).toMatchObject({
      id: "set-1",
      targetIds: ["entity-raider"],
      rollPolicyLabel: "Shared",
    });
  });

  it("projects accepted and rejected command evidence from Rust fingerprints", () => {
    expect(projectLiveCommandExecution(execution(true))).toMatchObject({
      accepted: true,
      stateChanged: true,
      eventLabels: ["Damage Applied"],
      rollModeLabel: "Authority Generated",
      generatedRolls: [
        {
          purposeLabel: "Attack Roll",
          dieExpression: "1d20",
          valueLabel: "17",
        },
      ],
      targetResults: [
        {
          targetId: "entity-raider",
          damageLabel: "9 damage",
          movementLabel: "Push 4,1 → 5,1",
          resourceLabels: ["standard-action 1 → 0 (-1)"],
        },
      ],
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
    authoredActionBinding: {
      bindingVersion: "1",
      contentPackRoot: {
        id: "pack.authored.v3",
        version: "3.0.0",
        fingerprint: { algorithm: "pack", value: "exact-pack" },
      },
      contentPackReferences: [],
      contentPackSetFingerprint: { algorithm: "set", value: "exact-set" },
      actionId: "action.binding-glyph",
      actionDefinitionFingerprint: {
        algorithm: "action",
        value: "exact-action",
      },
      abilityId: "ability.binding-glyph",
      scenarioId: "scenario",
      actorId: "entity-adept",
      grant: {
        grantKind: "sessionLocalBaseAbility",
        actorId: "entity-adept",
        abilityId: "ability.binding-glyph",
      },
      targetingOperationVocabularyVersion: "2",
      checkVocabularyVersion: "1",
      effectOperationVocabularyVersion: "1",
    },
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
          targetSets: [
            {
              id: "set-1",
              targetIds: ["entity-raider"],
              targetCell: null,
              rollPolicy: "shared",
              reason: "Canonical explicit target set.",
            },
          ],
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
    gameplayFabric: {
      registryDigest: "sha256:registry",
      bindingRegistryHash: "sha256:bindings",
      moduleStateHash: "sha256:module-state",
      runtimeHostHash: "sha256:runtime-host",
      reactionFrameHashes: [],
      decisions: [],
      pendingDecisionCount: 0,
    },
    currentReactionWindow: null,
    reactionWindowLifecycleLog: [],
    reactionAuditLog: [],
    stateFingerprint: { algorithm: "test", value: fingerprint },
    actionResourceFingerprint: {
      algorithm: "test-resources",
      value: `resources-${fingerprint}`,
    },
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
        targetIds: [],
        targetCell: null,
        destinationCell: null,
        observedOrigin: null,
      },
      rolls: [],
      events: accepted
        ? [{ kind: "damageApplied", summary: "Raider took damage." }]
        : [],
      targetResults: accepted
        ? [
            {
              targetId: "entity-raider",
              accepted: true,
              reason: "Target resolved.",
              attackOutcome: "hit",
              damageAmount: 9,
              movementKind: "push",
              movementFrom: { x: 4, y: 1 },
              movementTo: { x: 5, y: 1 },
              resourceChanges: [
                {
                  resourceId: "standard-action",
                  requestedDelta: -1,
                  before: 1,
                  after: 0,
                  maximum: 1,
                },
              ],
            },
          ]
        : [],
      trace: [],
      stateBeforeFingerprint: { algorithm: "test", value: "state-0" },
      stateAfterFingerprint: {
        algorithm: "test",
        value: accepted ? "state-1" : "state-0",
      },
      rollMode: accepted ? "authorityGenerated" : "supplied",
      generatedRolls: accepted
        ? [
            {
              sequence: 0,
              commandId: "accepted",
              requestKind: "attackRoll",
              dieExpression: "1d20",
              value: 17,
              sourceMode: "authorityGenerated",
            },
          ]
        : [],
    },
    snapshot: snapshot(accepted ? 9 : 18, accepted ? "state-1" : "state-0"),
  };
}
