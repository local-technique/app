import { render } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../i18n";
import MobileBottomNav from "./MobileBottomNav.vue";

describe("Mobile bottom nav", () => {
  it("keeps all visible items in the five grid slots", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/events", component: { template: "<div />" } }],
    });
    await router.push("/events");
    await router.isReady();

    const { container } = render(MobileBottomNav, {
      props: { showCoOwnerLinks: true, showAdminLink: true },
      global: { plugins: [router, createAppI18n("en")] },
    });

    expect(container.querySelectorAll(".nav-item")).toHaveLength(5);
  });

  it("marks only the current admin item active", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/admin/categories", component: { template: "<div />" } }],
    });
    await router.push("/admin/categories");
    await router.isReady();

    const { container } = render(MobileBottomNav, {
      props: { showCoOwnerLinks: true, showAdminLink: true },
      global: { plugins: [router, createAppI18n("en")] },
    });

    expect(container.querySelector("a[aria-label='Categories']")?.classList.contains("active")).toBe(true);
    expect(container.querySelector("a[aria-label='User roles']")?.classList.contains("active")).toBe(false);
  });
});
