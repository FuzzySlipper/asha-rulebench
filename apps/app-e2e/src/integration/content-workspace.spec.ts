import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

test("imports, activates, preserves, and selects Rust-owned authored content", async ({
  page,
}) => {
  await page.goto("/");
  await openMenuDialog(page, "File", "Content packs");
  const contentDialog = page.getByRole("dialog", { name: "Content packs" });
  await expect(
    contentDialog.getByRole("region", { name: "Live authored content workspace" }),
  ).toBeVisible();
  await expect(contentDialog.getByText("No authored packs stored.")).toBeVisible();

  await contentDialog.locator('input[type="file"]').setInputFiles({
    name: "authored-pack.json",
    mimeType: "application/json",
    buffer: Buffer.from(authoredPack(1, "Live Authored Pack")),
  });
  await contentDialog.getByRole("button", { name: "Import", exact: true }).click();
  await expect(contentDialog.getByText("Accepted", { exact: true })).toBeVisible();
  await expect(
    contentDialog.getByRole("button", { name: /Live Authored Pack pack\.e2e\.authored@1\.0\.0/ }),
  ).toBeVisible();
  await contentDialog.getByRole("button", { name: "Activate exact set" }).click();
  const selectedReview = contentDialog.getByRole("region", {
    name: "Selected authored pack review",
  });
  await expect(selectedReview.getByText("Active", { exact: true })).toBeVisible();
  await expect(contentDialog.getByText(/Content Activated/i)).toBeVisible();

  await contentDialog.locator('input[type="file"]').setInputFiles({
    name: "unsupported-pack.json",
    mimeType: "application/json",
    buffer: Buffer.from(authoredPack(99, "Unsupported Replacement")),
  });
  await contentDialog.getByRole("button", { name: "Import", exact: true }).click();
  await expect(contentDialog.getByText("Rejected", { exact: true })).toBeVisible();
  await expect(contentDialog.getByText("unsupportedAuthoredContentVersion")).toBeVisible();
  await expect(selectedReview.getByText("Active", { exact: true })).toBeVisible();

  await contentDialog.getByRole("button", { name: "Close" }).click();
  await openMenuDialog(page, "Scenario", "Live combat setup");
  const liveDialog = page.getByRole("dialog", { name: "Live combat setup" });
  await expect(liveDialog.getByRole("button", { name: "pack.e2e.authored@1.0.0" })).toBeVisible();
  await liveDialog.getByRole("button", { name: "pack.e2e.authored@1.0.0" }).click();
  await liveDialog.getByLabel("Session").fill("authored-e2e-session");
  await liveDialog.getByRole("button", { name: "Create session" }).click();
  await expect(liveDialog.getByRole("alert")).toHaveCount(0);
  await expect(liveDialog.getByText("Creating or loading live session")).toHaveCount(0);
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

function authoredPack(formatVersion: number, title: string): string {
  return JSON.stringify({
    format: "asha-rulebench.content-pack",
    formatVersion,
    pack: {
      id: "pack.e2e.authored",
      version: "1.0.0",
      title,
      summary: "Visible authored content lifecycle proof.",
      tags: ["e2e", "authored"],
      provenance: {
        sourceKind: "authoredFile",
        sourceId: "e2e:authored-pack",
        authoredBy: "Playwright",
      },
      rulesetId: "asha-rulebench.hexing-bolt.v0",
      dependencies: [],
      catalogs: {
        rulesets: [
          {
            id: "asha-rulebench.hexing-bolt.v0",
            name: "E2E Compatible Ruleset",
            version: "0.0.0",
            summary: "Matches the visible Hexing Bolt scenario.",
            modules: [
              {
                module: "actionResolution",
                version: "1",
                configuration: {
                  module: "actionResolution",
                  targetingPolicy: "declaredTargetsAndLineOfSight",
                  supportedCheckHandlers: ["attackVsDefense"],
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
      },
    },
  });
}
