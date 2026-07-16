import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

async function openCapabilityManifest(page: Page) {
  const menubar = page.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  await menubar.getByRole("menuitem", { name: "View" }).click();
  await page
    .getByRole("menu", { name: "View" })
    .getByRole("menuitem", { name: "Runtime capabilities" })
    .click();
  return page.getByRole("dialog", { name: "Runtime capabilities" });
}

async function openAuthorityViewer(page: Page) {
  const menubar = page.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  await menubar.getByRole("menuitem", { name: "Scenario" }).click();
  await page
    .getByRole("menu", { name: "Scenario" })
    .getByRole("menuitem", { name: "Scenario cases" })
    .click();
  return page.getByRole("dialog", { name: "Live authority viewer" });
}

test("renders the current Rust host capability manifest", async ({ page }) => {
  await page.goto("/");

  const dialog = await openCapabilityManifest(page);
  await expect(dialog).toBeVisible();
  await expect(
    dialog.getByText("rulebench-process-host · filesystem"),
  ).toBeVisible();
  await expect(
    dialog.getByText("pipeline 2 · effects 1", { exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByText("2 providers · 2 rulesets · 4 packages · 11 scenarios"),
  ).toBeVisible();
  await expect(
    dialog.getByText("Live authority readback; no checked-artifact fallback"),
  ).toBeVisible();
  await expect(
    dialog.getByText(/provider\.asha-rulebench\.turn-control@1/),
  ).toBeVisible();
  await expect(
    dialog.getByText(/asha-rulebench\.turn-control\.v0@0\.1\.0/),
  ).toBeVisible();

  const supportMatrix = dialog.getByRole("table", {
    name: "Executable support matrix",
  });
  await expect(supportMatrix).toBeVisible();
  await expect(
    supportMatrix.getByRole("cell", { name: "operation.openReactionWindow" }),
  ).toBeVisible();
  const activeRecoveryRow = supportMatrix.getByRole("row", {
    name: /session\.active-recovery/,
  });
  await expect(activeRecoveryRow).toContainText(
    "Durable and regression covered",
  );
  await expect(activeRecoveryRow).toContainText(
    "rulebench-process-host.session-recovery-mode:replayVerifiedCheckpoints",
  );
  const viewerReadbackRow = supportMatrix.getByRole("row", {
    name: /viewer\.authority-readback/,
  });
  await expect(viewerReadbackRow).toContainText(
    "Regression covered, not restart durable",
  );
});

test("keeps the capability evidence inspectable at mobile width", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");

  const dialog = await openCapabilityManifest(page);
  const matrix = dialog.getByRole("region", {
    name: "Scrollable capability matrix",
  });
  await matrix.scrollIntoViewIfNeeded();
  await expect(matrix).toBeInViewport();
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
});

test("renders owner-registered authority viewer evidence at desktop width", async ({
  page,
}, testInfo) => {
  await page.goto("/");

  const dialog = await openAuthorityViewer(page);
  await expect(
    dialog.getByRole("heading", { name: "Authority Scenario Evidence" }),
  ).toBeVisible();
  const providerScenario = dialog.getByRole("button", {
    name: /Storm Pulse Multiple-target Conformance/,
  });
  await expect(providerScenario).toBeVisible();
  await providerScenario.click();
  await expect(providerScenario).toHaveAttribute("aria-pressed", "true");
  await page.screenshot({
    path: testInfo.outputPath("authority-viewer-desktop.png"),
    fullPage: true,
  });

  await dialog.getByRole("button", { name: "Close" }).click();
  await expect(
    page.getByRole("grid", {
      name: "Scenario board for Storm Pulse Multiple-target Conformance",
    }),
  ).toBeVisible();
  await page.getByRole("tab", { name: "DomainEvents" }).click();
  await expect(page.getByRole("tabpanel")).toContainText("Resource Changed");
});

test("keeps live authority viewer evidence inspectable at mobile width", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");

  const dialog = await openAuthorityViewer(page);
  await expect(
    dialog.getByRole("button", {
      name: /Storm Pulse Multiple-target Conformance/,
    }),
  ).toBeVisible();
  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);
  await page.screenshot({
    path: testInfo.outputPath("authority-viewer-mobile.png"),
    fullPage: true,
  });
});

test("shows a classified authority viewer failure with explicit retry", async ({
  page,
}, testInfo) => {
  await page.route("**/api/rulebench/v1/viewer/scenarios", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({
        kind: "network",
        code: "viewerUnavailable",
        message: "Authority viewer route unavailable.",
        retryable: true,
      }),
    });
  });
  await page.goto("/");

  const dialog = await openAuthorityViewer(page);
  const catalog = dialog.getByRole("region", { name: "Scenario catalog" });
  await expect(
    catalog.getByText("Authority viewer route unavailable."),
  ).toBeVisible();
  await expect(
    catalog.getByRole("button", { name: "Retry scenarios" }),
  ).toBeVisible();
  await page.screenshot({
    path: testInfo.outputPath("authority-viewer-error.png"),
    fullPage: true,
  });
});
