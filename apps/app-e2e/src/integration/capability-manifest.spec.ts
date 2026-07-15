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
  await expect(activeRecoveryRow).toContainText("Not declared");
  await expect(activeRecoveryRow).toContainText(
    "rulebench-process-host.session-recovery-mode:none",
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
