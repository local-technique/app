import type { AttachmentItem } from "../common/attachments";

export type EventLocalizedText = {
  en?: string;
  fr?: string;
};

export type EventItem = {
  id: string;
  categoryCode: string;
  title: EventLocalizedText;
  shortDescription: EventLocalizedText;
  longDescription: EventLocalizedText;
  warning?: EventLocalizedText;
  location?: EventLocalizedText;
  startUtc: string;
  endUtc?: string;
  notifiedAtUtc?: string;
  handlers?: string[];
  attachments: AttachmentItem[];
};

export type EventStatusSection = "current" | "toCome" | "past";
