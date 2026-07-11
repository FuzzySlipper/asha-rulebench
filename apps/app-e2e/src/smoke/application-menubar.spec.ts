import { expect, test } from "@playwright/test";

test("operates the application menubar entirely by keyboard", async ({
  page,
}) => {
  await page.goto("/");

  const menubar = page.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  const scenario = menubar.getByRole("menuitem", { name: "Scenario" });
  await scenario.focus();
  await expect(scenario).toBeFocused();

  await page.keyboard.press("ArrowRight");
  const run = menubar.getByRole("menuitem", { name: "Run" });
  await expect(run).toBeFocused();

  await page.keyboard.press("End");
  const view = menubar.getByRole("menuitem", { name: "View" });
  await expect(view).toBeFocused();

  await page.keyboard.press("Home");
  const file = menubar.getByRole("menuitem", { name: "File" });
  await expect(file).toBeFocused();
  await page.keyboard.press("ArrowRight");
  await expect(scenario).toBeFocused();
  await page.keyboard.press("ArrowDown");

  const scenarioMenu = page.getByRole("menu", { name: "Scenario" });
  await expect(scenarioMenu).toBeVisible();
  await expect(
    scenarioMenu.getByRole("menuitem", { name: "Scenario cases" }),
  ).toBeFocused();

  await page.keyboard.press("i");
  const initiative = scenarioMenu.getByRole("menuitem", { name: "Initiative" });
  await expect(initiative).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("region", { name: "2. Initiative" }),
  ).toBeFocused();
  await expect(
    page
      .getByRole("region", { name: "3. Application menu" })
      .getByRole("status"),
  ).toHaveText("Focused Initiative");

  await run.focus();
  await page.keyboard.press("ArrowDown");
  const runMenu = page.getByRole("menu", { name: "Run" });
  await expect(
    runMenu.getByRole("menuitem", { name: "Current actor" }),
  ).toHaveAttribute("aria-disabled", "true");
  await page.keyboard.press("End");
  await expect(
    runMenu.getByRole("menuitem", { name: "Available actions" }),
  ).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect(
    runMenu.getByRole("menuitem", { name: "Configure automatic run" }),
  ).toBeFocused();

  await page.keyboard.press("Escape");
  await expect(runMenu).toHaveCount(0);
  await expect(run).toBeFocused();

  await file.click();
  await page
    .getByRole("menu", { name: "File" })
    .getByRole("menuitem", { name: "Content packs" })
    .click();
  const contentDialog = page.getByRole("dialog", { name: "Content packs" });
  await expect(
    contentDialog.getByRole("button", { name: "Close" }),
  ).toBeFocused();
  await contentDialog.getByRole("button", { name: "Close" }).click();
  await expect(file).toBeFocused();
});

test("keeps application menus accessible at mobile width", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(
    page.getByRole("grid", { name: "Scenario board for Adept hits Raider" }),
  ).toBeVisible();

  const replay = page
    .getByRole("menubar", { name: "Rulebench application menu" })
    .getByRole("menuitem", { name: "Replay" });
  await replay.focus();
  await page.keyboard.press("Enter");

  const replayMenu = page.getByRole("menu", { name: "Replay" });
  await expect(replayMenu).toBeInViewport();
  await expect(
    replayMenu.getByRole("menuitem", { name: "Replay archive" }),
  ).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(replayMenu).toHaveCount(0);

  await replay.focus();
  await page.keyboard.press("Enter");
  await page
    .getByRole("region", { name: "1. Combat grid" })
    .click({ position: { x: 12, y: 48 } });
  await expect(page.getByRole("menu", { name: "Replay" })).toHaveCount(0);

  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: window.innerWidth,
  }));
  expect(dimensions.body).toBe(dimensions.viewport);
});

test("preserves panel and dialog navigation under accessibility media", async ({
  page,
}) => {
  await page.emulateMedia({
    colorScheme: "dark",
    forcedColors: "active",
    reducedMotion: "reduce",
  });
  await page.setViewportSize({ width: 640, height: 450 });
  await page.goto("/");
  await page.evaluate(() => {
    document.documentElement.style.zoom = "200%";
  });

  const layout = page.getByLabel("Rulebench panel layout");
  await expect(layout).toBeVisible();
  for (const name of [
    "1. Combat grid",
    "2. Initiative",
    "3. Application menu",
    "4. Turn status",
    "5. Evidence log",
    "6. Available actions",
    "7. Active units",
  ]) {
    await expect(layout.getByRole("region", { name })).toHaveCount(1);
  }

  const menubar = layout.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  await menubar.getByRole("menuitem", { name: "Replay" }).focus();
  await page.keyboard.press("Enter");
  await page.keyboard.press("Enter");
  const replayDialog = page.getByRole("dialog", { name: "Replay archive" });
  await expect(replayDialog).toBeVisible();
  await expect(
    replayDialog.getByRole("region", { name: "Replay archive controls" }),
  ).toHaveCount(1);
  await expect(
    replayDialog.getByRole("button", { name: "Close" }),
  ).toBeFocused();

  const dimensions = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));
  expect(dimensions.body).toBeLessThanOrEqual(dimensions.viewport);

  await page.keyboard.press("Escape");
  await expect(replayDialog).toHaveCount(0);
  await expect(menubar.getByRole("menuitem", { name: "Replay" })).toBeFocused();
});
