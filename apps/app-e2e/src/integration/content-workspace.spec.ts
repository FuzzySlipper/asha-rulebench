import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { createLiveRulebenchTransport } from "@asha-rulebench/transport";

test.afterEach(async ({ page }) => {
  const apiBaseUrl = new URL("/api/rulebench/v1", page.url()).toString();
  const transport = createLiveRulebenchTransport({ apiBaseUrl });
  const connected = await transport.connect();
  expect(connected.ok).toBe(true);

  const sessions = await transport.listSessions();
  expect(sessions.ok).toBe(true);
  if (sessions.ok) {
    for (const session of sessions.value) {
      if (!session.sessionId.startsWith("authored-e2e-session-")) continue;
      const ended = await transport.submitControl(session.sessionId, {
        kind: "explicitEnd",
      });
      expect(ended.ok).toBe(true);
      const closed = await transport.closeSession(session.sessionId);
      expect(closed.ok).toBe(true);
    }
  }

  const workspace = await transport.listContentWorkspace();
  expect(workspace.ok).toBe(true);
  if (workspace.ok) {
    for (const pack of workspace.value.packs) {
      if (!pack.reference.id.startsWith("pack.e2e.authored.")) continue;
      if (pack.active) {
        const deactivated = await transport.deactivateContent(pack.reference);
        expect(deactivated.ok).toBe(true);
      }
      const deleted = await transport.deleteContent(pack.reference);
      expect(deleted.ok).toBe(true);
    }
  }
  transport.disconnect();
});

