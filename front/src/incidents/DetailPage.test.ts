import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createRouter, createWebHashHistory } from "vue-router";
import { createAppI18n } from "../common/i18n";
import IncidentDetailPage from "./DetailPage.vue";

describe("Incident detail", () => {
  it("shows incident id and dedicated timeline block", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/incidents/:id", component: IncidentDetailPage }],
    });
    await router.push("/incidents/INC-001?q=generator");
    await router.isReady();

    const { container } = render(IncidentDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

    await screen.findByText("Heating outage on block B");
    expect(container.querySelector(".title-icon-wrap")).not.toBeNull();
    expect(container.querySelector(".title-icon-wrap svg")?.getAttribute("style")).toContain("rgb(215, 58, 73)");
    expect(container.querySelector(".title-key")?.textContent).toBe("INC-001");
    expect(screen.getByRole("heading", { name: "Incident timeline" })).not.toBeNull();
    expect(screen.getByText("Issue detected by monitoring system")).not.toBeNull();
  });

  it("renders pending timeline entries before dated entries", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/incidents/:id", component: IncidentDetailPage }],
    });
    await router.push("/incidents/INC-001");
    await router.isReady();

    render(IncidentDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

    const pendingBadge = await screen.findByText("Pending");
    const pendingCard = pendingBadge.closest(".timeline-row");
    const datedTitle = await screen.findByText("Issue detected by monitoring system");
    const datedCard = datedTitle.closest(".timeline-row");

    expect(pendingCard).not.toBeNull();
    expect(datedCard).not.toBeNull();
    expect(pendingCard?.compareDocumentPosition(datedCard as Node)).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
    expect(datedCard?.querySelector(".timeline-entry-icon")).not.toBeNull();
    expect(screen.getByText("Temperature fell below threshold in two risers.").classList.contains("timeline-entry-details")).toBe(true);
  });

  it("renders not found state", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [{ path: "/incidents/:id", component: IncidentDetailPage }],
    });
    await router.push("/incidents/UNKNOWN");
    await router.isReady();

    render(IncidentDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByRole("heading", { name: "Incident not found" })).not.toBeNull();
  });
});
