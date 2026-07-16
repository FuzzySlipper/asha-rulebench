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
    }
  }
  transport.disconnect();
});

test("imports, activates, preserves, and selects Rust-owned authored content", async ({
  page,
}, testInfo) => {
  const identitySuffix = `${testInfo.workerIndex}.${testInfo.repeatEachIndex}.${testInfo.retry}`;
  const packId = `pack.e2e.authored.${identitySuffix}`;
  const sessionId = `authored-e2e-session-${identitySuffix}`;
  await page.goto("/");
  await openMenuDialog(page, "File", "Content packs");
  const contentDialog = page.getByRole("dialog", { name: "Content packs" });
  await expect(
    contentDialog.getByRole("region", {
      name: "Live authored content workspace",
    }),
  ).toBeVisible();
  await contentDialog.locator('input[type="file"]').setInputFiles({
    name: "authored-pack.json",
    mimeType: "application/json",
    buffer: Buffer.from(authoredPack(packId, 2, "Live Authored Pack")),
  });
  await contentDialog
    .getByRole("button", { name: "Import", exact: true })
    .click();
  await expect(
    contentDialog.getByText("Accepted", { exact: true }),
  ).toBeVisible();
  await expect(
    contentDialog.getByRole("button", {
      name: `Live Authored Pack ${packId}@1.0.0`,
    }),
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
  await expect(selectedReview).toContainText("ability");
  await expect(selectedReview).toContainText("ability.binding-glyph");
  await expect(
    contentDialog.getByText(/Content Activated/i).last(),
  ).toBeVisible();

  await contentDialog.locator('input[type="file"]').setInputFiles({
    name: "unsupported-pack.json",
    mimeType: "application/json",
    buffer: Buffer.from(authoredPack(packId, 99, "Unsupported Replacement")),
  });
  await contentDialog
    .getByRole("button", { name: "Import", exact: true })
    .click();
  await expect(
    contentDialog.getByText("Rejected", { exact: true }),
  ).toBeVisible();
  await expect(
    contentDialog.getByText("unsupportedAuthoredContentVersion"),
  ).toBeVisible();
  await expect(
    selectedReview.getByText("Active", { exact: true }),
  ).toBeVisible();

  await contentDialog.getByRole("button", { name: "Close" }).click();
  await openMenuDialog(page, "Scenario", "Live combat setup");
  const liveDialog = page.getByRole("dialog", { name: "Live combat setup" });
  await liveDialog
    .getByRole("button", { name: "Binding Glyph Failed Save", exact: true })
    .click();
  await expect(
    liveDialog.getByRole("button", { name: `${packId}@1.0.0` }),
  ).toBeVisible();
  await liveDialog.getByRole("button", { name: `${packId}@1.0.0` }).click();
  await liveDialog
    .getByRole("textbox", { name: "Session", exact: true })
    .fill(sessionId);
  await liveDialog.getByRole("button", { name: "Create session" }).click();
  await expect(liveDialog.getByRole("alert")).toHaveCount(0);
  await expect(
    liveDialog.getByText("Creating or loading live session"),
  ).toHaveCount(0);
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

function authoredPack(
  packId: string,
  formatVersion: number,
  title: string,
): string {
  return JSON.stringify({
    format: "asha-rulebench.content-pack",
    formatVersion,
    pack: {
      id: packId,
      version: "1.0.0",
      title,
      summary: "Visible authored content lifecycle proof.",
      tags: ["e2e", "authored"],
      provenance: {
        sourceKind: "authoredFile",
        sourceId: `e2e:${packId}`,
        authoredBy: "Playwright",
      },
      rulesetId: "asha-rulebench.turn-control.v0",
      dependencies: [],
      catalogs: {
        rulesets: [
          {
            id: "asha-rulebench.turn-control.v0",
            name: "E2E Objective Turn Control Ruleset",
            version: "0.1.0",
            summary: "Matches the visible second-provider scenario.",
            modules: [
              {
                module: "actionResolution",
                version: "1",
                configuration: {
                  module: "actionResolution",
                  targetingPolicy: "declaredTargetsAndLineOfSight",
                  supportedCheckHandlers: ["attackVsDefense", "savingThrow"],
                },
              },
              {
                module: "turnControl",
                version: "1",
                configuration: {
                  module: "turnControl",
                  turnOrderPolicy: "explicit",
                  combatEndPolicy: "objectiveSideVictory",
                  objectiveSide: "wardens",
                },
              },
            ],
          },
        ],
        entities: [
          {
            id: "entity.e2e.authored",
            name: "E2E Authored Entity",
            summary: "Generic definition review proof.",
            tags: ["e2e"],
            damageAdjustments: [],
          },
        ],
        abilities: [
          {
            id: "ability.binding-glyph",
            name: "Binding Glyph",
            kind: "spell",
            summary: "Forces a Body save, then damages and anchors on failure.",
            tags: ["save", "control"],
          },
        ],
      },
    },
  });
}
