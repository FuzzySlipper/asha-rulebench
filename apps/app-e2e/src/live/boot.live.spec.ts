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
    const angularTrackWarnings: string[] = [];
    page.on("console", (message) => {
      if (message.text().includes("NG0955")) {
        angularTrackWarnings.push(message.text());
      }
    });
    collector.addNonClaim(
      "This scenario proves one Rulebench fixture can be controlled through the live Rust host; it does not prove arbitrary rulesets, durable sessions, broad accessibility coverage, or performance.",
    );

    await page.goto(liveBaseUrl);
    await expect(page.getByLabel("Rulebench panel layout")).toBeVisible();

    await invokeApplicationCommand(page, "File", "Content packs");
    const contentDialog = page.getByRole("dialog", { name: "Content packs" });
    await expect(
      contentDialog.getByRole("heading", {
        name: "Content Packs",
        exact: true,
      }),
    ).toBeVisible();
    await contentDialog
      .getByRole("button", { name: /pack.error@1.0.0/ })
      .click();
    await expect(
      contentDialog.getByLabel("Selected content pack review"),
    ).toContainText("missingContentPackDependency");
    await collector.milestone("content diagnostics rendered", {
      screenshot: true,
      layerSnapshot: {
        selectedPack: await contentDialog
          .getByLabel("Selected content pack review")
          .innerText(),
        validation: await contentDialog
          .getByLabel("Content validation review")
          .innerText(),
      },
    });
    await contentDialog.getByRole("button", { name: "Close" }).click();

    const liveSetup = await openLiveSetup(page);
    await expect(
      liveSetup.controls.getByText("asha-rulebench.local-authority.v0"),
    ).toBeVisible();
    await liveSetup.controls
      .getByRole("button", { name: "Hexing Bolt Hit", exact: true })
      .click();
    await liveSetup.controls.getByLabel("Session").fill(liveSessionId);
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
      .getByLabel("Session")
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
