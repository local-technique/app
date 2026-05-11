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

    render(IncidentDetailPage, { global: { plugins: [router, createAppI18n("en")] } });

    expect(await screen.findByRole("heading", { name: "Heating outage on block B" })).not.toBeNull();
    expect(screen.getByText("ID: INC-001")).not.toBeNull();
    expect(screen.getByRole("heading", { name: "Incident timeline" })).not.toBeNull();
    expect(screen.getByText("Issue detected by monitoring system")).not.toBeNull();
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
