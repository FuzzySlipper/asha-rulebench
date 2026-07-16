import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { createLiveRulebenchTransport } from "@asha-rulebench/transport";

test.describe.configure({ mode: "serial" });

async function openLiveCombatWorkspace(page: Page) {
  const menubar = page.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  await menubar.getByRole("menuitem", { name: "Scenario" }).click();
  await page
    .getByRole("menu", { name: "Scenario" })
    .getByRole("menuitem", { name: "Live combat setup" })
    .click();
  const dialog = page.getByRole("dialog", { name: "Live combat setup" });
  await expect(dialog).toBeVisible();
  return dialog.getByRole("region", { name: "Live combat setup controls" });
}

async function invokeApplicationCommand(
  page: Page,
  group: string,
  command: string,
): Promise<void> {
  const menubar = page.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  await menubar.getByRole("menuitem", { name: group }).click();
  await page
    .getByRole("menu", { name: group })
    .getByRole("menuitem", { name: command })
    .click();
}

test("invokes live Rust authority through the Angular origin", async ({
  page,
}) => {
  await page.goto("/");
  const apiBaseUrl = new URL("/api/rulebench/v1", page.url()).toString();
  const transport = createLiveRulebenchTransport({ apiBaseUrl });
  const sessionId = `e2e-live-rust-session-${Date.now()}`;
  let sessionExists = false;

  try {
    const connected = await transport.connect();
    expect(connected).toEqual({
      ok: true,
      value: {
        protocolId: "asha-rulebench.protocol",
        protocolVersion: 7,
        authoritySurface: "asha-rulebench.local-authority.v0",
      },
    });

    const scenarios = await transport.listScenarios();
    expect(scenarios.ok).toBe(true);
    if (!scenarios.ok) return;
    expect(scenarios.value.map((scenario) => scenario.id)).toContain(
      "hexing-bolt-hit",
    );
    expect(scenarios.value.map((scenario) => scenario.id)).toContain(
      "hexing-bolt-reaction",
    );
    expect(
      scenarios.value.find((scenario) => scenario.id === "hexing-bolt-hit"),
    ).toEqual(
      expect.objectContaining({
        rulesetId: "asha-rulebench.hexing-bolt.v0",
        participants: [
          expect.objectContaining({ id: "entity-adept", sideId: "ally" }),
          expect.objectContaining({ id: "entity-raider", sideId: "enemy" }),
        ],
      }),
    );

    await expect(
      transport.createSession({
        sessionId: "e2e-invalid-setup",
        scenarioId: "hexing-bolt-hit",
        participantOrder: ["entity-adept"],
      }),
    ).resolves.toEqual({
      ok: false,
      error: {
        kind: "bridge",
        code: "invalidRequest",
        message:
          "Participant setup must include all 2 scenario participants exactly once.",
        retryable: false,
      },
    });

    const created = await transport.createSession({
      sessionId,
      scenarioId: "hexing-bolt-hit",
      participantOrder: ["entity-adept", "entity-raider"],
    });
    expect(created.ok).toBe(true);
    if (!created.ok) return;
    sessionExists = true;
    expect(created.value.lifecyclePhase).toBe("ready");
    expect(created.value.combatLog).toEqual([]);
    expect(created.value.auditLog).toEqual([]);
    expect(created.value.gameplayFabric).toEqual(
      expect.objectContaining({
        decisions: [],
        pendingDecisionCount: 0,
        reactionFrameHashes: [],
      }),
    );
    const initialFingerprint = created.value.stateFingerprint;

    const started = await transport.submitControl(sessionId, {
      kind: "explicitStart",
    });
    expect(started.ok).toBe(true);
    if (!started.ok) return;
    expect(started.value.accepted).toBe(true);
    expect(started.value.snapshot.lifecyclePhase).toBe("inProgress");

    const options = await transport.getCurrentActorOptions(sessionId);
    expect(options.ok).toBe(true);
    if (!options.ok) return;
    expect(options.value.currentActorId).toBe("entity-adept");
    expect(options.value.actions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          actionId: "hexing_bolt",
          available: true,
          targets: [expect.objectContaining({ targetId: "entity-raider" })],
        }),
        expect.objectContaining({
          actionId: "move.entity-adept",
          available: true,
          targetMode: "cell",
        }),
      ]),
    );

    const executed = await transport.submitIntent(sessionId, {
      id: "e2e-hexing-bolt-hit",
      title: "E2E Hexing Bolt hit",
      summary: "Canonical live Rust authority invocation.",
      intent: {
        actorId: "entity-adept",
        actionId: "hexing_bolt",
        targetId: "entity-raider",
        targetIds: [],
        targetCell: null,
        destinationCell: null,
        observedOrigin: null,
      },
      rollStream: [17, 5],
    });
    expect(executed.ok).toBe(true);
    if (!executed.ok) return;
    expect(executed.value.step.accepted).toBe(true);
    expect(executed.value.step.events.map((event) => event.kind)).toEqual([
      "actionUsed",
      "attackRolled",
      "damageApplied",
      "modifierApplied",
    ]);
    expect(executed.value.snapshot.stateFingerprint).not.toEqual(
      initialFingerprint,
    );
    expect(executed.value.snapshot.combatLog).toHaveLength(1);
    expect(executed.value.snapshot.auditLog).toHaveLength(1);
    expect(
      executed.value.snapshot.participants.find(
        (participant) => participant.id === "entity-raider",
      ),
    ).toEqual(
      expect.objectContaining({
        currentHitPoints: 9,
        conditions: ["rattled"],
      }),
    );

    const ended = await transport.submitControl(sessionId, {
      kind: "explicitEnd",
    });
    expect(ended.ok).toBe(true);
    if (!ended.ok) return;
    expect(ended.value.snapshot.lifecyclePhase).toBe("ended");

    const closed = await transport.closeSession(sessionId);
    expect(closed.ok).toBe(true);
    if (!closed.ok) return;
    sessionExists = false;
    expect(closed.value.stateFingerprint).toEqual(
      ended.value.snapshot.stateFingerprint,
    );

    const remainingSessions = await transport.listSessions();
    expect(remainingSessions.ok).toBe(true);
    if (remainingSessions.ok) {
      expect(
        remainingSessions.value.some(
          (session) => session.sessionId === sessionId,
        ),
      ).toBe(false);
    }

    const replayPackages = await transport.listReplayPackages();
    expect(replayPackages.ok).toBe(true);
    if (!replayPackages.ok) return;
    expect(replayPackages.value.map((entry) => entry.packageId)).toEqual(
      expect.arrayContaining([
        "hexing-bolt-replay",
        "hexing-bolt-replay-explicit-start",
        `live-${sessionId}`,
      ]),
    );
    const expectedReplayId = "hexing-bolt-replay";
    const actualReplayId = "hexing-bolt-replay-explicit-start";
    const replayReview = await transport.loadReplayPackage(expectedReplayId);
    expect(replayReview.ok).toBe(true);
    if (!replayReview.ok) return;
    expect(replayReview.value.commands[0]).toEqual(
      expect.objectContaining({
        commandKind: "intent",
        suppliedRollStream: [17, 5],
        actual: expect.objectContaining({
          accepted: true,
          acceptedEvents: expect.arrayContaining([
            expect.objectContaining({ kind: "damageApplied" }),
          ]),
        }),
      }),
    );
    await expect(
      transport.loadReplayVerification(expectedReplayId),
    ).resolves.toEqual(
      expect.objectContaining({
        ok: true,
        value: expect.objectContaining({ accepted: true, finalized: true }),
      }),
    );
    const replayComparison = await transport.compareReplayPackages(
      expectedReplayId,
      actualReplayId,
    );
    expect(replayComparison).toEqual(
      expect.objectContaining({
        ok: true,
        value: expect.objectContaining({
          matches: false,
          firstDifference: expect.objectContaining({ path: "commands.length" }),
        }),
      }),
    );

    const missing = await transport.getSession("e2e-missing-session");
    expect(missing).toEqual({
      ok: false,
      error: {
        kind: "bridge",
        code: "unknownSession",
        message: "Session does not exist: e2e-missing-session",
        retryable: false,
      },
    });

    const mismatched = createLiveRulebenchTransport({
      apiBaseUrl,
      protocolVersion: 999,
    });
    await expect(mismatched.connect()).resolves.toEqual({
      ok: false,
      error: {
        kind: "protocol",
        code: "protocolVersionMismatch",
        message: "Unsupported protocol version 999; expected 7.",
        retryable: false,
      },
    });
    mismatched.disconnect();
  } finally {
    if (sessionExists) {
      await transport.submitControl(sessionId, { kind: "explicitEnd" });
      await transport.closeSession(sessionId);
    }
    transport.disconnect();
  }
});

