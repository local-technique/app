import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { createAppI18n } from "../common/i18n";
import CategoriesPage from "./CategoriesPage.vue";

describe("Category admin page", () => {
  it("renders category icons in the form and list", async () => {
    render(CategoriesPage, { global: { plugins: [createAppI18n("en")] } });

    expect(await screen.findByTestId("category-icon-input-preview")).not.toBeNull();
    expect(await screen.findByTestId("category-icon-list-HEA")).not.toBeNull();
    expect(screen.getByText("flame")).not.toBeNull();
  });
});
