import type { AttachmentItem } from "../common/attachments";
import type { CategoryItem } from "../categories/types";

export type EventLocalizedText = {
  en?: string;
  fr?: string;
};

export type EventItem = {
  id: string;
  categoryCode: string;
  category?: Pick<CategoryItem, "id" | "key" | "icon" | "color" | "label">;
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
  lastModifiedAt?: string;
  lastModifiedBy?: { id: string; email: string } | null;
};

export type EditFieldValue = {
  fieldKey: string;
  value: string;
  exactValue?: string | null;
  fallbackLocale?: string | null;
  fallbackValue?: string | null;
};

export type EventEditData = {
  id: string;
  categoryId: string;
  startUtc: string;
  endUtc?: string;
  notifiedAtUtc?: string;
  locale: string;
  enabledLocales: string[];
  fields: EditFieldValue[];
};

export type EventSavePayload = {
  id?: string;
  categoryId: string;
  startUtc: string;
  endUtc?: string | null;
  notifiedAtUtc?: string | null;
  locale: string;
  fields: Record<string, string>;
};

export type EventStatusSection = "current" | "toCome" | "past";
