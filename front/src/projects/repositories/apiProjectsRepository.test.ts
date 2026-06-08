import { describe, expect, it, vi } from "vitest";
import { ApiProjectsRepository } from "./apiProjectsRepository";

describe("ApiProjectsRepository", () => {
  it("uses mock data in test mode instead of calling the backend", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");

    const result = await new ApiProjectsRepository().list("en", "bike");

    expect(fetchSpy).not.toHaveBeenCalled();
    expect(result.length).toBeGreaterThan(0);
    expect(result.some((project) => project.id === "PRJ/BIKE?1")).toBe(true);
    fetchSpy.mockRestore();
  });
});
