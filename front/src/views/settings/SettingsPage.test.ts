import { fireEvent, render, screen, waitFor } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";
import { createAppI18n } from "../../common/i18n";
import SettingsPage from "./SettingsPage.vue";

vi.mock("./api", () => ({
  getToken: vi.fn(),
  createToken: vi.fn(),
  revokeToken: vi.fn(),
}));

import * as api from "./api";

function mockMe(email: string) {
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: true,
    json: async () => ({ email }),
  });
}

describe("Settings page", () => {
  it("shows email and generate button when no token exists", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);

    render(SettingsPage, { global: { plugins: [createAppI18n("en")] } });

    expect(await screen.findByText("test@example.com")).not.toBeNull();
    expect(await screen.findByText("Generate token")).not.toBeNull();
  });

  it("shows token details when token exists", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue({
      id: "abc-123",
      token_prefix: "lc_abc",
      created_at: "2026-06-12T10:00:00Z",
      last_used_at: "2026-06-12T11:00:00Z",
    });

    render(SettingsPage, { global: { plugins: [createAppI18n("en")] } });

    expect(await screen.findByText("lc_abc...")).not.toBeNull();
    expect(screen.getByText("Renew token")).not.toBeNull();
    expect(screen.getByText("Revoke token")).not.toBeNull();
  });

  it("shows full token after generation", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);
    vi.mocked(api.createToken).mockResolvedValue({
      id: "new-id",
      token_prefix: "lc_xyz",
      token_full: "lc_xyzabc123def456ghi789jkl012mno345pqr678stu901vwx",
      created_at: "2026-06-12T12:00:00Z",
    });

    render(SettingsPage, { global: { plugins: [createAppI18n("en")] } });

    const generateBtn = await screen.findByText("Generate token");
    await fireEvent.click(generateBtn);

    await waitFor(() => {
      expect(screen.getByText("This token won't be shown again.")).not.toBeNull();
    });
    expect(screen.getByText(/lc_xyzabc123def456ghi789jkl012mno345pqr678stu901vwx/)).not.toBeNull();
  });
});
