import { describe, expect, it } from "vitest";
import {
  createFakeRulebenchTransport,
  defaultCombatAutomaticRunReplayReadout,
  defaultCombatAutomaticRunReadout,
  defaultCombatControlHistoryReadout,
  defaultCombatScriptReadout,
  defaultCombatSessionCatalog,
  defaultCombatSessionStepReadout,
  defaultContentValidationCatalog,
  defaultContentValidationReport,
  defaultContentImportCatalog,
  defaultRulesetCatalog,
  defaultScenarioCatalog,
  defaultScenarioReadout,
} from "./index";
import { rustBackedCombatSessionCatalog } from "./generated/rust-combat-session";
import {
  rustBackedContentImportCatalog,
  rustBackedContentValidationCatalog,
  rustBackedRulesetCatalog,
  rustBackedScenarioCatalog,
} from "./generated/rust-scenario-catalog";

describe("RulebenchTransport fixtures", () => {
  it("delivers generated Rust content import outcomes without TS validation", async () => {
    expect(defaultContentImportCatalog).toBe(rustBackedContentImportCatalog);
    const result = await createFakeRulebenchTransport().loadContentImportExamples();

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.map((example) => example.exampleId)).toEqual([
        "content-import-valid",
        "content-import-warning",
        "content-import-error",
      ]);
      expect(result.value[2]?.diagnostics[0]?.code).toBe(
        "missingContentPackDependency",
      );
    }
  });

  it("uses the checked Rust-backed ruleset catalog as the default transport payload", async () => {
    expect(defaultRulesetCatalog).toBe(rustBackedRulesetCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadRulesetCatalog();

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultRulesetCatalog);
      expect(result.value.selectedRulesetId).toBe(
        "asha-rulebench.hexing-bolt.v0",
      );
      expect(result.value.rulesets).toEqual([
        {
          id: "asha-rulebench.hexing-bolt.v0",
          name: "Hexing Bolt Fixture Rules",
          version: "0.0.0",
          summary:
            "Local single-action fixture ruleset for authority incubation.",
        },
        {
          id: "asha-rulebench.turn-control.v0",
          name: "Turn Control Fixture Rules",
          version: "0.0.0",
          summary:
            "Minimal second ruleset proving static turn-control module selection.",
        },
      ]);
    }
  });

  it("uses the checked Rust-backed content validation catalog as the default transport payload", async () => {
    expect(defaultContentValidationCatalog).toBe(
      rustBackedContentValidationCatalog,
    );

    const transport = createFakeRulebenchTransport();
    const result =
      await transport.loadContentValidationReport("hexing-bolt-hit");

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultContentValidationReport);
      expect(result.value.scenarioId).toBe("hexing-bolt-hit");
      expect(result.value.scenarioTitle).toBe("Hexing Bolt Hit");
      expect(result.value.report).toEqual({
        accepted: true,
        errorCount: 0,
        warningCount: 0,
        diagnostics: [],
      });
    }
  });

  it("reads generated invalid Rust content validation diagnostics through transport", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadContentValidationReport(
      "hexing-bolt-invalid-selected-ability",
    );

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.scenarioId).toBe(
        "hexing-bolt-invalid-selected-ability",
      );
      expect(result.value.scenarioTitle).toBe(
        "Hexing Bolt Invalid Selected Ability",
      );
      expect(result.value.report).toEqual({
        accepted: false,
        errorCount: 1,
        warningCount: 0,
        diagnostics: [
          {
            severity: "error",
            code: "selectedAbilityMissingFromCatalog",
            contentId: "ability.missing",
            message:
              "Selected ability ability.missing is not present in the scenario ability catalog.",
          },
        ],
      });
    }
  });

  it("uses the checked Rust-backed scenario catalog as the default transport payload", async () => {
    expect(defaultScenarioCatalog).toBe(rustBackedScenarioCatalog);

    const transport = createFakeRulebenchTransport();
    const catalogResult = await transport.loadCatalog();
    const result = await transport.loadScenario("hexing-bolt-hit");

    expect(catalogResult.ok).toBe(true);
    if (catalogResult.ok) {
      expect(catalogResult.value.map((summary) => summary.id)).toEqual([
        "hexing-bolt-hit",
        "hexing-bolt-miss",
        "hexing-bolt-self-target-rejected",
        "turn-control-hit",
        "hexing-bolt-veteran-hit",
      ]);
    }
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultScenarioReadout);
      expect(result.value.selectedAction.attack.modifier).toBe(4);
      expect(result.value.domainEvents.map((event) => event.type)).toEqual([
        "ActionUsed",
        "AttackRolled",
        "DamageApplied",
        "ModifierApplied",
      ]);
      expect(result.value.trace.at(-1)?.phase).toBe("commit");
      expect(result.value.finalState.combatants[1]?.conditions).toEqual([
        "rattled",
      ]);
    }
  });

  it("classifies missing content validation report ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result =
      await transport.loadContentValidationReport("missing-scenario");

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message: "Content validation report not found: missing-scenario",
        retryable: false,
      },
    });
  });

  it("classifies missing scenario ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadScenario("missing-scenario");

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message: "Scenario not found: missing-scenario",
        retryable: false,
      },
    });
  });

  it("uses the checked Rust-backed combat session catalog as the default transport payload", async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const catalogResult = await transport.loadSessionCatalog();
    const result = await transport.loadSessionStep(
      "hexing-bolt-opening-exchange",
      "adept-hexing-bolt-hit",
    );

    expect(catalogResult.ok).toBe(true);
    if (catalogResult.ok) {
      expect(catalogResult.value[0]?.steps.map((step) => step.id)).toEqual([
        "adept-hexing-bolt-hit",
        "adept-hexing-bolt-miss",
        "adept-hexing-bolt-self-target-rejected",
      ]);
    }
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatSessionStepReadout);
      expect(result.value.combatLog[0]?.eventTypes).toEqual([
        "ActionUsed",
        "AttackRolled",
        "DamageApplied",
        "ModifierApplied",
      ]);
      expect(result.value.stateBefore.combatants[1]?.hitPoints.current).toBe(
        18,
      );
      expect(result.value.stateAfter.combatants[1]?.conditions).toEqual([
        "rattled",
      ]);
      expect(result.value.actionResourceLedger.combatants).toEqual([
        {
          combatantId: "entity-adept",
          resources: [
            {
              resourceId: "standard-action",
              sourceId: "base",
              kind: "standardAction",
              current: 0,
              max: 1,
              available: false,
              refreshPolicy: { kind: "turnStart", turns: null },
              remainingRefreshTurns: null,
            },
          ],
        },
        {
          combatantId: "entity-raider",
          resources: [
            {
              resourceId: "standard-action",
              sourceId: "base",
              kind: "standardAction",
              current: 1,
              max: 1,
              available: true,
              refreshPolicy: { kind: "turnStart", turns: null },
              remainingRefreshTurns: null,
            },
          ],
        },
      ]);
    }
  });

  it("uses the checked Rust-backed control history fixture as the default transport payload", async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadSessionControlHistory(
      "hexing-bolt-control-sequence",
    );

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatControlHistoryReadout);
      expect(result.value.history.map((entry) => entry.commandKind)).toEqual([
        "explicitStart",
        "advanceTurn",
        "explicitEnd",
      ]);
      expect(result.value.history.map((entry) => entry.decisionKind)).toEqual([
        "accepted",
        "accepted",
        "accepted",
      ]);
      expect(result.value.history[0]?.lifecycleTransitionSequence).toBe(0);
      expect(result.value.history[1]?.turnTransitionSequence).toBe(0);
    }
  });

  it("classifies missing combat control history ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionControlHistory(
      "missing-control-history",
    );

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message: "Combat control history not found: missing-control-history",
        retryable: false,
      },
    });
  });

  it("uses the checked Rust-backed mixed combat script fixture as the default transport payload", async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadSessionScriptReadout(
      "hexing-bolt-mixed-control-script",
    );

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatScriptReadout);
      expect(result.value.steps.map((step) => step.commandKind)).toEqual([
        "control",
        "control",
        "intent",
        "selectedCandidate",
        "control",
        "selectedCandidate",
        "intent",
        "control",
        "control",
      ]);
      expect(result.value.steps.map((step) => step.decisionKind)).toEqual([
        "accepted",
        "rejectedNoop",
        "rejectedByResolver",
        "acceptedByResolver",
        "accepted",
        "rejectedByUnavailableCandidates",
        "rejectedByTurnOrder",
        "accepted",
        "accepted",
      ]);
      expect(result.value.steps[2]?.runtimeStepId).toBe(
        "script-missing-damage-intent",
      );
      expect(result.value.steps[2]?.commandAuditSequence).toBe(0);
      expect(result.value.steps[3]?.runtimeStepId).toBe(
        "script-selected-runtime-hit",
      );
      expect(result.value.steps[3]?.commandAuditSequence).toBe(1);
      expect(result.value.steps[5]?.runtimeStepId).toBeNull();
      expect(result.value.steps[5]?.commandAuditSequence).toBeNull();
      expect(result.value.steps[6]?.runtimeStepId).toBe(
        "script-wrong-turn-intent",
      );
      expect(result.value.steps[6]?.commandAuditSequence).toBe(2);
      expect(result.value.steps[1]?.controlHistorySequence).toBe(1);
      expect(result.value.steps[7]?.id).toBe("script-advance-turn-wrap");
      expect(result.value.steps[7]?.controlHistorySequence).toBe(3);
      expect(result.value.finalLifecyclePhase).toBe("ended");
      expect(result.value.finalState).toEqual({
        summary: "Current session state.",
        combatants: [
          {
            id: "entity-adept",
            name: "Adept",
            hitPoints: { current: 24, max: 24 },
            conditions: [],
          },
          {
            id: "entity-raider",
            name: "Raider",
            hitPoints: { current: 9, max: 18 },
            conditions: [],
          },
        ],
      });
      expect(result.value.finalTurnOrder).toEqual({
        roundNumber: 2,
        currentTurnIndex: 0,
        participantOrder: ["entity-adept", "entity-raider"],
        currentActorId: "entity-adept",
      });
      expect(result.value.finalActionResourceLedger).toEqual({
        combatants: [
          {
            combatantId: "entity-adept",
            resources: [
              {
                resourceId: "standard-action",
                sourceId: "base",
                kind: "standardAction",
                current: 1,
                max: 1,
                available: true,
                refreshPolicy: { kind: "turnStart", turns: null },
                remainingRefreshTurns: null,
              },
            ],
          },
          {
            combatantId: "entity-raider",
            resources: [
              {
                resourceId: "standard-action",
                sourceId: "base",
                kind: "standardAction",
                current: 1,
                max: 1,
                available: true,
                refreshPolicy: { kind: "turnStart", turns: null },
                remainingRefreshTurns: null,
              },
            ],
          },
        ],
      });
      expect(result.value.currentTurnActionUsage).toEqual({
        roundNumber: 2,
        turnIndex: 0,
        currentActorId: "entity-adept",
        usedActionCount: 0,
        usedActionIds: [],
        usedAbilityIds: [],
      });
      expect(result.value.finalEquipmentLedger.combatants).toEqual([
        {
          combatantId: "entity-adept",
          inventoryItemIds: ["item.hex-focus"],
          equippedItemIds: ["item.hex-focus"],
          availableAbilityIds: ["ability.hexing-bolt"],
        },
        {
          combatantId: "entity-raider",
          inventoryItemIds: ["item.raider-mail"],
          equippedItemIds: ["item.raider-mail"],
          availableAbilityIds: [],
        },
      ]);
      expect(result.value.finalClassBuildLedger.combatants).toEqual([
        {
          combatantId: "entity-adept",
          classInputs: [
            {
              classId: "class.hex-adept",
              version: "1.0.0",
              level: 1,
              appliedGrantLevels: [1],
              sourceIds: ["class:class.hex-adept@1.0.0:1"],
            },
          ],
        },
        {
          combatantId: "entity-raider",
          classInputs: [
            {
              classId: "class.raider",
              version: "1.0.0",
              level: 1,
              appliedGrantLevels: [],
              sourceIds: [],
            },
          ],
        },
      ]);
      expect(result.value.finalCurrentActorOptions).toEqual({
        roundNumber: 2,
        turnIndex: 0,
        lifecyclePhase: "ended",
        currentActorId: "entity-adept",
        currentActorDefeated: false,
        available: false,
        unavailableReason: "combatEnded",
        actions: [],
      });
      expect(result.value.finalCombatantVitality).toEqual({
        combatants: [
          {
            combatantId: "entity-adept",
            currentHitPoints: 24,
            maxHitPoints: 24,
            defeated: false,
          },
          {
            combatantId: "entity-raider",
            currentHitPoints: 9,
            maxHitPoints: 18,
            defeated: false,
          },
        ],
        activeCombatantIds: ["entity-adept", "entity-raider"],
        defeatedCombatantIds: [],
        activeCount: 2,
        defeatedCount: 0,
      });
      expect(result.value.finalCombatEndCondition).toEqual({
        policy: {
          kind: "lastSideStanding",
          objectiveSideId: null,
        },
        combatShouldEnd: false,
        conditionKind: "ongoing",
        outcomeKind: "ongoing",
        activeSides: ["ally", "enemy"],
        defeatedSides: [],
        winningSides: [],
        activeAllyCount: 1,
        activeEnemyCount: 1,
        defeatedAllyCount: 0,
        defeatedEnemyCount: 0,
        reason:
          "Combat can continue because multiple configured sides have active combatants.",
      });
      expect(result.value.lifecycleTransitionLog).toEqual([
        {
          sequence: 0,
          trigger: "explicitStart",
          stepIndex: 0,
          previousLifecyclePhase: "ready",
          nextLifecyclePhase: "inProgress",
          startedAtStep: 0,
          endedAtStep: null,
        },
        {
          sequence: 1,
          trigger: "explicitEnd",
          stepIndex: 3,
          previousLifecyclePhase: "inProgress",
          nextLifecyclePhase: "ended",
          startedAtStep: 0,
          endedAtStep: 3,
        },
      ]);
      expect(result.value.turnTransitionLog).toEqual([
        {
          sequence: 0,
          previousRoundNumber: 1,
          previousTurnIndex: 0,
          previousActorId: "entity-adept",
          nextRoundNumber: 1,
          nextTurnIndex: 1,
          nextActorId: "entity-raider",
          wrappedRound: false,
        },
        {
          sequence: 1,
          previousRoundNumber: 1,
          previousTurnIndex: 1,
          previousActorId: "entity-raider",
          nextRoundNumber: 2,
          nextTurnIndex: 0,
          nextActorId: "entity-adept",
          wrappedRound: true,
        },
      ]);
      expect(result.value.commandAuditLog).toEqual([
        {
          id: "audit-script-missing-damage-intent",
          stepId: "script-missing-damage-intent",
          sequence: 0,
          outcomeClass: "rejectedInvalidCommand",
          decisionKind: "rejectedByResolver",
          preflightDecisionKind: "accepted",
          accepted: false,
          rejection: "missingDamageRoll",
          eventCount: 0,
          traceCount: 2,
          rollConsumption: [
            {
              sequence: 0,
              requestKind: "attackRoll",
              suppliedValue: 17,
              consumed: true,
              reason: "Attack roll value was consumed for hit resolution.",
            },
            {
              sequence: 1,
              requestKind: "damageRoll",
              suppliedValue: null,
              consumed: false,
              reason:
                "Damage roll was requested after a hit but no roll value was supplied.",
            },
          ],
          stateBeforeFingerprint: {
            algorithm: "fnv1a64.rulebench-state.v0",
            value: "43b17555d3d7ff0d",
          },
          stateAfterFingerprint: {
            algorithm: "fnv1a64.rulebench-state.v0",
            value: "43b17555d3d7ff0d",
          },
        },
        {
          id: "audit-script-selected-runtime-hit",
          stepId: "script-selected-runtime-hit",
          sequence: 1,
          outcomeClass: "acceptedHit",
          decisionKind: "acceptedByResolver",
          preflightDecisionKind: "accepted",
          accepted: true,
          rejection: null,
          eventCount: 4,
          traceCount: 5,
          rollConsumption: [
            {
              sequence: 0,
              requestKind: "attackRoll",
              suppliedValue: 17,
              consumed: true,
              reason: "Attack roll value was consumed for hit resolution.",
            },
            {
              sequence: 1,
              requestKind: "damageRoll",
              suppliedValue: 5,
              consumed: true,
              reason: "Damage roll value was consumed for damage resolution.",
            },
          ],
          stateBeforeFingerprint: {
            algorithm: "fnv1a64.rulebench-state.v0",
            value: "43b17555d3d7ff0d",
          },
          stateAfterFingerprint: {
            algorithm: "fnv1a64.rulebench-state.v0",
            value: "1872b66dd0de303a",
          },
        },
        {
          id: "audit-script-wrong-turn-intent",
          stepId: "script-wrong-turn-intent",
          sequence: 2,
          outcomeClass: "rejectedInvalidCommand",
          decisionKind: "rejectedByTurnOrder",
          preflightDecisionKind: "rejectedByTurnOrder",
          accepted: false,
          rejection: "invalidAction",
          eventCount: 0,
          traceCount: 2,
          rollConsumption: [],
          stateBeforeFingerprint: {
            algorithm: "fnv1a64.rulebench-state.v0",
            value: "1872b66dd0de303a",
          },
          stateAfterFingerprint: {
            algorithm: "fnv1a64.rulebench-state.v0",
            value: "1872b66dd0de303a",
          },
        },
      ]);
      expect(result.value.actionUsageLog).toEqual([
        {
          id: "action-usage-script-selected-runtime-hit",
          stepId: "script-selected-runtime-hit",
          stepIndex: 1,
          roundNumber: 1,
          turnIndex: 0,
          actorId: "entity-adept",
          actionId: "hexing_bolt",
          abilityId: "ability.hexing-bolt",
          targetId: "entity-raider",
          outcomeClass: "acceptedHit",
        },
      ]);
      expect(result.value.actionResourceTransitionLog).toEqual([
        {
          sequence: 0,
          transitionKind: "spent",
          combatantId: "entity-adept",
          resourceId: "standard-action",
          resourceKind: "standardAction",
          amount: 1,
          previousResource: {
            resourceId: "standard-action",
            sourceId: "base",
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
            refreshPolicy: { kind: "turnStart", turns: null },
            remainingRefreshTurns: null,
          },
          nextResource: {
            resourceId: "standard-action",
            sourceId: "base",
            kind: "standardAction",
            current: 0,
            max: 1,
            available: false,
            refreshPolicy: { kind: "turnStart", turns: null },
            remainingRefreshTurns: null,
          },
          commandStepId: "script-selected-runtime-hit",
          commandStepIndex: 1,
          turnTransitionSequence: null,
          roundNumber: 1,
          turnIndex: 0,
          currentActorId: "entity-adept",
          reason: "Action resource spent.",
        },
        {
          sequence: 1,
          transitionKind: "refreshed",
          combatantId: "entity-raider",
          resourceId: "standard-action",
          resourceKind: "standardAction",
          amount: 0,
          previousResource: {
            resourceId: "standard-action",
            sourceId: "base",
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
            refreshPolicy: { kind: "turnStart", turns: null },
            remainingRefreshTurns: null,
          },
          nextResource: {
            resourceId: "standard-action",
            sourceId: "base",
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
            refreshPolicy: { kind: "turnStart", turns: null },
            remainingRefreshTurns: null,
          },
          commandStepId: null,
          commandStepIndex: null,
          turnTransitionSequence: 0,
          roundNumber: 1,
          turnIndex: 1,
          currentActorId: "entity-raider",
          reason: "Action resource refreshed at turn start.",
        },
        {
          sequence: 2,
          transitionKind: "refreshed",
          combatantId: "entity-adept",
          resourceId: "standard-action",
          resourceKind: "standardAction",
          amount: 1,
          previousResource: {
            resourceId: "standard-action",
            sourceId: "base",
            kind: "standardAction",
            current: 0,
            max: 1,
            available: false,
            refreshPolicy: { kind: "turnStart", turns: null },
            remainingRefreshTurns: null,
          },
          nextResource: {
            resourceId: "standard-action",
            sourceId: "base",
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
            refreshPolicy: { kind: "turnStart", turns: null },
            remainingRefreshTurns: null,
          },
          commandStepId: null,
          commandStepIndex: null,
          turnTransitionSequence: 1,
          roundNumber: 2,
          turnIndex: 0,
          currentActorId: "entity-adept",
          reason: "Action resource refreshed at turn start.",
        },
      ]);
      expect(result.value.modifierDurationExpirationLog).toEqual([
        {
          sequence: 0,
          combatantId: "entity-raider",
          modifierId: "rattled",
          previousModifier: {
            modifierId: "rattled",
            sourceId: "hexing_bolt",
            label: "rattled",
            duration: "until end of next turn",
            tenure: "temporary",
            stackingGroup: "rattled",
            stackingPolicy: "refresh",
            durationPolicy: { kind: "turns", value: 1, event: null },
            remainingTurns: 1,
            remainingRounds: null,
          },
          nextModifier: null,
          trigger: { kind: "turnBoundary", event: null },
          turnTransitionSequence: 1,
          roundNumber: 2,
          turnIndex: 0,
          currentActorId: "entity-adept",
          reason: "Turn-counted modifier expired at turn boundary.",
        },
      ]);
    }
  });

  it("reads generated bounded automatic combat run evidence through transport", async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadSessionAutomaticRunReadout(
      "hexing-bolt-bounded-automatic-run",
    );

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatAutomaticRunReadout);
      expect(result.value.id).toBe("hexing-bolt-bounded-automatic-run");
      expect(result.value.accepted).toBe(true);
      expect(result.value.decisionKind).toBe("completedCombatEnded");
      expect(result.value.maxSteps).toBe(8);
      expect(result.value.policy).toEqual({
        id: "firstAcceptedCandidate",
        version: 1,
        noCandidateBehavior: "advanceTurn",
      });
      expect(result.value.executedStepCount).toBe(4);
      expect(
        result.value.stepDecisions.map(
          (step) => step.policyValidation.code,
        ),
      ).toEqual(["accepted", "accepted", "accepted", "accepted"]);
      expect(result.value.policyDecisions).toHaveLength(4);
      expect(
        result.value.policyDecisions.map(
          (decision) => decision.selectedCandidateIndex,
        ),
      ).toEqual([0, null, null, 0]);
      expect(result.value.policyDecisions[0]?.candidates).toEqual([
        {
          index: 0,
          actionId: "hexing_bolt",
          targetId: "entity-raider",
          accepted: true,
          decisionKind: "accepted",
        },
      ]);
      expect(
        result.value.stepDecisions.map((step) => step.decisionKind),
      ).toEqual([
        "submitCandidate",
        "advanceTurn",
        "advanceTurn",
        "submitCandidate",
      ]);
      expect(
        result.value.stepDecisions.map((step) => step.operationKind),
      ).toEqual([
        "submitCandidate",
        "advanceTurn",
        "advanceTurn",
        "submitCandidate",
      ]);
      expect(result.value.finalLifecyclePhase).toBe("ended");
      expect(result.value.finalState.combatants[1]?.hitPoints.current).toBe(0);
      expect(
        result.value.finalActionResourceLedger.combatants.map(
          (combatant) => combatant.combatantId,
        ),
      ).toEqual(["entity-adept", "entity-raider"]);
      expect(
        result.value.finalActionResourceLedger.combatants.map(
          (combatant) => combatant.resources[0]?.available,
        ),
      ).toEqual([false, true]);
      expect(result.value.finalCurrentActorOptions).toMatchObject({
        roundNumber: 2,
        turnIndex: 0,
        lifecyclePhase: "ended",
        currentActorId: "entity-adept",
        currentActorDefeated: false,
        available: false,
        unavailableReason: "combatEnded",
        actions: [],
      });
      expect(result.value.finalCombatantVitality).toMatchObject({
        activeCombatantIds: ["entity-adept"],
        defeatedCombatantIds: ["entity-raider"],
        activeCount: 1,
        defeatedCount: 1,
      });
      expect(
        result.value.finalCombatantVitality.combatants.map(
          (combatant) => combatant.defeated,
        ),
      ).toEqual([false, true]);
      expect(result.value.finalCombatEndCondition).toEqual({
        policy: {
          kind: "lastSideStanding",
          objectiveSideId: null,
        },
        combatShouldEnd: true,
        conditionKind: "noActiveEnemies",
        outcomeKind: "victory",
        activeSides: ["ally"],
        defeatedSides: ["enemy"],
        winningSides: ["ally"],
        activeAllyCount: 1,
        activeEnemyCount: 0,
        defeatedAllyCount: 0,
        defeatedEnemyCount: 1,
        reason: "Combat should end because no active enemies remain.",
      });
      expect(result.value.finalization).toMatchObject({
        trigger: "conditionalEnd",
        outcomeKind: "victory",
        winningSides: ["ally"],
        remainingSides: ["ally"],
        combatLogEntryCount: 2,
        commandAuditEntryCount: 2,
      });
      expect(result.value.combatLogEntryCount).toBe(2);
      expect(result.value.auditEntryCount).toBe(2);
      expect(result.value.combatLog).toHaveLength(2);
      expect(result.value.commandAuditLog).toHaveLength(2);
      expect(
        result.value.combatLog.map((entry) => entry.outcomeClass),
      ).toEqual(["acceptedHit", "acceptedHit"]);
      expect(result.value.combatLog[0]?.eventTypes).toEqual([
        "ActionUsed",
        "AttackRolled",
        "DamageApplied",
        "ModifierApplied",
      ]);
      expect(result.value.combatLog[1]?.eventTypes).toEqual([
        "ActionUsed",
        "AttackRolled",
        "DamageApplied",
        "ModifierApplied",
      ]);
      expect(
        result.value.commandAuditLog.map((entry) => entry.decisionKind),
      ).toEqual(["acceptedByResolver", "acceptedByResolver"]);
      expect(result.value.commandAuditLog.map((entry) => entry.accepted)).toEqual(
        [true, true],
      );
      expect(result.value.commandAuditLog[0]?.rollConsumption).toEqual([
        {
          sequence: 0,
          requestKind: "attackRoll",
          suppliedValue: 17,
          consumed: true,
          reason: "Attack roll value was consumed for hit resolution.",
        },
        {
          sequence: 1,
          requestKind: "damageRoll",
          suppliedValue: 5,
          consumed: true,
          reason: "Damage roll value was consumed for damage resolution.",
        },
      ]);
      expect(result.value.commandAuditLog[1]?.rollConsumption).toEqual([
        {
          sequence: 0,
          requestKind: "attackRoll",
          suppliedValue: 17,
          consumed: true,
          reason: "Attack roll value was consumed for hit resolution.",
        },
        {
          sequence: 1,
          requestKind: "damageRoll",
          suppliedValue: 5,
          consumed: true,
          reason: "Damage roll value was consumed for damage resolution.",
        },
      ]);
      expect(
        result.value.lifecycleTransitionLog.map((entry) => entry.trigger),
      ).toEqual(["commandStart", "conditionalEnd"]);
      expect(
        result.value.lifecycleTransitionLog.map(
          (entry) => entry.nextLifecyclePhase,
        ),
      ).toEqual(["inProgress", "ended"]);
      expect(
        result.value.turnTransitionLog.map((entry) => entry.nextActorId),
      ).toEqual(["entity-raider", "entity-adept"]);
      expect(
        result.value.turnTransitionLog.map((entry) => entry.wrappedRound),
      ).toEqual([false, true]);
      expect(
        result.value.actionUsageLog.map((entry) => entry.stepId),
      ).toEqual([
        "hexing-bolt-bounded-automatic-run-step-0",
        "hexing-bolt-bounded-automatic-run-step-3",
      ]);
      expect(
        result.value.actionUsageLog.map((entry) => entry.abilityId),
      ).toEqual(["ability.hexing-bolt", "ability.hexing-bolt"]);
      expect(
        result.value.actionResourceTransitionLog.map(
          (entry) => entry.transitionKind,
        ),
      ).toEqual(["spent", "refreshed", "refreshed", "spent"]);
      expect(
        result.value.actionResourceTransitionLog.map(
          (entry) => entry.combatantId,
        ),
      ).toEqual([
        "entity-adept",
        "entity-raider",
        "entity-adept",
        "entity-adept",
      ]);
      expect(result.value.modifierDurationExpirationLog).toEqual([
        {
          sequence: 0,
          combatantId: "entity-raider",
          modifierId: "rattled",
          previousModifier: {
            modifierId: "rattled",
            sourceId: "hexing_bolt",
            label: "rattled",
            duration: "until end of next turn",
            tenure: "temporary",
            stackingGroup: "rattled",
            stackingPolicy: "refresh",
            durationPolicy: { kind: "turns", value: 1, event: null },
            remainingTurns: 1,
            remainingRounds: null,
          },
          nextModifier: null,
          trigger: { kind: "turnBoundary", event: null },
          turnTransitionSequence: 1,
          roundNumber: 2,
          turnIndex: 0,
          currentActorId: "entity-adept",
          reason: "Turn-counted modifier expired at turn boundary.",
        },
      ]);
      expect(result.value.reason).toBe(
        "Automatic combat run completed because combat reached ended lifecycle.",
      );
    }
  });

  it("reads generated automatic run replay verification evidence through transport", async () => {
    expect(defaultCombatSessionCatalog).toBe(rustBackedCombatSessionCatalog);

    const transport = createFakeRulebenchTransport();
    const result = await transport.loadSessionAutomaticRunReplayReadout(
      "hexing-bolt-bounded-automatic-run-replay",
    );

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBe(defaultCombatAutomaticRunReplayReadout);
      expect(result.value.id).toBe(
        "hexing-bolt-bounded-automatic-run-replay",
      );
      expect(result.value.accepted).toBe(true);
      expect(result.value.decisionKind).toBe("verified");
      expect(result.value.expectedFinalStateFingerprint).toEqual({
        algorithm: "fnv1a64.rulebench-projection.v0",
        value: "977a31e4f5dc71bc",
      });
      expect(result.value.actualFinalStateFingerprint).toEqual({
        algorithm: "fnv1a64.rulebench-projection.v0",
        value: "977a31e4f5dc71bc",
      });
      expect(result.value.finalStateFingerprintMatches).toBe(true);
      expect(result.value.finalizationMatches).toBe(true);
      expect(result.value.expectedRunDecisionKind).toBe(
        "completedCombatEnded",
      );
      expect(result.value.actualRunDecisionKind).toBe("completedCombatEnded");
      expect(result.value.runDecisionKindMatches).toBe(true);
      expect(result.value.expectedExecutedStepCount).toBe(4);
      expect(result.value.actualExecutedStepCount).toBe(4);
      expect(result.value.executedStepCountMatches).toBe(true);
      expect(result.value.policyDecisionsMatch).toBe(true);
      expect(result.value.actionResourceTransitionLogMatches).toBe(true);
      expect(result.value.equipmentLedgerMatches).toBe(true);
      expect(result.value.classBuildLedgerMatches).toBe(true);
      expect(result.value.equipmentTransitionLogMatches).toBe(true);
      expect(result.value.reactionWindowLifecycleLogMatches).toBe(true);
      expect(result.value.reactionAuditLogMatches).toBe(true);
      expect(result.value.modifierDurationExpirationLogMatches).toBe(true);
      expect(result.value.replayedRun.id).toBe(
        "hexing-bolt-bounded-automatic-run",
      );
      expect(result.value.replayedRun.decisionKind).toBe(
        "completedCombatEnded",
      );
      expect(result.value.replayedRun.executedStepCount).toBe(4);
      expect(result.value.replayedRun.finalLifecyclePhase).toBe("ended");
      expect(result.value.replayedRun.finalization?.outcomeKind).toBe(
        "victory",
      );
      expect(result.value.replayedRun.combatLogEntryCount).toBe(2);
      expect(result.value.replayedRun.auditEntryCount).toBe(2);
      expect(result.value.reason).toBe(
        "Automatic run replay verified expected final evidence.",
      );
    }
  });

  it("classifies missing combat script readout ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionScriptReadout("missing-script");

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message: "Combat script readout not found: missing-script",
        retryable: false,
      },
    });
  });

  it("classifies missing combat session ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionStep(
      "missing-session",
      "missing-step",
    );

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message: "Combat session not found: missing-session",
        retryable: false,
      },
    });
  });

  it("classifies missing combat session step ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionStep(
      "hexing-bolt-opening-exchange",
      "missing-step",
    );

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message:
          "Combat session step not found: hexing-bolt-opening-exchange / missing-step",
        retryable: false,
      },
    });
  });

  it("classifies missing automatic combat run readout ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionAutomaticRunReadout(
      "missing-automatic-run",
    );

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message:
          "Combat automatic run readout not found: missing-automatic-run",
        retryable: false,
      },
    });
  });

  it("classifies missing automatic combat run replay readout ids as not found", async () => {
    const transport = createFakeRulebenchTransport();

    const result = await transport.loadSessionAutomaticRunReplayReadout(
      "missing-automatic-run-replay",
    );

    expect(result).toEqual({
      ok: false,
      error: {
        kind: "not-found",
        message:
          "Combat automatic run replay readout not found: missing-automatic-run-replay",
        retryable: false,
      },
    });
  });
});
