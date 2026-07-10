import { expect, test } from "@playwright/test";
import { createLiveRulebenchTransport } from "@asha-rulebench/transport";

test.describe.configure({ mode: "serial" });

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
        protocolVersion: 1,
        authoritySurface: "asha-rulebench.local-authority.v0",
      },
    });

    const scenarios = await transport.listScenarios();
    expect(scenarios.ok).toBe(true);
    if (!scenarios.ok) return;
    expect(scenarios.value.map((scenario) => scenario.id)).toContain(
      "hexing-bolt-hit",
    );

    const created = await transport.createSession({
      sessionId,
      scenarioId: "hexing-bolt-hit",
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
    expect(options.value.actions).toEqual([
      expect.objectContaining({
        actionId: "hexing_bolt",
        available: true,
        targets: [expect.objectContaining({ targetId: "entity-raider" })],
      }),
    ]);

    const executed = await transport.submitIntent(sessionId, {
      id: "e2e-hexing-bolt-hit",
      title: "E2E Hexing Bolt hit",
      summary: "Canonical live Rust authority invocation.",
      intent: {
        actorId: "entity-adept",
        actionId: "hexing_bolt",
        targetId: "entity-raider",
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
        message: "Unsupported protocol version 999; expected 1.",
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

test("completes a supported scenario through the visible manual workspace", async ({ page }) => {
  await page.goto("/");
  const workspace = page.getByRole("region", { name: "Live combat controls" });

  await expect(workspace.getByText("asha-rulebench.local-authority.v0")).toBeVisible();
  await workspace.getByRole("button", { name: "Hexing Bolt Hit", exact: true }).click();
  await workspace.getByLabel("Session").fill("e2e-visible-manual-session");
  await workspace.getByRole("button", { name: "Create session" }).click();

  const sessionState = workspace.getByRole("region", { name: "Live session state" });
  await expect(sessionState).toContainText("e2e-visible-manual-session · Ready");
  await workspace.getByRole("button", { name: "Start", exact: true }).click();
  await expect(sessionState).toContainText("In Progress");

  await workspace.getByRole("button", { name: "Hexing Bolt", exact: true }).click();
  await workspace.getByRole("button", { name: /Raider · 18\/18 HP/ }).click();
  await workspace.getByRole("button", { name: "Preflight", exact: true }).click();
  await expect(workspace.getByRole("region", { name: "Live preflight evidence" })).toContainText("Accepted");

  await workspace.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(sessionState).toContainText("Raider9/18 HP · Active");
  await expect(workspace.getByRole("region", { name: "Live combat log" })).toContainText("Damage Applied");
  await expect(workspace.getByRole("region", { name: "Live command audit" })).toContainText("state changed");

  await workspace.getByRole("button", { name: "End", exact: true }).click();
  await expect(sessionState).toContainText("Ended");
  await workspace.getByRole("button", { name: "Close", exact: true }).click();
  await expect(workspace.getByText("e2e-visible-manual-session · Ended")).toHaveCount(0);
});
