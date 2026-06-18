import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../i18n";
import SidebarNav from "./SidebarNav.vue";

function renderNav(path: string) {
  const router = createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: "/admin/users", component: { template: "<div />" } },
      { path: "/admin/categories", component: { template: "<div />" } },
      { path: "/projects", component: { template: "<div />" } },
    ],
  });
  return router.push(path).then(async () => {
    await router.isReady();
    render(SidebarNav, {
      props: { showCoOwnerLinks: true, showAdminLink: true, showAdminCategoryLink: true },
      global: { plugins: [router, createAppI18n("en")] },
    });
  });
}

describe("Sidebar navigation", () => {
  it("marks only the current admin item active", async () => {
    await renderNav("/admin/categories");

    expect(screen.getByText("Categories").closest("a")?.classList.contains("active")).toBe(true);
    expect(screen.getByText("User roles").closest("a")?.classList.contains("active")).toBe(false);
  });

  it("includes and activates the projects link", async () => {
    await renderNav("/projects");

    expect(screen.getByText("Projects").closest("a")?.classList.contains("active")).toBe(true);
  });
});
