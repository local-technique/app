import { describe, expect, it, vi, beforeEach } from "vitest";
import * as session from "./session";

vi.mock("./session", () => ({
  getAccessToken: vi.fn(),
  isAccessTokenUsable: vi.fn(),
  ensureAuthenticated: vi.fn(),
  refreshAccessToken: vi.fn(),
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

  it("retries once on 401 if refresh succeeds", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken)
      .mockReturnValueOnce("expired-token")
      .mockReturnValueOnce("new-token");
    vi.mocked(session.refreshAccessToken).mockResolvedValue(true);

    let callCount = 0;
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockImplementation(async () => {
      callCount++;
      return new Response("ok", { status: callCount === 1 ? 401 : 200 });
    });

    const { authenticatedFetch } = await import("./authenticatedFetch");
    const response = await authenticatedFetch("https://api.example.com/data");

    expect(session.refreshAccessToken).toHaveBeenCalledTimes(1);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
    expect(response.status).toBe(200);
    fetchSpy.mockRestore();
  });

  it("redirects to /login when refresh fails on 401", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken).mockReturnValue("expired-token");
    vi.mocked(session.refreshAccessToken).mockResolvedValue(false);

    const originalHash = window.location.hash;
    window.location.hash = "#/events/some-page";

    vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("unauthorized", { status: 401 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");

    await expect(authenticatedFetch("https://api.example.com/data")).rejects.toThrow("session expired");
    expect(window.location.hash).toBe("#/login?redirect=%2Fevents%2Fsome-page");

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

  it("preserves caller's Content-Type header", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken).mockReturnValue("valid-token");

    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("ok", { status: 200 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");
    await authenticatedFetch("https://api.example.com/data", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ foo: "bar" }),
    });

    const callHeaders = fetchSpy.mock.calls[0][1]?.headers as Headers;
    expect(callHeaders.get("Authorization")).toBe("Bearer valid-token");
    expect(callHeaders.get("Content-Type")).toBe("application/json");
    expect(fetchSpy.mock.calls[0][1]?.body).toBe(JSON.stringify({ foo: "bar" }));
    fetchSpy.mockRestore();
  });

  it("does not retry more than once", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken)
      .mockReturnValueOnce("token1")
      .mockReturnValueOnce("token2");
    vi.mocked(session.refreshAccessToken).mockResolvedValue(true);

    vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("unauthorized", { status: 401 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");

    const response = await authenticatedFetch("https://api.example.com/data");
    expect(response.status).toBe(401);
  });
});
