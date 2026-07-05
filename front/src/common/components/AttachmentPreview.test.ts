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
    downloadUrl: "/mock-files/default.pdf",
    ...overrides,
  };
}

describe("AttachmentPreview", () => {
  it("renders a PDF placeholder with filename, size, and clickable link", () => {
    render(AttachmentPreview, {
      global: { plugins: [createAppI18n("en")] },
      props: {
        attachment: makeAttachment({
          id: "att-pdf",
          fileName: "invoice-2026-04.pdf",
          mimeType: "application/pdf",
          url: "/mock-files/invoice-2026-04.pdf",
        }),
      },
    });

    expect(screen.getByText("PDF")).not.toBeNull();
    expect(screen.getByText("invoice-2026-04.pdf")).not.toBeNull();
    expect(screen.getByText("1.2 KB")).not.toBeNull();
    const link = document.querySelector(".preview-link-wrap") as HTMLAnchorElement;
    expect(link?.getAttribute("href")).toBe("/mock-files/invoice-2026-04.pdf");
  });

  it("renders an image preview with clickable link", () => {
    render(AttachmentPreview, {
      global: { plugins: [createAppI18n("en")] },
      props: {
        attachment: makeAttachment({
          id: "att-png",
          fileName: "elevator-check-photo.jpg",
          mimeType: "image/jpeg",
          sizeBytes: 900,
          url: "/mock-files/elevator-check-photo.jpg",
        }),
      },
    });

    const image = screen.getByRole("img");
    expect(image.getAttribute("src")).toBe("/mock-files/elevator-check-photo.jpg");
    const link = document.querySelector(".preview-link-wrap") as HTMLAnchorElement;
    expect(link?.getAttribute("href")).toBe("/mock-files/elevator-check-photo.jpg");
  });

  it("shows file icon placeholder for unsupported types", () => {
    render(AttachmentPreview, {
      global: { plugins: [createAppI18n("en")] },
      props: {
        attachment: makeAttachment({
          id: "att-zip",
          fileName: "archive.zip",
          mimeType: "application/zip",
          sizeBytes: 2048,
          url: "/mock-files/archive.zip",
        }),
      },
    });

    expect(screen.getByText("archive.zip")).not.toBeNull();
    expect(screen.getByText("2.0 KB")).not.toBeNull();
    const link = document.querySelector(".preview-link-wrap") as HTMLAnchorElement;
    expect(link?.getAttribute("href")).toBe("/mock-files/archive.zip");
  });
});
