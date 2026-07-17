import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { readFile } from "node:fs/promises";
import type { RulebenchContentPackReferenceDto } from "@asha-rulebench/protocol";
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

test("materializes Shatterline v4 scenario provenance and composition in the live UI @gate", async ({
  page,
}) => {
  await page.goto("/");
  const apiBaseUrl = new URL("/api/rulebench/v1", page.url()).toString();
  const transport = createLiveRulebenchTransport({ apiBaseUrl });
  const nonce = Date.now().toString();
  const packId = `pack.e2e.shatterline.foundation-${nonce}`;
  const sessionId = `e2e-shatterline-foundation-${nonce}`;
  const fixture = await readFile(
    new URL(
      "../../../../rulebench-rs/hosts/rulebench-process-host/src/fixtures/shatterline-foundation-v4.json",
      import.meta.url,
    ),
    "utf8",
  );
  const payload = fixture
    .replace("pack.shatterline.foundation", packId)
    .replace(
      "fixture:shatterline-foundation-v4",
      `fixture:e2e-shatterline-foundation-v4-${nonce}`,
    );
  let reference: RulebenchContentPackReferenceDto | undefined;
  let sessionExists = false;

  try {
    const connected = await transport.connect();
    expect(connected.ok).toBe(true);
    const imported = await transport.importContent(payload, "reject");
    expect(imported.ok).toBe(true);
    if (!imported.ok || imported.value.outcome === null) return;
    reference = imported.value.outcome.review.pack.reference;
    const activated = await transport.activateContent(reference);
    expect(activated.ok).toBe(true);
    if (!activated.ok) return;
    expect(
      activated.value.packs.find(
        (pack) =>
          pack.reference.fingerprint.value === reference?.fingerprint.value,
      )?.active,
    ).toBe(true);
    const scenarios = await transport.listScenarios();
    expect(scenarios.ok).toBe(true);
    if (!scenarios.ok) return;
    expect(scenarios.value.map((scenario) => scenario.id)).toEqual(
      expect.arrayContaining([
        "shatterline-foundation-manual",
        "shatterline-foundation-automatic",
      ]),
    );

    const setup = await openLiveCombatWorkspace(page);
    await setup.getByRole("button", { name: "Connect", exact: true }).click();
    await setup
      .getByRole("button", {
        name: "Shatterline Foundation Automatic",
        exact: true,
      })
      .click();
    await expect(
      setup.getByText("automatic · firstAcceptedCandidate v1", {
        exact: true,
      }),
    ).toBeVisible();
    await setup
      .getByRole("button", {
        name: "Shatterline Foundation Manual",
        exact: true,
      })
      .click();
    await expect(setup.getByText("manual", { exact: true })).toBeVisible();
    await expect(setup.getByText(`${packId} · 4.0.0`)).toBeVisible();
    await expect(
      setup.getByRole("button", { name: `${packId}@4.0.0` }),
    ).toHaveAttribute("aria-pressed", "true");
    const createSession = setup.getByRole("button", {
      name: "Create session",
    });
    await expect(createSession).toBeEnabled();

    await page.setViewportSize({ width: 1280, height: 900 });
    await setup.screenshot({
      path: "dist/.playwright/shatterline-foundation-setup-desktop.png",
    });
    await page.setViewportSize({ width: 640, height: 900 });
    await setup.screenshot({
      path: "dist/.playwright/shatterline-foundation-setup-narrow.png",
    });

    await setup
      .getByRole("textbox", { name: "Session", exact: true })
      .fill(sessionId);
    await createSession.click();
    await expect(setup.getByRole("alert")).toHaveCount(0);
    sessionExists = true;
    await page
      .getByRole("dialog", { name: "Live combat setup" })
      .getByRole("button", { name: "Close" })
      .click();
    const evidence = page.getByRole("region", { name: "5. Evidence log" });
    await evidence.getByRole("tab", { name: "State" }).click();
    const composition = evidence.getByRole("region", {
      name: "Live authored scenario composition",
    });
    await expect(composition).toContainText(
      "shatterline-foundation-manual · manual",
    );
    await expect(composition).toContainText("archetype.anchor@1 · level 1");
    await expect(composition).toContainText(
      "action.anchor-lash → foundation-anchor-lash",
    );
    await expect(composition).toContainText(
      "action.binding-spark → foundation-binding-spark",
    );
    await page.screenshot({
      path: "dist/.playwright/shatterline-foundation-receipt-narrow.png",
      fullPage: true,
    });
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.screenshot({
      path: "dist/.playwright/shatterline-foundation-receipt-desktop.png",
      fullPage: true,
    });
  } finally {
    if (sessionExists) {
      await transport.submitControl(sessionId, { kind: "explicitEnd" });
      await transport.closeSession(sessionId);
    }
    if (reference !== undefined) {
      await transport.deactivateContent(reference);
      await transport.deleteContent(reference);
    }
    transport.disconnect();
  }
});



test("completes a supported scenario through the visible panel workbench @gate", async ({
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
  const recoveryEntry = recovery
    .locator(".choice-row")
    .filter({ hasText: `${sessionId} · new this process · generation 0` });
  await expect(recoveryEntry).toHaveCount(1);
  await expect(
    recoveryEntry.getByRole("button", { name: "Fork" }),
  ).toBeVisible();
  await expect(
    recoveryEntry.getByRole("button", { name: "Discard" }),
  ).toBeVisible();

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



test("reviews and compares archived Rust replay evidence @gate", async ({
  page,
}) => {
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
