import { describe, expect, it } from "vitest";
import { classifyEventStatus } from "./date";

describe("classifyEventStatus", () => {
  it("classifies ranged events as toCome/current/past", () => {
    const startUtc = "2030-01-01T10:00:00Z";
    const endUtc = "2030-01-01T12:00:00Z";

    expect(classifyEventStatus({ startUtc, endUtc }, new Date("2030-01-01T09:59:59Z"))).toBe("toCome");
    expect(classifyEventStatus({ startUtc, endUtc }, new Date("2030-01-01T10:00:00Z"))).toBe("current");
    expect(classifyEventStatus({ startUtc, endUtc }, new Date("2030-01-01T12:00:00Z"))).toBe("current");
    expect(classifyEventStatus({ startUtc, endUtc }, new Date("2030-01-01T12:00:01Z"))).toBe("past");
  });

  it("classifies point-in-time events as toCome before start and past at or after start", () => {
    const startUtc = "2030-05-01T10:00:00Z";

    expect(classifyEventStatus({ startUtc }, new Date("2030-05-01T09:59:59Z"))).toBe("toCome");
    expect(classifyEventStatus({ startUtc }, new Date("2030-05-01T10:00:00Z"))).toBe("past");
    expect(classifyEventStatus({ startUtc }, new Date("2030-05-01T10:00:01Z"))).toBe("past");
  });
});
