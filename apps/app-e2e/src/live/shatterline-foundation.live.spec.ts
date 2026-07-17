import type { Page } from "@playwright/test";
import { readFile } from "node:fs/promises";
import type { RulebenchContentPackReferenceDto } from "@asha-rulebench/protocol";
import { createLiveRulebenchTransport } from "@asha-rulebench/transport";
import { expect, liveScenario } from "./support/live-scenario";

liveScenario(
  "Shatterline v4 scenario composition live evidence @live-artifact",
  async ({ page, collector, liveBaseUrl }) => {
    const nonce = Date.now().toString();
    const packId = `pack.live.shatterline.foundation-${nonce}`;
    const sessionId = `live-shatterline-foundation-${nonce}`;
    const apiBaseUrl = new URL("/api/rulebench/v1", liveBaseUrl).toString();
    const transport = createLiveRulebenchTransport({ apiBaseUrl });
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
        `fixture:live-shatterline-foundation-v4-${nonce}`,
      );
    let reference: RulebenchContentPackReferenceDto | undefined;
    let sessionExists = false;

    collector.addNonClaim(
      "This scenario proves the two foundation scenario choices and one manual session composition receipt through the current live host; it does not claim the later Shatterline pressure scenarios or complete campaign behavior.",
    );

    try {
      await page.goto(liveBaseUrl);
      const connected = await transport.connect();
      expect(connected.ok).toBe(true);
      const imported = await transport.importContent(payload, "reject");
      expect(imported.ok).toBe(true);
      if (!imported.ok || imported.value.outcome === null) return;
      reference = imported.value.outcome.review.pack.reference;
      const activated = await transport.activateContent(reference);
      expect(activated.ok).toBe(true);
      const setup = await openLiveSetup(page);
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
      await expect(
        setup.getByRole("button", { name: `${packId}@4.0.0` }),
      ).toHaveAttribute("aria-pressed", "true");
      await expect(
        setup.getByRole("button", { name: "Create session" }),
      ).toBeEnabled();

      await page.setViewportSize({ width: 1280, height: 900 });
      await collector.milestone("Shatterline foundation setup desktop", {
        screenshot: true,
        layerSnapshot: { setup: await setup.innerText() },
      });
      await page.setViewportSize({ width: 640, height: 900 });
      await collector.milestone("Shatterline foundation setup narrow", {
        screenshot: true,
        layerSnapshot: { viewport: "640x900", setup: await setup.innerText() },
      });

      await setup
        .getByRole("textbox", { name: "Session", exact: true })
        .fill(sessionId);
      await setup.getByRole("button", { name: "Create session" }).click();
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
        "action.binding-spark → foundation-binding-spark",
      );
      await collector.milestone("Shatterline composition receipt narrow", {
        screenshot: true,
        layerSnapshot: { composition: await composition.innerText() },
      });
      await page.setViewportSize({ width: 1280, height: 900 });
      await collector.milestone("Shatterline composition receipt desktop", {
        screenshot: true,
        layerSnapshot: { composition: await composition.innerText() },
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
  },
);

async function openLiveSetup(page: Page) {
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
  return dialog.getByRole("region", {
    name: "Live combat setup controls",
  });
}
