import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import EventsFormPage from "./FormPage.vue";

describe("Events form", () => {
  it("shows the selected category icon next to the category dropdown", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/events/new", component: EventsFormPage }] });
    await router.push("/events/new");
    await router.isReady();

    const { container } = render(EventsFormPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByLabelText("Category")).not.toBeNull();
    expect(container.querySelector(".category-select-row .category-badge-inline")?.textContent).toContain("HEA");
    expect(getComputedStyle(container.querySelector(".category-select-row .category-badge-inline") as Element).getPropertyValue("--category-color").trim()).toBe("#d73a49");
  });
});
