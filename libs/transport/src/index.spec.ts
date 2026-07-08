import { describe, expect, it } from "vitest";
import {
  createFakeRulebenchTransport,
  defaultCombatAutomaticRunReadout,
  defaultCombatControlHistoryReadout,
  defaultCombatScriptReadout,
  defaultCombatSessionCatalog,
  defaultCombatSessionStepReadout,
  defaultContentValidationCatalog,
  defaultContentValidationReport,
  defaultRulesetCatalog,
  defaultScenarioCatalog,
  defaultScenarioReadout,
} from "./index";
import { rustBackedCombatSessionCatalog } from "./generated/rust-combat-session";
import {
  rustBackedContentValidationCatalog,
  rustBackedRulesetCatalog,
  rustBackedScenarioCatalog,
} from "./generated/rust-scenario-catalog";

describe("RulebenchTransport fixtures", () => {
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
            { kind: "standardAction", current: 0, max: 1, available: false },
          ],
        },
        {
          combatantId: "entity-raider",
          resources: [
            { kind: "standardAction", current: 1, max: 1, available: true },
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
        "advanceTurn",
      ]);
      expect(result.value.history.map((entry) => entry.decisionKind)).toEqual([
        "accepted",
        "accepted",
        "accepted",
        "rejectedByLifecycle",
      ]);
      expect(result.value.history[0]?.lifecycleTransitionSequence).toBe(0);
      expect(result.value.history[1]?.turnTransitionSequence).toBe(0);
      expect(result.value.history[3]?.reason).toBe("Combat is already ended.");
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
        "acceptedByResolver",
        "accepted",
        "rejectedByUnavailableCandidates",
        "rejectedByTurnOrder",
        "accepted",
        "accepted",
      ]);
      expect(result.value.steps[2]?.runtimeStepId).toBe(
        "script-selected-runtime-hit",
      );
      expect(result.value.steps[2]?.commandAuditSequence).toBe(0);
      expect(result.value.steps[4]?.runtimeStepId).toBeNull();
      expect(result.value.steps[4]?.commandAuditSequence).toBeNull();
      expect(result.value.steps[5]?.runtimeStepId).toBe(
        "script-wrong-turn-intent",
      );
      expect(result.value.steps[5]?.commandAuditSequence).toBe(1);
      expect(result.value.steps[1]?.controlHistorySequence).toBe(1);
      expect(result.value.steps[6]?.id).toBe("script-advance-turn-wrap");
      expect(result.value.steps[6]?.controlHistorySequence).toBe(3);
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
                kind: "standardAction",
                current: 1,
                max: 1,
                available: true,
              },
            ],
          },
          {
            combatantId: "entity-raider",
            resources: [
              {
                kind: "standardAction",
                current: 1,
                max: 1,
                available: true,
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
        combatShouldEnd: false,
        conditionKind: "ongoing",
        activeAllyCount: 1,
        activeEnemyCount: 1,
        defeatedAllyCount: 0,
        defeatedEnemyCount: 0,
        reason:
          "Combat can continue because both sides have active combatants.",
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
          stepIndex: 2,
          previousLifecyclePhase: "inProgress",
          nextLifecyclePhase: "ended",
          startedAtStep: 0,
          endedAtStep: 2,
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
          id: "audit-script-selected-runtime-hit",
          stepId: "script-selected-runtime-hit",
          sequence: 0,
          outcomeClass: "acceptedHit",
          decisionKind: "acceptedByResolver",
          preflightDecisionKind: "accepted",
          accepted: true,
          rejection: null,
          eventCount: 4,
          traceCount: 4,
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
          sequence: 1,
          outcomeClass: "rejectedInvalidCommand",
          decisionKind: "rejectedByTurnOrder",
          preflightDecisionKind: "rejectedByTurnOrder",
          accepted: false,
          rejection: "invalidAction",
          eventCount: 0,
          traceCount: 2,
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
          stepIndex: 0,
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
          resourceKind: "standardAction",
          previousResource: {
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
          },
          nextResource: {
            kind: "standardAction",
            current: 0,
            max: 1,
            available: false,
          },
          commandStepId: "script-selected-runtime-hit",
          commandStepIndex: 0,
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
          resourceKind: "standardAction",
          previousResource: {
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
          },
          nextResource: {
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
          },
          commandStepId: null,
          commandStepIndex: null,
          turnTransitionSequence: 0,
          roundNumber: 1,
          turnIndex: 1,
          currentActorId: "entity-raider",
          reason: "Action resource refreshed.",
        },
        {
          sequence: 2,
          transitionKind: "refreshed",
          combatantId: "entity-adept",
          resourceKind: "standardAction",
          previousResource: {
            kind: "standardAction",
            current: 0,
            max: 1,
            available: false,
          },
          nextResource: {
            kind: "standardAction",
            current: 1,
            max: 1,
            available: true,
          },
          commandStepId: null,
          commandStepIndex: null,
          turnTransitionSequence: 1,
          roundNumber: 2,
          turnIndex: 0,
          currentActorId: "entity-adept",
          reason: "Action resource refreshed.",
        },
      ]);
      expect(result.value.modifierDurationExpirationLog).toEqual([
        {
          sequence: 0,
          combatantId: "entity-raider",
          modifierId: "rattled",
          previousModifier: {
            modifierId: "rattled",
            label: "rattled",
            duration: "until end of next turn",
            tenure: "temporary",
          },
          nextModifier: null,
          turnTransitionSequence: 1,
          roundNumber: 2,
          turnIndex: 0,
          currentActorId: "entity-adept",
          reason: "Temporary modifier expired at turn boundary.",
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
      expect(result.value.executedStepCount).toBe(5);
      expect(
        result.value.stepDecisions.map((step) => step.decisionKind),
      ).toEqual([
        "submitCandidate",
        "advanceTurn",
        "advanceTurn",
        "submitCandidate",
        "conditionalEnd",
      ]);
      expect(
        result.value.stepDecisions.map((step) => step.operationKind),
      ).toEqual([
        "submitCandidate",
        "advanceTurn",
        "advanceTurn",
        "submitCandidate",
        "conditionalEnd",
      ]);
      expect(result.value.finalLifecyclePhase).toBe("ended");
      expect(result.value.finalState.combatants[1]?.hitPoints.current).toBe(0);
      expect(result.value.combatLogEntryCount).toBe(2);
      expect(result.value.auditEntryCount).toBe(2);
      expect(result.value.reason).toBe(
        "Automatic combat run completed because combat reached ended lifecycle.",
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
});
