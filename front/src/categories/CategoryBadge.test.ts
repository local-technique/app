import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import CategoryBadge from "./CategoryBadge.vue";

describe("CategoryBadge", () => {
  it("renders a rail badge with the category code", () => {
    const { container } = render(CategoryBadge, { props: { categoryKey: "HEA", icon: "flame", label: "Heating", variant: "rail" } });

    expect(screen.getByLabelText("HEA - Heating")).not.toBeNull();
    expect(container.querySelector(".category-badge-rail")?.textContent).toContain("HEA");
  });

  it("uses a compact gap between the inline icon and code", () => {
    const { container } = render(CategoryBadge, { props: { categoryKey: "HEA", icon: "flame", label: "Heating" } });
    const badge = container.querySelector(".category-badge-inline");

    expect(badge).not.toBeNull();
    expect(getComputedStyle(badge as Element).gap).toBe("0.21rem");
  });

  it("packs the rail icon and code together within the card height", () => {
    const { container } = render(CategoryBadge, { props: { categoryKey: "HEA", icon: "flame", label: "Heating", variant: "rail" } });
    const badge = container.querySelector(".category-badge-rail");

    expect(badge).not.toBeNull();
    expect(getComputedStyle(badge as Element).alignContent).toBe("center");
  });

  it("applies the category color to the badge", () => {
    const { container } = render(CategoryBadge, { props: { categoryKey: "HEA", icon: "flame", label: "Heating", color: "#d73a49", variant: "rail" } });
    const badge = container.querySelector(".category-badge-rail");

    expect(badge).not.toBeNull();
    expect(getComputedStyle(badge as Element).getPropertyValue("--category-color").trim()).toBe("#d73a49");
  });
});
