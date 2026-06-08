import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";
import { createAppI18n } from "../common/i18n";
import CategoriesPage from "./CategoriesPage.vue";

describe("Category admin page", () => {
  it("renders category icons in the form and list", async () => {
    const { container } = render(CategoriesPage, { global: { plugins: [createAppI18n("en")] } });

    expect(await screen.findByTestId("category-icon-input-preview")).not.toBeNull();
    expect(await screen.findByTestId("category-icon-list-HEA")).not.toBeNull();
    expect(screen.getByText("flame")).not.toBeNull();
    expect(screen.getByLabelText("Color").closest(".icon-input-row")).not.toBeNull();
    const listIcon = container.querySelector('[data-testid="category-icon-list-HEA"]') as Element;
    expect(getComputedStyle(listIcon).color).toBe("rgb(215, 58, 73)");
    expect(getComputedStyle(screen.getByText("flame")).color).toBe("rgb(215, 58, 73)");
  });

  it("edits category color with a GitHub-like picker and random refresh button", async () => {
    vi.spyOn(Math, "random").mockReturnValue(0.5);
    render(CategoriesPage, { global: { plugins: [createAppI18n("en")] } });

    const colorInput = await screen.findByLabelText<HTMLInputElement>("Color");
    expect(colorInput.value).toBe("#9aaab1");
    expect(screen.queryByText("Choose from default colors")).toBeNull();

    await fireEvent.focus(colorInput);
    const popover = screen.getByText("Choose from default colors").closest(".color-popover") as HTMLElement;
    expect(popover).not.toBeNull();
    expect(popover.classList.contains("color-popover")).toBe(true);
    expect(screen.getAllByRole("button", { name: /^Use color / })).toHaveLength(16);

    await fireEvent.click(screen.getByRole("button", { name: "Use color #d73a49" }));
    expect(colorInput.value).toBe("#d73a49");

    await fireEvent.click(screen.getByRole("button", { name: "Set random color" }));
    expect(colorInput.value).toBe("#800000");
  });
});
