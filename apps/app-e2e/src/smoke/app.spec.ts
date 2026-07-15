import { expect, test } from "@playwright/test";

test("boots the rulebench shell", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByLabel("Rulebench panel layout")).toBeVisible();
  for (const label of [
    "1. Combat grid",
    "2. Initiative",
    "3. Application menu",
    "4. Turn status",
    "5. Evidence log",
    "6. Available actions",
    "7. Active units",
  ]) {
    await expect(page.getByRole("region", { name: label })).toBeVisible();
  }
  const scenarioGrid = page.getByRole("grid", {
    name: "Scenario board for Adept hits Raider",
  });
  await expect(scenarioGrid).toBeVisible();
  await expect(
    scenarioGrid.getByRole("gridcell", {
      name: /Coordinate 1, 1; clear; occupied by Adept/,
    }),
  ).toContainText("Selected actor");
  await expect(
    scenarioGrid.getByRole("gridcell", {
      name: /Coordinate 4, 1; clear; occupied by Raider/,
    }),
  ).toContainText("Selected target");
  await expect(
    scenarioGrid.getByRole("gridcell", {
      name: /Coordinate 2, 2; cover; unoccupied/,
    }),
  ).toBeVisible();
  await page.getByRole("tab", { name: "DomainEvents" }).click();
  await expect(page.getByRole("tabpanel")).toContainText(
    "Accepted DomainEvents",
  );
  await expect(page.getByRole("tabpanel")).toContainText("DamageApplied");
  await page.getByRole("tab", { name: "DomainEvents" }).press("End");
  await expect(page.getByRole("tab", { name: "Replay" })).toBeFocused();
  await expect(page.getByRole("tab", { name: "Replay" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.getByRole("tabpanel")).toContainText("Replay review");
  await page.getByRole("tab", { name: "Replay" }).press("Home");
  await expect(page.getByRole("tab", { name: "Combat" })).toBeFocused();
  await expect(page.getByRole("tabpanel")).toContainText("Adept hits Raider");

  await expect(page.getByLabel("Rulebench panel layout")).toBeVisible();
  const menubar = page.getByRole("menubar", {
    name: "Rulebench application menu",
  });
  await menubar.getByRole("menuitem", { name: "File" }).click();
  await page
    .getByRole("menu", { name: "File" })
    .getByRole("menuitem", { name: "Content packs" })
    .click();
  const contentDialog = page.getByRole("dialog", { name: "Content packs" });
  await expect(
    contentDialog.getByRole("heading", {
      name: "Live Authored Content",
      exact: true,
    }),
  ).toBeVisible();
  await expect(
    contentDialog.getByRole("region", {
      name: "Live authored content workspace",
    }),
  ).toBeVisible();
  await contentDialog.getByRole("button", { name: "Close" }).click();

  await menubar.getByRole("menuitem", { name: "Scenario" }).click();
  await page
    .getByRole("menu", { name: "Scenario" })
    .getByRole("menuitem", { name: "Scenario cases" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Combat Session" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", {
      name: "1 · Adept hits Raider Accepted hit",
      exact: true,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Adept misses Raider/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Adept targets themself/ }),
  ).toBeVisible();
  const scenarioDialog = page.getByRole("dialog", { name: "Scenario cases" });
  const combatSession = scenarioDialog.getByRole("region", {
    name: "Combat session",
  });
  await combatSession.getByRole("button", { name: "Next" }).first().click();
  await expect(
    combatSession.getByRole("button", { name: /Adept misses Raider/ }),
  ).toHaveAttribute("aria-pressed", "true");
  await scenarioDialog
    .getByRole("button", { name: /Adept targets themself/ })
    .click();
  await expect(
    combatSession.getByRole("button", { name: /Adept targets themself/ }),
  ).toHaveAttribute("aria-pressed", "true");
  await combatSession.getByRole("button", { name: "Previous" }).first().click();
  await expect(
    combatSession.getByRole("button", { name: /Adept misses Raider/ }),
  ).toHaveAttribute("aria-pressed", "true");

  const scenarioCatalog = scenarioDialog.getByRole("region", {
    name: "Scenario catalog",
  });
  await expect(
    scenarioCatalog.getByRole("button", {
      name: "Hexing Bolt Hit Accepted hit · roll-stream:17,5",
      exact: true,
    }),
  ).toBeVisible();
  await expect(
    scenarioCatalog.getByRole("button", { name: /Hexing Bolt Miss/ }),
  ).toBeVisible();
  await expect(
    scenarioCatalog.getByRole("button", {
      name: /Hexing Bolt Self Target Rejected/,
    }),
  ).toBeVisible();

  const rejectedScenario = scenarioCatalog.getByRole("button", {
    name: /Hexing Bolt Self Target Rejected/,
  });
  await rejectedScenario.click();
  await expect(rejectedScenario).toHaveAttribute("aria-pressed", "true");
  await scenarioDialog.getByRole("button", { name: "Close" }).click();

  const evidence = page.getByRole("region", { name: "5. Evidence log" });
  await evidence.getByRole("tab", { name: "Audit" }).click();
  await expect(evidence.getByRole("tabpanel")).toContainText(
    "Rejected: Target is not hostile.",
  );
  await evidence.getByRole("tab", { name: "Rule trace" }).click();
  await expect(evidence.getByRole("tabpanel")).toContainText("Intent rejected");
  await evidence.getByRole("tab", { name: "State" }).click();
  await expect(evidence.getByRole("tabpanel")).toContainText(
    "No authority state changed",
  );
});
