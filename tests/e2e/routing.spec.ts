import { expect, test } from "@playwright/test";

test("supports hash deep-link to incident detail", async ({ page }) => {
  await page.goto("/#/incidents/INC-001");

  await expect(page.getByRole("heading", { name: "Heating outage on block B" })).toBeVisible();
  await expect(page.getByText("ID: INC-001")).toBeVisible();
});

test("shows not-found page for unknown route", async ({ page }) => {
  await page.goto("/#/definitely-not-a-real-route");

  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Back to events" })).toBeVisible();
});