test("authors, validates, binds, executes, and replays a Rust-owned action @live", async ({
  page,
}, testInfo) => {
  const identitySuffix = `${testInfo.workerIndex}.${testInfo.repeatEachIndex}.${testInfo.retry}.${Date.now()}`;
  const packId = `pack.e2e.authored.${identitySuffix}`;
  const cloneId = `pack.e2e.authored.clone.${identitySuffix}`;
  const sessionId = `authored-e2e-session-${identitySuffix}`;
  await page.goto("/");
  await openMenuDialog(page, "File", "Content packs");
  const contentDialog = page.getByRole("dialog", { name: "Content packs" });
  await expect(
    contentDialog.getByRole("region", {
      name: "Live authored content workspace",
    }),
  ).toBeVisible();
  await contentDialog.getByLabel("New pack id").fill(packId);
  await contentDialog.getByLabel("New pack version").fill("1.0.0");
  const templateButton = contentDialog.getByRole("button", {
    name: "Start Rust template",
  });
  await templateButton.focus();
  await templateButton.press("Enter");
  const editor = contentDialog.getByLabel("Authored JSON draft");
  await expect(editor).toHaveValue(new RegExp(`"id": "${packId.replaceAll(".", "\\.")}"`));
  await expect(contentDialog).toContainText(
    "JSON syntax is valid. Rust semantic validation has not been inferred.",
  );
  await expect(contentDialog).toContainText(
    "Not validated. JSON syntax alone does not claim semantic validity.",
  );

  const templatePayload = await editor.inputValue();
  await editor.fill("{");
  await expect(contentDialog.getByText("JSON syntax error")).toBeVisible();
  await expect(
    contentDialog.getByRole("button", { name: "Validate with Rust" }),
  ).toBeDisabled();

  const unsupportedEffect = templatePayload.replace(
    /\{\s*"operation": "damage",\s*"damageBonus": 4,\s*"damageType": "arcane"\s*\}/,
    JSON.stringify(
      {
        operation: "openReactionWindow",
        hookId: "e2e-unsupported-reaction",
        window: "afterEffect",
        eligibleReactors: ["declaredTargets"],
        options: [
          {
            id: "brace",
            reactor: "declaredTargets",
            opensNestedWindow: false,
          },
        ],
        maximumNestedDepth: 0,
      },
      null,
      2,
    ),
  );
  expect(unsupportedEffect).not.toBe(templatePayload);
  await editor.fill(unsupportedEffect);
  await contentDialog
    .getByRole("button", { name: "Validate with Rust" })
    .click();
  const semanticValidation = contentDialog.getByRole("region", {
    name: "Rust semantic validation",
  });
  await expect(semanticValidation).toContainText("Rejected");
  await expect(semanticValidation).toContainText(
    "unsupportedAuthoredActionEffect",
  );
  await expect(semanticValidation).toContainText("catalogs.actions[0].effects[0]");

  await editor.fill(templatePayload);
  await contentDialog
    .getByRole("button", { name: "Validate with Rust" })
    .click();
  await expect(semanticValidation).toContainText("Accepted");
  await contentDialog
    .getByRole("button", { name: "Import validated draft" })
    .click();
  await expect(
    contentDialog.getByText("Accepted", { exact: true }),
  ).toBeVisible();
  await expect(
    contentDialog.getByText(`${packId}@1.0.0`, { exact: true }).first(),
  ).toBeVisible();
  await contentDialog
    .getByRole("button", { name: "Activate exact set" })
    .click();
  const selectedReview = contentDialog.getByRole("region", {
    name: "Selected authored pack review",
  });
  await expect(
    selectedReview.getByText("Active", { exact: true }),
  ).toBeVisible();
  await expect(selectedReview).toContainText("Authored abilities");
  await expect(selectedReview).toContainText("ability.binding-glyph");
  await expect(selectedReview).toContainText("modifier.binding-glyph.anchored");
  await expect(selectedReview).toContainText("action.binding-glyph");
  await expect(selectedReview).toContainText("body saving throw vs DC 12");
  await expect(selectedReview).toContainText("apply modifier");
  await expect(
    contentDialog.getByText(/Content Activated/i).last(),
  ).toBeVisible();

  await contentDialog.screenshot({
    path: "dist/.playwright/authored-action-workflow-desktop.png",
  });
  await page.setViewportSize({ width: 640, height: 900 });
  await editor.scrollIntoViewIfNeeded();
  await contentDialog.screenshot({
    path: "dist/.playwright/authored-action-workflow-narrow.png",
  });
  await page.setViewportSize({ width: 1280, height: 900 });

  await contentDialog.getByLabel("New pack id").fill(cloneId);
  await contentDialog.getByLabel("New pack version").fill("2.0.0");
  await contentDialog.getByRole("button", { name: "Clone selected pack" }).click();
  await expect(editor).toHaveValue(new RegExp(`"id": "${cloneId.replaceAll(".", "\\.")}"`));
  await expect(contentDialog).toContainText(`Clone of ${packId}@1.0.0`);
  await expect(
    contentDialog.getByText(`${cloneId}@2.0.0`, { exact: true }),
  ).toHaveCount(0);

  await contentDialog.getByRole("button", { name: "Close" }).click();
  await openMenuDialog(page, "Scenario", "Live combat setup");
  const liveDialog = page.getByRole("dialog", { name: "Live combat setup" });
  await liveDialog
    .getByRole("button", { name: "Binding Glyph Failed Save", exact: true })
    .click();
  const authoredAction = liveDialog.getByRole("button", {
    name: new RegExp(`Binding Glyph · action\\.binding-glyph · ${packId.replaceAll(".", "\\.")}@1\\.0\\.0`),
  });
  await expect(authoredAction).toBeVisible();
  await authoredAction.click();
  const actor = liveDialog.getByRole("button", {
    name: "Warden · entity-warden",
  });
  await actor.focus();
  await actor.press("Enter");
  await liveDialog
    .getByRole("textbox", { name: "Session", exact: true })
    .fill(sessionId);
  await liveDialog.getByRole("button", { name: "Create session" }).click();
  await expect(liveDialog.getByRole("alert")).toHaveCount(0);
  await expect(
    liveDialog.getByText("Creating or loading live session"),
  ).toHaveCount(0);
  await liveDialog.getByRole("button", { name: "Close" }).click();

  const statusPanel = page.getByRole("region", { name: "4. Turn status" });
  await expect(statusPanel).toContainText("action.binding-glyph · entity-warden");
  await openMenuCommand(page, "Run", "Start combat");
  const actionsPanel = page.getByRole("region", {
    name: "6. Available actions",
  });
  const unitsPanel = page.getByRole("region", { name: "7. Active units" });
  await actionsPanel
    .getByRole("button", {
      name: "Select Binding Glyph · action.binding-glyph",
      exact: true,
    })
    .click();
  await actionsPanel.getByLabel("Saving throw roll").fill("5");
  await actionsPanel.getByLabel("Damage roll").fill("4");
  await unitsPanel
    .getByRole("button", { name: "Select Saboteur as target" })
    .click();
  await actionsPanel
    .getByRole("button", { name: "Submit", exact: true })
    .click();
  const commandEvidence = actionsPanel.getByRole("region", {
    name: "Command decision evidence",
  });
  await expect(commandEvidence).toContainText("Saving Throw Resolved");
  await expect(commandEvidence).toContainText("Modifier Applied");
  const evidencePanel = page.getByRole("region", { name: "5. Evidence log" });
  await evidencePanel.getByRole("tab", { name: "State" }).click();
  await expect(
    evidencePanel.getByRole("region", { name: "Live authored action binding" }),
  ).toContainText("action.binding-glyph");
  await expect(
    evidencePanel.getByRole("region", { name: "Live authored action binding" }),
  ).toContainText(`${packId}@1.0.0`);

  await openMenuCommand(page, "Run", "End combat");
  await openMenuCommand(page, "Run", "Close session");
  await openMenuDialog(page, "Replay", "Replay archive");
  const replayDialog = page.getByRole("dialog", { name: "Replay archive" });
  const replayWorkspace = replayDialog.getByRole("region", {
    name: "Replay archive controls",
  });
  const replay = replayWorkspace
    .getByLabel("Archived replay packages")
    .getByRole("button", { name: new RegExp(`live-${sessionId} ·`) });
  await expect(replay).toBeVisible();
  await replay.click();
  const replayBinding = replayWorkspace.getByRole("region", {
    name: "Replay authored action binding",
  });
  await expect(replayBinding).toContainText("action.binding-glyph");
  await expect(replayBinding).toContainText("ability.binding-glyph");
  await expect(replayBinding).toContainText("entity-warden");
  await expect(replayBinding).toContainText(`${packId}@1.0.0`);
  await expect(replayBinding).toContainText("targeting 2 · check 1 · effects 1");
});

async function openMenuDialog(
  page: Page,
  menuName: string,
  itemName: string,
): Promise<void> {
  await page
    .getByRole("menubar", { name: "Rulebench application menu" })
    .getByRole("menuitem", { name: menuName })
    .click();
  await page
    .getByRole("menu", { name: menuName })
    .getByRole("menuitem", { name: itemName })
    .click();
}

async function openMenuCommand(
  page: Page,
  menuName: string,
  itemName: string,
): Promise<void> {
  await openMenuDialog(page, menuName, itemName);
}
