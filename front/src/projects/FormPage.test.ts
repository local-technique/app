import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import ProjectsFormPage from "./FormPage.vue";

describe("Projects form", () => {
  it("renders project status as an attached dropdown and text input group", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/projects/new", component: ProjectsFormPage }] });
    await router.push("/projects/new");
    await router.isReady();

    const { container } = render(ProjectsFormPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(screen.getByText("Project status")).not.toBeNull();
    expect(screen.queryByText("Project status text")).toBeNull();
    expect(await screen.findByLabelText("Category")).not.toBeNull();
    expect(container.querySelector(".category-select-row .category-badge-inline")?.textContent).toContain("HEA");
    expect(getComputedStyle(container.querySelector(".category-select-row .category-badge-inline") as Element).getPropertyValue("--category-color").trim()).toBe("#d73a49");
    expect(screen.getByLabelText("Project status type")).not.toBeNull();
    expect(screen.getByLabelText("Project status text")).not.toBeNull();
    expect(container.querySelector(".status-input-group")).not.toBeNull();
  });

  it("encodes edit project IDs in the cancel link", async () => {
    const router = createRouter({ history: createWebHashHistory(), routes: [{ path: "/projects/:id/edit", component: ProjectsFormPage }] });
    await router.push(`/projects/${encodeURIComponent("PRJ/BIKE?1")}/edit`);
    await router.isReady();

    render(ProjectsFormPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByText("Cancel")).not.toBeNull();
    expect(screen.getByText("Cancel").closest("a")?.getAttribute("href")).toContain("PRJ%2FBIKE%3F1");
  });
});
