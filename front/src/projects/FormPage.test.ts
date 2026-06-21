import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import type { LocaleCode } from "../common/i18n";
import ProjectsFormPage from "./FormPage.vue";

describe("Projects form", () => {
  it("renders status as an attached dropdown and text input group", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/projects/new", component: ProjectsFormPage }] });
    await router.push("/projects/new");
    await router.isReady();

    const { container } = render(ProjectsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(await screen.findByText("HEA")).not.toBeNull();
    expect(container.querySelector(".category-trigger")).not.toBeNull();
    expect(container.querySelector(".category-arrow")).not.toBeNull();
    expect(container.querySelector(".status-trigger")).not.toBeNull();
    expect(screen.getByLabelText("Project status text")).not.toBeNull();
    expect(container.querySelector(".status-input-group")).not.toBeNull();
  });

  it("has no language selector", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/projects/new", component: ProjectsFormPage }] });
    await router.push("/projects/new");
    await router.isReady();

    render(ProjectsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(screen.queryByText("Edit language")).toBeNull();
    expect(screen.queryByText("Langue à modifier")).toBeNull();
  });

  it("shows date inputs for start (always) and end (edit only)", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/projects/new", component: ProjectsFormPage }] });
    await router.push("/projects/new");
    await router.isReady();

    const { container } = render(ProjectsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    const startText = container.querySelector<HTMLInputElement>('.date-field .date-text');
    expect(startText).not.toBeNull();
    expect(startText?.getAttribute("inputmode")).toBe("numeric");
    expect(startText?.getAttribute("placeholder")).toBe("YYYY-MM-DD");
    const startNative = container.querySelector<HTMLInputElement>('.date-field .date-native');
    expect(startNative).not.toBeNull();
    expect(startNative?.getAttribute("type")).toBe("date");
    expect(screen.queryByLabelText("End")).toBeNull();
  });

  it("encodes edit project IDs in the cancel link", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/projects/:id/edit", component: ProjectsFormPage }] });
    await router.push(`/projects/${encodeURIComponent("PRJ/BIKE?1")}/edit`);
    await router.isReady();

    render(ProjectsFormPage, { global: { plugins: [router, createAppI18n("en")], provide: { selectedLocale: ref<LocaleCode>("en") } } });

    expect(await screen.findByText("Cancel")).not.toBeNull();
    expect(screen.getByText("Cancel").closest("a")?.getAttribute("href")).toContain("PRJ%2FBIKE%3F1");
  });
});
