import { describe, expect, it } from "vitest";
import { resolveLocalized } from "./localeContent";

describe("resolveLocalized", () => {
  it("applies fallback active -> en -> fr", () => {
    expect(resolveLocalized({ fr: "Bonjour" }, "en")).toBe("Bonjour");
    expect(resolveLocalized({ en: "Hello", fr: "Bonjour" }, "fr")).toBe("Bonjour");
    expect(resolveLocalized({ en: "Hello" }, "en")).toBe("Hello");
  });

  it("returns empty string when all locales missing", () => {
    expect(resolveLocalized({}, "fr")).toBe("");
  });
});
