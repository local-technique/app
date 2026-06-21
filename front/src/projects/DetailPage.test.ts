import { render, screen } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import { currentUserRoles } from "../auth/session";
import ProjectsDetailPage from "./DetailPage.vue";

describe("Projects detail", () => {
  it("renders markdown description, empty attachments, audit, and admin actions", async () => {
    currentUserRoles.loaded = true;
    currentUserRoles.roles = ["ADMIN"];
    vi.stubGlobal("confirm", vi.fn(() => false));
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [
        { path: "/projects", component: { template: "<div />" } },
        { path: "/projects/:id", component: ProjectsDetailPage },
        { path: "/projects/:id/edit", component: { template: "<div />" } },
      ],
    });
    await router.push(`/projects/${encodeURIComponent("PRJ/BIKE?1")}?q=bike`);
    await router.isReady();

    const { container } = render(ProjectsDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

    await screen.findByText("Awaiting quote");
    expect(container.querySelector(".title-key")?.textContent).toBe("PRJ/BIKE?1");
    expect(container.querySelector(".title-text")?.textContent).toBe("Bike shelter");
    expect(container.querySelector(".title-icon-wrap")).not.toBeNull();
    expect(container.querySelector(".title-icon-wrap svg")?.getAttribute("style")).toContain("rgb(3, 102, 214)");
    expect(container.querySelector(".project-description h1")?.textContent).toBe("Bike shelter");
    expect(screen.getByText("Awaiting quote")).not.toBeNull();
    expect(screen.getByText("No attachments available.")).not.toBeNull();
    expect(screen.getByText("Edit")).not.toBeNull();
    expect(screen.getByText("Edit").closest("a")?.getAttribute("href")).toContain("PRJ%2FBIKE%3F1");
    expect(screen.getByText("Delete")).not.toBeNull();
  });
});
