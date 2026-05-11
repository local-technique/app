import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import AttachmentPreview from "./AttachmentPreview.vue";
import type { AttachmentItem } from "../attachments";
import { createAppI18n } from "../i18n";

function makeAttachment(overrides: Partial<AttachmentItem>): AttachmentItem {
  return {
    id: "att-default",
    fileName: "default.pdf",
    mimeType: "application/pdf",
    sizeBytes: 1200,
    url: "/mock-files/default.pdf",
    ...overrides,
  };
}

describe("AttachmentPreview", () => {
  it("renders a PDF object preview with fallback link", () => {
    render(AttachmentPreview, {
      global: { plugins: [createAppI18n("en")] },
      props: {
        attachment: {
          ...makeAttachment({
            id: "att-pdf",
            fileName: "invoice-2026-04.pdf",
            mimeType: "application/pdf",
            url: "/mock-files/invoice-2026-04.pdf",
          }),
        },
      },
    });

    const objectElement = screen.getByTitle("Preview invoice-2026-04.pdf");
    expect(objectElement.getAttribute("type")).toBe("application/pdf");
    expect(screen.getByRole("link", { name: "Open file" }).getAttribute("href")).toBe(
      "/mock-files/invoice-2026-04.pdf",
    );
  });

  it("renders an image preview for PNG", () => {
    render(AttachmentPreview, {
      global: { plugins: [createAppI18n("en")] },
      props: {
        attachment: {
          ...makeAttachment({
            id: "att-png",
            fileName: "elevator-check-photo.jpg",
            mimeType: "image/jpeg",
            sizeBytes: 900,
            url: "/mock-files/elevator-check-photo.jpg",
          }),
        },
      },
    });

    const image = screen.getByRole("img", { name: "Preview elevator-check-photo.jpg" });
    expect(image.getAttribute("src")).toBe("/mock-files/elevator-check-photo.jpg");
  });

  it("shows fallback link for unsupported file types", () => {
    render(AttachmentPreview, {
      global: { plugins: [createAppI18n("en")] },
      props: {
        attachment: {
          ...makeAttachment({
            id: "att-zip",
            fileName: "archive.zip",
            mimeType: "application/zip",
            sizeBytes: 2048,
            url: "/mock-files/archive.zip",
          }),
        },
      },
    });

    expect(screen.getByText("Preview unavailable for this file type.")).not.toBeNull();
    expect(screen.getByRole("link", { name: "Open file" }).getAttribute("href")).toBe(
      "/mock-files/archive.zip",
    );
  });
});
