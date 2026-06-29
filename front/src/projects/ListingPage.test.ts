import { render, screen } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import { currentUserRoles } from "../auth/session";
import ProjectsListingPage from "./ListingPage.vue";

describe("Projects listing", () => {
  it("shows projects in ongoing, to come, and finished sections", async () => {
    vi.setSystemTime(new Date("2026-06-01T12:00:00Z"));
    currentUserRoles.loaded = true;
    currentUserRoles.roles = ["ADMIN"];

    const router = createRouter({
      history: createWebHashHistory(),
      routes: [
        { path: "/projects", component: ProjectsListingPage },
        { path: "/projects/new", component: { template: "<div />" } },
        { path: "/projects/:id", component: { template: "<div />" } },
      ],
    });
    await router.push("/projects");
    await router.isReady();

    const { container } = render(ProjectsListingPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByRole("heading", { name: "Ongoing Projects" })).not.toBeNull();
    expect(container.querySelector(".category-badge-rail")?.textContent).toContain("GAR");
    expect(getComputedStyle(container.querySelector(".category-badge-rail") as Element).getPropertyValue("--category-color").trim()).toBe("#6f42c1");
    expect(screen.getByRole("heading", { name: "Projects To Come" })).not.toBeNull();
    expect(screen.getByRole("heading", { name: "Finished Projects" }).closest("section")?.getAttribute("data-status")).toBe("past");
    expect(screen.getByText("Planned - Installing fans")).not.toBeNull();
    expect(screen.getByText("Blocked - Awaiting quote")).not.toBeNull();
    expect(screen.getByText("Create project")).not.toBeNull();
    expect(screen.getByText("Garage ventilation").closest("a")?.getAttribute("href")).toContain("/projects/PRJ/ONGOING?A");
    vi.useRealTimers();
  });
});
