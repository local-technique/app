import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import type { LocaleCode } from "../common/i18n";
import EventsFormPage from "./FormPage.vue";

describe("Events form", () => {
  it("shows the selected category in the category trigger", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/events/new", component: EventsFormPage }] });
    await router.push("/events/new");
    await router.isReady();

    const { container } = render(EventsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(await screen.findByText("HEA")).not.toBeNull();
    expect(container.querySelector(".category-trigger")).not.toBeNull();
  });

  it("has no language selector", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/events/new", component: EventsFormPage }] });
    await router.push("/events/new");
    await router.isReady();

    render(EventsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(screen.queryByText("Edit language")).toBeNull();
    expect(screen.queryByText("Langue à modifier")).toBeNull();
  });
});