test("completes a supported scenario through the visible panel workbench", async ({
  page,
}) => {
  const sessionId = `e2e-visible-panel-session-${Date.now()}`;
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);

  await expect(
    workspace.getByText("asha-rulebench.local-authority.v0"),
  ).toBeVisible();
  await workspace
    .getByRole("button", { name: "Ruined Watchtower Skirmish", exact: true })
    .click();
  await workspace.getByLabel("Session", { exact: true }).fill(sessionId);
  await workspace.getByRole("button", { name: "Create session" }).click();

  const recovery = workspace.getByRole("region", {
    name: "Session recovery",
  });
  await expect(recovery).toContainText("Restart-safe sessions");
  await expect(recovery).toContainText(
    `${sessionId} · new this process · generation 0`,
  );
  await expect(recovery.getByRole("button", { name: "Fork" })).toBeVisible();
  await expect(recovery.getByRole("button", { name: "Discard" })).toBeVisible();

  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();

  const statusPanel = page.getByRole("region", { name: "4. Turn status" });
  const initiativePanel = page.getByRole("region", { name: "2. Initiative" });
  const actionsPanel = page.getByRole("region", {
    name: "6. Available actions",
  });
  const gridPanel = page.getByRole("region", { name: "1. Combat grid" });
  const unitsPanel = page.getByRole("region", { name: "7. Active units" });
  await expect(statusPanel).toContainText("e2e-visible-panel-session");
  await expect(statusPanel).toContainText("Ready");
  await expect(
    initiativePanel.getByRole("listitem", { name: "Adept, Current" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Raider, Next" }),
  ).toHaveCount(0);
  await expect(
    initiativePanel.getByRole("listitem", { name: "Scout, Next" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Raider, Queued" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Bruiser, Queued" }),
  ).toBeVisible();

  await invokeApplicationCommand(page, "Run", "Start combat");
  await expect(statusPanel).toContainText("In Progress");
  await expect(
    gridPanel.getByRole("grid", { name: /Live combat board/ }),
  ).toBeVisible();
  await actionsPanel.getByRole("radio", { name: "Automatic" }).check();
  await expect(actionsPanel.getByLabel("Attack roll")).toHaveCount(0);
  await expect(actionsPanel).toContainText("records the concrete results");
  await actionsPanel.getByRole("radio", { name: "Manual" }).check();
  await expect(actionsPanel.getByLabel("Attack roll")).toBeVisible();
  await expect(actionsPanel.getByLabel("Additional rolls")).toBeVisible();
  await actionsPanel.getByRole("button", { name: "Select Move" }).click();
  const destinations = gridPanel.getByRole("gridcell", { name: /^Move to / });
  await expect(destinations.first()).toBeVisible();
  await destinations.first().click();
  await expect(destinations.first()).toHaveAttribute("aria-pressed", "true");
  const keyboardDestination = destinations.nth(1);
  await keyboardDestination.focus();
  await keyboardDestination.press("Enter");
  await expect(keyboardDestination).toHaveAttribute("aria-pressed", "true");
  await actionsPanel
    .getByRole("button", { name: "Submit", exact: true })
    .click();
  await expect(
    gridPanel.getByRole("gridcell", { name: /occupied by Adept/ }),
  ).toBeVisible();
  await actionsPanel
    .getByRole("button", { name: "Select Hexing Bolt" })
    .click();
  const gridTarget = gridPanel.getByRole("gridcell", {
    name: /^Target at .*occupied by Raider/,
  });
  await gridTarget.click();
  await expect(gridTarget).toHaveAttribute("aria-pressed", "true");
  await unitsPanel
    .getByRole("button", { name: "Select Adept as target" })
    .click();
  await actionsPanel
    .getByRole("button", { name: "Preflight", exact: true })
    .click();
  const commandEvidence = actionsPanel.getByRole("region", {
    name: "Command decision evidence",
  });
  await expect(commandEvidence).toBeFocused();
  await expect(commandEvidence).toContainText("Target is not hostile");

  await unitsPanel
    .getByRole("button", { name: "Select Raider as target" })
    .click();
  await actionsPanel
    .getByRole("button", { name: "Preflight", exact: true })
    .click();
  await expect(commandEvidence).toContainText("Accepted");

  await actionsPanel
    .getByRole("button", { name: "Submit", exact: true })
    .click();
  await expect(
    unitsPanel.getByRole("listitem", {
      name: /Raider, Active, selected target/,
    }),
  ).toContainText("9/18 HP");
  await expect(commandEvidence).toContainText("Damage Applied");
  const evidencePanel = page.getByRole("region", { name: "5. Evidence log" });
  await evidencePanel.getByRole("tab", { name: "Combat" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Manual command",
  );
  await evidencePanel.getByRole("tab", { name: "DomainEvents" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Damage Applied",
  );
  await evidencePanel.getByRole("tab", { name: "Rule trace" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Hit branch selected",
  );
  await evidencePanel.getByRole("tab", { name: "Audit" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "state changed",
  );
  await evidencePanel.getByRole("tab", { name: "State" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Raider · 9/18 HP",
  );

  await invokeApplicationCommand(page, "Run", "Advance turn");
  await expect(
    initiativePanel.getByRole("listitem", { name: "Scout, Current" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Raider, Next" }),
  ).toBeVisible();
  await expect(statusPanel).toContainText("entity-scout");
  await invokeApplicationCommand(page, "Run", "End combat");
  await expect(statusPanel).toContainText("Ended");
  await expect(
    initiativePanel.getByRole("listitem", { name: "Adept, Complete" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Raider, Complete" }),
  ).toBeVisible();
  await invokeApplicationCommand(page, "Run", "Close session");
  await expect(statusPanel).toContainText("Not selected");
  await invokeApplicationCommand(page, "Replay", "Replay archive");
  const replayDialog = page.getByRole("dialog", { name: "Replay archive" });
  const replayWorkspace = replayDialog.getByRole("region", {
    name: "Replay archive controls",
  });
  const liveReplay = replayWorkspace
    .getByLabel("Archived replay packages")
    .getByRole("button", { name: new RegExp(`live-${sessionId} ·`) });
  await expect(liveReplay).toBeVisible();
  await liveReplay.click();
  await expect(
    replayWorkspace.getByRole("region", { name: "Replay verification" }),
  ).toContainText("Verified · Finalized");
  await replayWorkspace
    .getByRole("region", { name: "Replay package detail" })
    .getByRole("button", { name: /panel-command-1/ })
    .click();
  await expect(
    replayWorkspace.getByRole("region", { name: "Replay command evidence" }),
  ).toContainText("Move");
  await replayDialog.getByLabel("Close", { exact: true }).click();
});

test("renders restored and quarantined recovery states without conflating them", async ({
  page,
}) => {
  await page.route("**/api/rulebench/v1/session-recovery", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({
        sessions: [
          {
            sessionId: "restored-reaction-session",
            origin: "restored",
            state: "recoverable",
            generation: 3,
            lastVerifiedFrameId: "3:verified-fingerprint",
            pendingReactionWindowId: "reaction-window-1",
            actions: ["discard", "fork"],
          },
        ],
        issues: [
          {
            code: "sessionRecoveryFrameMismatch",
            message: "Stored authority frame did not verify.",
            path: "session-recovery/quarantine.json",
          },
        ],
      }),
    });
  });
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  const recovery = workspace.getByRole("region", {
    name: "Session recovery",
  });

  await expect(recovery).toContainText(
    "restored-reaction-session · restored after restart · generation 3 · suspended reaction reaction-window-1",
  );
  await expect(recovery.getByRole("alert")).toContainText(
    "Unrecoverable · sessionRecoveryFrameMismatch · Stored authority frame did not verify.",
  );
  await recovery.scrollIntoViewIfNeeded();
  await recovery.screenshot({
    path: "dist/.playwright/session-recovery-states.png",
  });
});

test("resolves a bounded area target set and renders every v2 result @live", async ({
  page,
}) => {
  const sessionId = `e2e-operation-pipeline-v2-${Date.now()}`;
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  await workspace
    .getByRole("button", { name: "Ruined Watchtower Skirmish", exact: true })
    .click();
  await workspace.getByLabel("Session", { exact: true }).fill(sessionId);
  await workspace.getByRole("button", { name: "Create session" }).click();
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();
  await invokeApplicationCommand(page, "Run", "Start combat");

  const actionsPanel = page.getByRole("region", {
    name: "6. Available actions",
  });
  const gridPanel = page.getByRole("region", { name: "1. Combat grid" });
  const unitsPanel = page.getByRole("region", { name: "7. Active units" });
  await actionsPanel
    .getByRole("button", { name: "Select Storm Pulse", exact: true })
    .click();
  await expect(actionsPanel).toContainText("Shared");
  const area = gridPanel.getByRole("gridcell", {
    name: /^Target area at Coordinate 8, 3/,
  });
  await area.click();
  await expect(area).toHaveAttribute("aria-pressed", "true");
  await actionsPanel
    .getByRole("button", { name: "Preflight", exact: true })
    .click();
  const commandEvidence = actionsPanel.getByRole("region", {
    name: "Command decision evidence",
  });
  await expect(commandEvidence).toContainText("Accepted");
  await actionsPanel
    .getByRole("button", { name: "Submit", exact: true })
    .click();

  await expect(commandEvidence.getByTestId("target-result")).toHaveCount(2);
  await expect(commandEvidence).toContainText("Bruiser · Hit · 7 damage");
  await expect(commandEvidence).toContainText("Raider · Hit · 7 damage");
  await expect(commandEvidence).toContainText("Push 8,4 → 9,4");
  await expect(commandEvidence).toContainText("standard-action 1 → 0 (-1)");
  await expect(
    unitsPanel.getByRole("listitem", { name: /Bruiser, Active/ }),
  ).toContainText("11/18 HP");
  await expect(
    unitsPanel.getByRole("listitem", { name: /Raider, Active/ }),
  ).toContainText("11/18 HP");
  await page.screenshot({
    path: "dist/.playwright/operation-pipeline-v2-area.png",
    fullPage: true,
  });

  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
});

test("runs and archives the second compiled ruleset through the visible workbench @live", async ({
  page,
}) => {
  const sessionId = `e2e-turn-control-manual-${Date.now()}`;
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  await workspace
    .getByRole("button", { name: "Binding Glyph Failed Save", exact: true })
    .click();
  await expect(workspace.getByLabel("Scenario setup")).toContainText(
    "asha-rulebench.turn-control.v0",
  );
  await expect(workspace.getByLabel("Scenario setup")).toContainText(
    "Warden · wardens · initiative 20",
  );
  await expect(workspace.getByLabel("Scenario setup")).toContainText(
    "Scout · wardens · initiative 15",
  );
  await expect(workspace.getByLabel("Scenario setup")).toContainText(
    "Saboteur · invaders · initiative 10",
  );
  await workspace.getByLabel("Session", { exact: true }).fill(sessionId);
  await workspace.getByRole("button", { name: "Create session" }).click();
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();

  await invokeApplicationCommand(page, "Run", "Start combat");
  const actionsPanel = page.getByRole("region", {
    name: "6. Available actions",
  });
  const unitsPanel = page.getByRole("region", { name: "7. Active units" });
  await actionsPanel
    .getByRole("button", { name: "Select Binding Glyph", exact: true })
    .click();
  await expect(actionsPanel.getByLabel("Saving throw roll")).toBeVisible();
  await actionsPanel.getByLabel("Saving throw roll").fill("5");
  await actionsPanel.getByLabel("Damage roll").fill("4");
  await unitsPanel
    .getByRole("button", { name: "Select Saboteur as target" })
    .click();
  await actionsPanel
    .getByRole("button", { name: "Preflight", exact: true })
    .click();
  const commandEvidence = actionsPanel.getByRole("region", {
    name: "Command decision evidence",
  });
  await expect(commandEvidence).toContainText("Accepted");
  await actionsPanel
    .getByRole("button", { name: "Submit", exact: true })
    .click();
  await expect(commandEvidence).toContainText("Saving Throw Resolved");
  await expect(commandEvidence).toContainText("Modifier Applied");
  await expect(
    unitsPanel.getByRole("listitem", { name: /Saboteur, Active/ }),
  ).toContainText("12/18 HP");
  await page.screenshot({
    path: "dist/.playwright/second-ruleset-manual.png",
    fullPage: true,
  });

  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
  await invokeApplicationCommand(page, "Replay", "Replay archive");
  const replayWorkspace = page
    .getByRole("dialog", { name: "Replay archive" })
    .getByRole("region", { name: "Replay archive controls" });
  const liveReplay = replayWorkspace
    .getByLabel("Archived replay packages")
    .getByRole("button", { name: new RegExp(`live-${sessionId} ·`) });
  await expect(liveReplay).toBeVisible();
  await liveReplay.click();
  await expect(
    replayWorkspace.getByRole("region", { name: "Replay verification" }),
  ).toContainText("Verified · Finalized");
});

test("runs the second compiled ruleset through automatic policy controls @live", async ({
  page,
}) => {
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  await workspace
    .getByRole("button", { name: "Binding Glyph Failed Save", exact: true })
    .click();
  await workspace
    .getByLabel("Session", { exact: true })
    .fill(`e2e-turn-control-automatic-${Date.now()}`);
  await workspace.getByRole("button", { name: "Create session" }).click();
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();
  await invokeApplicationCommand(page, "Run", "Start combat");
  await invokeApplicationCommand(page, "Run", "Configure automatic run");
  const configuration = page.getByRole("dialog", {
    name: "Automatic run configuration",
  });
  await configuration.getByLabel("Max steps").fill("1");
  await configuration.getByRole("radio", { name: /Supplied rolls/ }).check();
  await configuration.getByLabel("Roll stream").fill("5,4");
  await configuration.getByLabel("Close", { exact: true }).click();
  await invokeApplicationCommand(page, "Run", "Run bounded combat");

  const evidencePanel = page.getByRole("region", { name: "5. Evidence log" });
  await evidencePanel.getByRole("tab", { name: "Audit" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Stopped At Max Steps",
  );
  await expect(
    page.getByRole("region", { name: "7. Active units" }),
  ).toContainText("12/18 HP");
  await page.screenshot({
    path: "dist/.playwright/second-ruleset-automatic.png",
    fullPage: true,
  });
  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
});

test("completes and archives a Rust-owned gameplay-fabric reaction @live", async ({
  page,
}) => {
  const sessionId = `e2e-visible-gameplay-fabric-${Date.now()}`;
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  await workspace
    .getByRole("button", { name: "Hexing Bolt Reaction", exact: true })
    .click();
  await workspace.getByLabel("Session", { exact: true }).fill(sessionId);
  await workspace.getByRole("button", { name: "Create session" }).click();
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();

  await invokeApplicationCommand(page, "Run", "Start combat");
  const actionsPanel = page.getByRole("region", {
    name: "6. Available actions",
  });
  const gridPanel = page.getByRole("region", { name: "1. Combat grid" });
  const unitsPanel = page.getByRole("region", { name: "7. Active units" });
  await actionsPanel
    .getByRole("button", { name: "Select Hexing Bolt" })
    .click();
  await gridPanel
    .getByRole("gridcell", { name: /^Target at .*occupied by Raider/ })
    .click();
  await actionsPanel
    .getByRole("button", { name: "Submit", exact: true })
    .click();

  await expect(
    unitsPanel.getByRole("listitem", {
      name: /Raider, Active, selected target/,
    }),
  ).toContainText("18/18 HP");
  const evidencePanel = page.getByRole("region", { name: "5. Evidence log" });
  await evidencePanel.getByRole("tab", { name: "Audit" }).click();
  const gameplayEvidence = evidencePanel.getByTestId(
    "gameplay-fabric-evidence",
  );
  await expect(gameplayEvidence).toContainText("1 decisions");
  await expect(gameplayEvidence).toContainText("0 state frames");
  await expect(gameplayEvidence).toContainText("1 pending");
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Gameplay decision · panel-command-1",
  );
  await expect(evidencePanel.getByRole("tabpanel")).toContainText("Suspended");
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "2 declared reads",
  );
  const reaction = actionsPanel.getByRole("region", {
    name: "Reaction response",
  });
  await expect(reaction).toContainText("entity-adept must pass");
  await reaction.getByRole("button", { name: "Pass reaction" }).click();
  await expect(reaction).toContainText("entity-raider must pass");
  await page.screenshot({
    path: "dist/.playwright/gameplay-fabric-reaction-open.png",
    fullPage: true,
  });
  await reaction.getByRole("button", { name: /Use Raider Ward/ }).click();
  await expect(reaction).toHaveCount(0);
  await expect(
    unitsPanel.getByRole("listitem", {
      name: /Raider, Active/,
    }),
  ).toContainText("11/18 HP");
  await expect(gameplayEvidence).toContainText("2 decisions");
  await expect(gameplayEvidence).toContainText("1 state frames");
  await expect(gameplayEvidence).toContainText("0 pending");
  await expect(evidencePanel.getByRole("tabpanel")).toContainText("Accepted");
  await page.screenshot({
    path: "dist/.playwright/gameplay-fabric-reaction-resolved.png",
    fullPage: true,
  });
  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
  await expect(
    page.getByRole("region", { name: "4. Turn status" }),
  ).toContainText("Not selected");
  await invokeApplicationCommand(page, "Replay", "Replay archive");
  const replayDialog = page.getByRole("dialog", { name: "Replay archive" });
  const replayWorkspace = replayDialog.getByRole("region", {
    name: "Replay archive controls",
  });
  const liveReplay = replayWorkspace
    .getByLabel("Archived replay packages")
    .getByRole("button", { name: new RegExp(`live-${sessionId} ·`) });
  await expect(liveReplay).toBeVisible();
  await liveReplay.click();
  await expect(liveReplay).toHaveAttribute("aria-pressed", "true");
  await expect(
    replayWorkspace
      .getByRole("region", { name: "Replay package detail" })
      .getByRole("heading", { name: `live-${sessionId}` }),
  ).toBeVisible();
  await expect(
    replayWorkspace.getByRole("region", { name: "Replay verification" }),
  ).toContainText("Verified · Finalized");
  await page.screenshot({
    path: "dist/.playwright/gameplay-fabric-evidence.png",
    fullPage: true,
  });
});

test("shows Rust automatic step and bounded-run decisions", async ({
  page,
}) => {
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  await expect(
    workspace.getByText("asha-rulebench.local-authority.v0"),
  ).toBeVisible();
  await workspace
    .getByRole("button", { name: "Hexing Bolt Hit", exact: true })
    .click();
  await workspace
    .getByLabel("Session", { exact: true })
    .fill(`e2e-visible-automatic-session-${Date.now()}`);
  await workspace.getByRole("button", { name: "Create session" }).click();
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();
  await invokeApplicationCommand(page, "Run", "Start combat");
  await invokeApplicationCommand(page, "Run", "Configure automatic run");
  const configuration = page.getByRole("dialog", {
    name: "Automatic run configuration",
  });
  await expect(configuration).toContainText("not AI");
  await configuration.getByLabel("Max steps").fill("1");
  await configuration
    .getByRole("radio", { name: /Authority-generated rolls/ })
    .check();
  await expect(configuration.getByLabel("Roll stream")).toHaveCount(0);
  await expect(configuration).toContainText("generates rolls lazily");
  await configuration.getByRole("radio", { name: "Advance turn" }).check();
  await configuration.getByLabel("Close", { exact: true }).click();

  await invokeApplicationCommand(page, "Run", "Run one policy step");
  const evidencePanel = page.getByRole("region", { name: "5. Evidence log" });
  await evidencePanel.getByRole("tab", { name: "Audit" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Submit Candidate",
  );

  await invokeApplicationCommand(page, "Run", "Run bounded combat");
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Stopped At Max Steps",
  );
  await expect(evidencePanel.getByRole("tabpanel")).toContainText("1/1 steps");

  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
  await expect(
    page.getByRole("region", { name: "4. Turn status" }),
  ).toContainText("Not selected");
});

test("configures, monitors, cancels, compares, and opens policy laboratory trials", async ({
  page,
}, testInfo) => {
  await page.goto("/");
  await invokeApplicationCommand(page, "Replay", "Policy laboratory");
  const dialog = page.getByRole("dialog", {
    name: "Deterministic policy laboratory",
  });
  const laboratory = dialog.getByRole("region", {
    name: "Deterministic policy laboratory",
  });
  await expect(laboratory).toBeVisible();
  await laboratory
    .getByLabel("Scenario and ruleset")
    .selectOption("watchtower-vitality-operations");
  await laboratory
    .getByLabel("Primary policy")
    .selectOption("firstAcceptedCandidate");
  await laboratory
    .getByLabel("Comparison policy")
    .selectOption("lowestVitalityTarget");
  await laboratory.getByLabel("Roll seeds").fill("7");
  await laboratory.getByLabel("Max steps per trial").fill("8");
  await laboratory.getByRole("button", { name: "Create bounded matrix" }).click();

  const experiment = laboratory.locator("article").last();
  await expect(experiment).toContainText("0 / 2 trials");
  await experiment.getByRole("button", { name: "Run next trial" }).click();
  await expect(experiment).toContainText("1 / 2 trials");
  await experiment.getByRole("button", { name: "Run next trial" }).click();
  await expect(experiment).toContainText("2 / 2 trials");
  await expect(experiment).toContainText("replay verified");
  await laboratory
    .getByRole("button", { name: "Compare first and latest trial" })
    .click();
  await expect(laboratory).toContainText("First divergence found");
  await experiment
    .getByRole("button", { name: "Open trial replay" })
    .first()
    .click();
  await expect(laboratory).toContainText(/Replay experiment-.* is open/);
  await page.screenshot({
    path: testInfo.outputPath("policy-laboratory-desktop.png"),
    fullPage: true,
  });

  await laboratory.getByLabel("Comparison policy").selectOption("");
  await laboratory.getByLabel("Roll seeds").fill("3,5");
  await laboratory.getByRole("button", { name: "Create bounded matrix" }).click();
  const cancellable = laboratory.locator("article").last();
  await expect(cancellable).toContainText("0 / 2 trials");
  await cancellable.getByRole("button", { name: "Cancel" }).click();
  await expect(cancellable).toContainText("cancelled");

  await laboratory
    .getByLabel("Scenario and ruleset")
    .selectOption("hexing-bolt-reaction");
  await laboratory.getByLabel("Roll seeds").fill("7");
  await laboratory.getByRole("button", { name: "Create bounded matrix" }).click();
  await expect(laboratory.getByRole("alert")).toContainText(
    "requires the explicit manual reaction workflow",
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Close" }).focus();
  await page.screenshot({
    path: testInfo.outputPath("policy-laboratory-mobile.png"),
    fullPage: true,
  });
});

test("configures participants from Rust scenario readbacks", async ({
  page,
}) => {
  const invalidThenValidSessionId = `e2e-visible-invalid-setup-${Date.now()}`;
  await page.goto("/");
  const workspace = await openLiveCombatWorkspace(page);
  await expect(
    workspace.getByText("asha-rulebench.local-authority.v0"),
  ).toBeVisible();
  await workspace
    .getByRole("button", { name: "Hexing Bolt Hit", exact: true })
    .click();

  const setup = workspace.getByLabel("Scenario setup");
  await expect(setup).toContainText("asha-rulebench.hexing-bolt.v0");
  await expect(setup).toContainText("Adept · ally · initiative 15");
  await expect(setup).toContainText("Raider · enemy · initiative 10");

  await setup.getByRole("button", { name: "Later" }).first().click();
  await workspace
    .getByLabel("Session", { exact: true })
    .fill(`e2e-reordered-setup-session-${Date.now()}`);
  await workspace.getByRole("button", { name: "Create session" }).click();
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();
  await expect(
    page.getByRole("region", { name: "4. Turn status" }),
  ).toContainText("entity-raider");
  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
  const nextWorkspace = await openLiveCombatWorkspace(page);

  await page.route("**/api/rulebench/v1/sessions", async (route) => {
    const body: unknown = route.request().postDataJSON();
    if (
      typeof body !== "object" ||
      body === null ||
      !("participantOrder" in body) ||
      !Array.isArray(body.participantOrder)
    ) {
      await route.continue();
      return;
    }
    const response = await route.fetch({
      postData: JSON.stringify({
        ...body,
        participantOrder: body.participantOrder.slice(0, 1),
      }),
    });
    await route.fulfill({ response });
  });
  await nextWorkspace
    .getByRole("textbox", { name: "Session" })
    .fill(invalidThenValidSessionId);
  await nextWorkspace.getByRole("button", { name: "Create session" }).click();
  await expect(nextWorkspace.getByRole("alert")).toContainText(
    "invalidRequest · Participant setup must include all 2 scenario participants exactly once.",
  );

  await page.unroute("**/api/rulebench/v1/sessions");
  await nextWorkspace.getByRole("button", { name: "Create session" }).click();
  await expect(nextWorkspace.getByRole("alert")).toHaveCount(0);
  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();
  await expect(
    page.getByRole("region", { name: "4. Turn status" }),
  ).toContainText(invalidThenValidSessionId);
  await invokeApplicationCommand(page, "Run", "End combat");
  await invokeApplicationCommand(page, "Run", "Close session");
});

test("reviews and compares archived Rust replay evidence", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("menuitem", { name: "Replay" }).click();
  await page.getByRole("menuitem", { name: "Replay archive" }).click();
  const dialog = page.getByRole("dialog", { name: "Replay archive" });
  const workspace = dialog.getByRole("region", {
    name: "Replay archive controls",
  });
  const packages = workspace.getByLabel("Archived replay packages");
  await expect(
    packages.getByRole("button", { name: /hexing-bolt-replay ·/ }).first(),
  ).toBeVisible();
  expect(await packages.getByRole("button").count()).toBeGreaterThanOrEqual(2);
  await packages
    .getByRole("button", { name: /hexing-bolt-replay ·/ })
    .first()
    .click();

  const detail = workspace.getByRole("region", {
    name: "Replay package detail",
  });
  await expect(detail).toContainText("Hexing Bolt Replay");
  await expect(detail.getByRole("button")).toHaveCount(2);
  await expect(
    workspace.getByRole("region", { name: "Replay verification" }),
  ).toContainText("Verified · Finalized");
  const comparison = workspace.getByRole("region", {
    name: "Replay comparison",
  });
  await comparison
    .getByLabel("Actual")
    .selectOption("hexing-bolt-replay-explicit-start");
  await comparison.getByRole("button", { name: "Compare" }).click();
  await expect(comparison).toContainText("Differences found");
  await expect(comparison).toContainText(
    "First difference · Replay Command Count Mismatch",
  );
  await expect(comparison).toContainText("commands.length");

  const command = workspace.getByRole("region", {
    name: "Replay command evidence",
  });
  await expect(command).toContainText("Supplied rolls · 17, 5");
  await expect(
    command.getByRole("region", { name: "Expected replay evidence" }),
  ).toContainText("Damage Applied");
  const actual = command.getByRole("region", {
    name: "Actual replay evidence",
  });
  await expect(actual).toContainText("Attack Roll");
  await expect(actual).toContainText("Resolution");
  const state = command.getByRole("region", { name: "Replay resulting state" });
  await expect(state).toContainText("Raider · 9/18 HP · Active");
  await expect(state).toContainText("Adept hits Raider");
  await expect(state).toContainText(
    "Accepted By Resolver · 4 events · 5 trace entries",
  );

  await detail
    .getByRole("button", { name: /2 · Control · explicit-end/ })
    .click();
  await expect(command).toContainText("No supplied rolls");
  await expect(state).toContainText("Ended");

  await dialog.getByRole("button", { name: "Close" }).click();
  await page.getByRole("tab", { name: "Replay" }).click();
  const replayPanel = page.getByRole("tabpanel", {
    name: "Replay",
  });
  await expect(replayPanel).toContainText("Hexing Bolt Replay");
  await expect(replayPanel).toContainText("Verified");
  await expect(replayPanel).toContainText(
    "First difference · Replay Command Count Mismatch",
  );
  await expect(
    replayPanel.getByRole("region", { name: "Expected replay summary" }),
  ).toBeVisible();
  await expect(
    replayPanel.getByRole("region", { name: "Actual replay summary" }),
  ).toBeVisible();
  await expect(replayPanel).toContainText("Resulting state");
});
