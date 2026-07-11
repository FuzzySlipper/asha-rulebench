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
  const sessionId = "e2e-live-rust-session";
  let sessionExists = false;

  try {
    const connected = await transport.connect();
    expect(connected).toEqual({
      ok: true,
      value: {
        protocolId: "asha-rulebench.protocol",
        protocolVersion: 3,
        authoritySurface: "asha-rulebench.local-authority.v0",
      },
    });

    const scenarios = await transport.listScenarios();
    expect(scenarios.ok).toBe(true);
    if (!scenarios.ok) return;
    expect(scenarios.value.map((scenario) => scenario.id)).toContain(
      "hexing-bolt-hit",
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
        destinationCell: null,
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
    expect(remainingSessions).toEqual({ ok: true, value: [] });

    const replayPackages = await transport.listReplayPackages();
    expect(replayPackages.ok).toBe(true);
    if (!replayPackages.ok) return;
    expect(replayPackages.value).toHaveLength(2);
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
        kind: "bridge",
        code: "protocolVersionMismatch",
        message: "Unsupported protocol version 999; expected 3.",
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
    .fill("e2e-visible-panel-session");
  await workspace.getByRole("button", { name: "Create session" }).click();

  await page
    .getByRole("dialog", { name: "Live combat setup" })
    .getByLabel("Close", { exact: true })
    .click();

  const statusPanel = page.getByRole("region", { name: "4. Turn status" });
  const initiativePanel = page.getByRole("region", { name: "2. Initiative" });
  const actionsPanel = page.getByRole("region", {
    name: "6. Available actions",
  });
  const unitsPanel = page.getByRole("region", { name: "7. Active units" });
  await expect(statusPanel).toContainText("e2e-visible-panel-session");
  await expect(statusPanel).toContainText("Ready");
  await expect(
    initiativePanel.getByRole("listitem", { name: "Adept, Current" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Raider, Next" }),
  ).toBeVisible();

  await invokeApplicationCommand(page, "Run", "Start combat");
  await expect(statusPanel).toContainText("In Progress");
  await actionsPanel
    .getByRole("button", { name: "Select Hexing Bolt" })
    .click();
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
    initiativePanel.getByRole("listitem", { name: "Raider, Current" }),
  ).toBeVisible();
  await expect(
    initiativePanel.getByRole("listitem", { name: "Adept, Next" }),
  ).toBeVisible();
  await expect(statusPanel).toContainText("entity-raider");
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
    .fill("e2e-visible-automatic-session");
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
  await configuration.getByLabel("Roll stream").fill("17,5,2,5");
  await configuration.getByRole("radio", { name: "Advance turn" }).check();
  await configuration.getByLabel("Close", { exact: true }).click();

  await invokeApplicationCommand(page, "Run", "Run one policy step");
  const evidencePanel = page.getByRole("region", { name: "5. Evidence log" });
  await evidencePanel.getByRole("tab", { name: "Audit" }).click();
  await expect(evidencePanel.getByRole("tabpanel")).toContainText(
    "Submit Candidate",
  );
  await expect(
    page
      .getByRole("region", { name: "7. Active units" })
      .getByRole("listitem", { name: /Raider, Active/ }),
  ).toContainText("9/18 HP");

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

test("configures participants from Rust scenario readbacks", async ({
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

  const setup = workspace.getByLabel("Scenario setup");
  await expect(setup).toContainText("asha-rulebench.hexing-bolt.v0");
  await expect(setup).toContainText("Adept · ally · initiative 15");
  await expect(setup).toContainText("Raider · enemy · initiative 10");

  await setup.getByRole("button", { name: "Later" }).first().click();
  await workspace
    .getByLabel("Session", { exact: true })
    .fill("e2e-reordered-setup-session");
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
    .fill("e2e-visible-invalid-setup");
  await nextWorkspace.getByRole("button", { name: "Create session" }).click();
  await expect(nextWorkspace.getByRole("alert")).toContainText(
    "invalidRequest · Participant setup must include all 2 scenario participants exactly once.",
  );
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
  await expect(packages.getByRole("button")).toHaveCount(2);
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
