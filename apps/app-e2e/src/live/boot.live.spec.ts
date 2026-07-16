import type { Page } from "@playwright/test";
import { expect, liveScenario } from "./support/live-scenario";

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

async function openLiveSetup(page: Page) {
  await invokeApplicationCommand(page, "Scenario", "Live combat setup");
  const dialog = page.getByRole("dialog", { name: "Live combat setup" });
  return {
    dialog,
    controls: dialog.getByRole("region", {
      name: "Live combat setup controls",
    }),
  };
}

liveScenario(
  "boot live evidence @live",
  async ({ page, collector, liveBaseUrl }) => {
    const liveSessionId = `live-evidence-${Date.now()}`;
    const livePackId = `pack.live.${Date.now()}`;
    const angularTrackWarnings: string[] = [];
    page.on("console", (message) => {
      if (message.text().includes("NG0955")) {
        angularTrackWarnings.push(message.text());
      }
    });
    collector.addNonClaim(
      "This scenario proves one Rulebench fixture and the current capability manifest can be read through the live Rust host; it does not prove arbitrary rulesets, durable sessions, broad accessibility coverage, or performance.",
    );

    await page.goto(liveBaseUrl);
    await expect(page.getByLabel("Rulebench panel layout")).toBeVisible();

    await invokeApplicationCommand(page, "View", "Runtime capabilities");
    const capabilityDialog = page.getByRole("dialog", {
      name: "Runtime capabilities",
    });
    const capabilityMatrix = capabilityDialog.getByRole("table", {
      name: "Executable support matrix",
    });
    await expect(
      capabilityDialog.getByText("rulebench-process-host · filesystem"),
    ).toBeVisible();
    await expect(
      capabilityMatrix.getByRole("row", { name: /session\.active-recovery/ }),
    ).toContainText("Durable and regression covered");
    await collector.milestone("live rust capability manifest desktop", {
      screenshot: true,
      layerSnapshot: {
        manifest: await capabilityDialog.innerText(),
      },
    });
    await page.setViewportSize({ width: 390, height: 844 });
    await collector.milestone("live rust capability manifest mobile", {
      screenshot: true,
      layerSnapshot: {
        viewport: "390x844",
        manifest: await capabilityDialog.innerText(),
      },
    });
    await page.setViewportSize({ width: 1280, height: 900 });
    await capabilityDialog.getByRole("button", { name: "Close" }).click();

    await invokeApplicationCommand(page, "File", "Content packs");
    const contentDialog = page.getByRole("dialog", { name: "Content packs" });
    await expect(
      contentDialog.getByRole("heading", {
        name: "Live Authored Content",
        exact: true,
      }),
    ).toBeVisible();
    await contentDialog.locator('input[type="file"]').setInputFiles({
      name: "live-authored-pack.json",
      mimeType: "application/json",
      buffer: Buffer.from(authoredPack(livePackId, 1, "Live Evidence Pack")),
    });
    await contentDialog
      .getByRole("button", { name: "Import", exact: true })
      .click();
    await expect(
      contentDialog.getByText("Accepted", { exact: true }),
    ).toBeVisible();
    await contentDialog
      .getByRole("button", { name: "Activate exact set" })
      .click();
    const selectedContent = contentDialog.getByRole("region", {
      name: "Selected authored pack review",
    });
    await expect(
      selectedContent.getByText("Active", { exact: true }),
    ).toBeVisible();
    await collector.milestone("authored content active desktop", {
      screenshot: true,
      layerSnapshot: {
        selectedPack: await selectedContent.innerText(),
        lifecycleAudit: await contentDialog
          .getByRole("region", { name: "Content lifecycle audit" })
          .innerText(),
      },
    });
    await contentDialog.locator('input[type="file"]').setInputFiles({
      name: "unsupported-live-pack.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        authoredPack(livePackId, 99, "Unsupported Live Pack"),
      ),
    });
    await contentDialog
      .getByRole("button", { name: "Import", exact: true })
      .click();
    await expect(
      contentDialog.getByText("unsupportedAuthoredContentVersion"),
    ).toBeVisible();
    await expect(
      selectedContent.getByText("Active", { exact: true }),
    ).toBeVisible();
    await collector.milestone(
      "authored content rejection preserves active pack",
      {
        screenshot: true,
        layerSnapshot: {
          rejection: await contentDialog
            .getByRole("region", { name: "Import authored content" })
            .innerText(),
          retainedPack: await selectedContent.innerText(),
        },
      },
    );
    await page.setViewportSize({ width: 390, height: 844 });
    await collector.milestone("authored content mobile", {
      screenshot: true,
      layerSnapshot: {
        viewport: "390x844",
        selectedPack: await selectedContent.innerText(),
      },
    });
    await page.setViewportSize({ width: 1280, height: 900 });
    await contentDialog.getByRole("button", { name: "Close" }).click();

    const liveSetup = await openLiveSetup(page);
    await expect(
      liveSetup.controls.getByText("asha-rulebench.local-authority.v0"),
    ).toBeVisible();
    await liveSetup.controls
      .getByRole("button", { name: "Hexing Bolt Hit", exact: true })
      .click();
    await liveSetup.controls
      .getByRole("button", { name: `${livePackId}@1.0.0`, exact: true })
      .click();
    await liveSetup.controls
      .getByRole("textbox", { name: "Session" })
      .fill(liveSessionId);
    await liveSetup.controls
      .getByRole("button", { name: "Create session" })
      .click();
    await liveSetup.dialog.getByRole("button", { name: "Close" }).click();

    const statusPanel = page.getByRole("region", { name: "4. Turn status" });
    const actionsPanel = page.getByRole("region", {
      name: "6. Available actions",
    });
    const unitsPanel = page.getByRole("region", { name: "7. Active units" });
    const evidencePanel = page.getByRole("region", {
      name: "5. Evidence log",
    });
    await invokeApplicationCommand(page, "Run", "Start combat");
    await actionsPanel
      .getByRole("button", { name: "Select Hexing Bolt" })
      .click();
    await unitsPanel
      .getByRole("button", { name: "Select Raider as target" })
      .click();
    await actionsPanel
      .getByRole("button", { name: "Preflight", exact: true })
      .click();
    await actionsPanel
      .getByRole("button", { name: "Submit", exact: true })
      .click();
    await expect(unitsPanel).toContainText("Raider");
    await expect(unitsPanel).toContainText("9/18 HP");
    await evidencePanel.getByRole("tab", { name: "Combat" }).click();
    await expect(evidencePanel.getByRole("tabpanel")).toContainText(
      "Manual command",
    );
    await evidencePanel.getByRole("tab", { name: "Audit" }).click();
    await expect(evidencePanel.getByRole("tabpanel")).toContainText(
      "state changed",
    );
    await collector.milestone("live rust command rendered", {
      screenshot: true,
      layerSnapshot: {
        status: await statusPanel.innerText(),
        units: await unitsPanel.innerText(),
        evidence: await evidencePanel.getByRole("tabpanel").innerText(),
      },
    });

    await page.setViewportSize({ width: 390, height: 844 });
    await collector.milestone("live rust command mobile", {
      screenshot: true,
      layerSnapshot: {
        viewport: "390x844",
        status: await statusPanel.innerText(),
        units: await unitsPanel.innerText(),
      },
    });
    await page.setViewportSize({ width: 1280, height: 900 });
    await invokeApplicationCommand(page, "Run", "End combat");
    await invokeApplicationCommand(page, "Run", "Close session");
    await expect(statusPanel).toContainText("Not selected");

    const automaticSetup = await openLiveSetup(page);
    await automaticSetup.controls
      .getByRole("button", { name: "Hexing Bolt Hit", exact: true })
      .click();
    await automaticSetup.controls
      .getByRole("textbox", { name: "Session" })
      .fill(`live-automatic-${Date.now()}`);
    await automaticSetup.controls
      .getByRole("button", { name: "Create session" })
      .click();
    await automaticSetup.dialog.getByRole("button", { name: "Close" }).click();
    await invokeApplicationCommand(page, "Run", "Start combat");
    await invokeApplicationCommand(page, "Run", "Configure automatic run");
    const configuration = page.getByRole("dialog", {
      name: "Automatic run configuration",
    });
    await configuration.getByLabel("Max steps").fill("1");
    await configuration.getByRole("button", { name: "Close" }).click();
    await invokeApplicationCommand(page, "Run", "Run one policy step");
    await evidencePanel.getByRole("tab", { name: "Audit" }).click();
    await expect(evidencePanel.getByRole("tabpanel")).toContainText(
      "Submit Candidate",
    );
    await invokeApplicationCommand(page, "Run", "Run bounded combat");
    await expect(evidencePanel.getByRole("tabpanel")).toContainText(
      "Stopped At Max Steps",
    );
    await collector.milestone("automatic rust decisions rendered", {
      screenshot: true,
      layerSnapshot: {
        status: await statusPanel.innerText(),
        audit: await evidencePanel.getByRole("tabpanel").innerText(),
      },
    });
    await invokeApplicationCommand(page, "Run", "End combat");
    await invokeApplicationCommand(page, "Run", "Close session");
    await expect(statusPanel).toContainText("Not selected");

    await collector.milestone("panel shell rendered", {
      screenshot: true,
      layerSnapshot: {
        route: page.url(),
        panels: await page.getByLabel("Rulebench panel layout").innerText(),
      },
    });
    expect(angularTrackWarnings).toEqual([]);
  },
);

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
      summary: "Live rendered authored-content evidence.",
      tags: ["live", "authored"],
      provenance: {
        sourceKind: "authoredFile",
        sourceId: `live:${packId}`,
        authoredBy: "Rulebench live scenario",
      },
      rulesetId: "asha-rulebench.hexing-bolt.v0",
      dependencies: [],
      catalogs: {
        rulesets: [
          {
            id: "asha-rulebench.hexing-bolt.v0",
            name: "Live Compatible Ruleset",
            version: "0.0.0",
            summary: "Matches the live Hexing Bolt authority modules.",
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
            id: `entity.${packId}`,
            name: "Live Authored Entity",
            summary: "Generic definition projection evidence.",
            tags: ["live"],
            damageAdjustments: [],
          },
        ],
      },
    },
  });
}
