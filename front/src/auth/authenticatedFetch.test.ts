import { describe, expect, it, vi, beforeEach } from "vitest";
import * as session from "./session";

vi.mock("./session", () => ({
  getAccessToken: vi.fn(),
  isAccessTokenUsable: vi.fn(),
  ensureAuthenticated: vi.fn(),
}));

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("authenticatedFetch", () => {
  it("calls fetch with auth header when token is usable", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken).mockReturnValue("valid-token");

    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("ok", { status: 200 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");
    const response = await authenticatedFetch("https://api.example.com/data");

    expect(fetchSpy).toHaveBeenCalledWith("https://api.example.com/data", {
      headers: new Headers({ Authorization: "Bearer valid-token" }),
    });
    expect(response.status).toBe(200);
    fetchSpy.mockRestore();
  });

  it("proactively refreshes token before call if not usable", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(false);
    vi.mocked(session.ensureAuthenticated).mockResolvedValue(true);
    vi.mocked(session.getAccessToken).mockReturnValue("refreshed-token");

    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("ok", { status: 200 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");
    await authenticatedFetch("https://api.example.com/data");

    expect(session.ensureAuthenticated).toHaveBeenCalledTimes(1);
    expect(fetchSpy).toHaveBeenCalledWith("https://api.example.com/data", {
      headers: new Headers({ Authorization: "Bearer refreshed-token" }),
    });
    fetchSpy.mockRestore();
  });

  it("redirects to login on 401 response", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken).mockReturnValue("token");

    const originalHash = window.location.hash;
    window.location.hash = "#/events";

    vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("unauthorized", { status: 401 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");

    await expect(authenticatedFetch("https://api.example.com/data")).rejects.toThrow("session expired");
    expect(window.location.hash).toBe("#/login?redirect=%2Fevents");

    window.location.hash = originalHash;
  });

  it("redirects to /login when proactive refresh fails", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(false);
    vi.mocked(session.ensureAuthenticated).mockResolvedValue(false);

    const originalHash = window.location.hash;
    window.location.hash = "#/events";

    const { authenticatedFetch } = await import("./authenticatedFetch");

    await expect(authenticatedFetch("https://api.example.com/data")).rejects.toThrow("session expired");
    expect(window.location.hash).toBe("#/login?redirect=%2Fevents");

    window.location.hash = originalHash;
  });
});
