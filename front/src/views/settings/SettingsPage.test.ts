import { fireEvent, render, screen, waitFor } from "@testing-library/vue";
import { ref, type Ref } from "vue";
import { describe, expect, it, vi } from "vitest";
import { createAppI18n } from "../../common/i18n";
import type { LocaleCode } from "../../common/i18n";
import type { ThemeMode } from "../../common/theme";
import SettingsPage from "./SettingsPage.vue";

vi.mock("./api", () => ({
  getToken: vi.fn(),
  createToken: vi.fn(),
  revokeToken: vi.fn(),
}));

import * as api from "./api";

function mockMe(email: string, roles?: string[]) {
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: true,
    json: async () => ({ email, roles: roles ?? ["ADMIN"] }),
  });
}

function renderPage() {
  const selectedLocale = ref<LocaleCode>("en");
  const selectedTheme = ref<ThemeMode>("system");
  return render(SettingsPage, {
    global: {
      plugins: [createAppI18n("en")],
      provide: {
        selectedLocale,
        selectedTheme,
      },
    },
  });
}

describe("Settings page", () => {
  it("shows email and generate button when no token exists", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);

    renderPage();

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

    renderPage();

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

    renderPage();

    const generateBtn = await screen.findByText("Generate token");
    await fireEvent.click(generateBtn);

    await waitFor(() => {
      expect(screen.getByText("This token won't be shown again.")).not.toBeNull();
    });
    expect(screen.getByText(/lc_xyzabc123def456ghi789jkl012mno345pqr678stu901vwx/)).not.toBeNull();
  });

  it("renders locale and theme controls", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);

    renderPage();

    expect(await screen.findByText("Language")).not.toBeNull();
    expect(await screen.findByText("Theme")).not.toBeNull();
  });

  it("shows sign out button and calls logout on click", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);

    renderPage();

    const btn = await screen.findByText("Sign out");
    expect(btn).not.toBeNull();

    await fireEvent.click(btn);

    expect(globalThis.fetch).toHaveBeenCalledWith(
      expect.stringContaining("/auth/logout"),
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("updates injected locale and theme refs on selection change", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);

    const locale = ref<LocaleCode>("en");
    const theme = ref<ThemeMode>("system");

    render(SettingsPage, {
      global: {
        plugins: [createAppI18n("en")],
        provide: { selectedLocale: locale, selectedTheme: theme },
      },
    });

    await screen.findByText("test@example.com");

    const themeSelect = screen.getByDisplayValue("System");
    await fireEvent.update(themeSelect, "dark");

    const langSelect = screen.getByDisplayValue("EN");
    await fireEvent.update(langSelect, "fr");

    expect(theme.value).toBe("dark");
    expect(locale.value).toBe("fr");
  });
});
