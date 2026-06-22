import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import type { LocaleCode } from "../common/i18n";
import IncidentsFormPage from "./FormPage.vue";

describe("Incidents form", () => {
  it("shows the selected category in the category trigger", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/incidents/new", component: IncidentsFormPage }] });
    await router.push("/incidents/new");
    await router.isReady();

    const { container } = render(IncidentsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(await screen.findByText("HEA")).not.toBeNull();
    expect(container.querySelector(".category-trigger")).not.toBeNull();
  });

  it("has no language selector", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/incidents/new", component: IncidentsFormPage }] });
    await router.push("/incidents/new");
    await router.isReady();

    render(IncidentsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(screen.queryByText("Edit language")).toBeNull();
    expect(screen.queryByText("Langue à modifier")).toBeNull();
  });
});
