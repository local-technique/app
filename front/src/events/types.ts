import type { AttachmentItem } from "../common/attachments";
import type { CategoryItem } from "../categories/types";

export type EventLocalizedText = {
  en?: string;
  fr?: string;
};

export type EventTimelineEntry = {
  id: string;
  atUtc: string | null;
  title: EventLocalizedText;
  details?: EventLocalizedText;
};

export type EventItem = {
  id: string;
  categoryCode: string;
  category?: Pick<CategoryItem, "id" | "key" | "icon" | "color" | "label">;
  title: EventLocalizedText;
  description: EventLocalizedText;
  warning?: EventLocalizedText;
  location?: EventLocalizedText;
  startUtc: string;
  endUtc?: string;
  statusType: EventStoredStatus;
  statusText: EventLocalizedText;
  timeline: EventTimelineEntry[];
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

export type EventTimelineEditItem = {
  id: string;
  atUtc: string | null;
  sortOrder: number;
  fields: EditFieldValue[];
};

export type EventEditData = {
  id: string;
  categoryId: string;
  startUtc: string;
  endUtc?: string;
  statusType: EventStoredStatus;
  locale: string;
  enabledLocales: string[];
  fields: EditFieldValue[];
  timeline: EventTimelineEditItem[];
};

export type EventSavePayload = {
  id?: string;
  categoryId: string;
  startUtc: string;
  endUtc?: string | null;
  statusType: EventStoredStatus;
  locale: string;
  fields: Record<string, string>;
  replaceTimeline?: boolean;
  timeline: Array<{ id: string; atUtc: string | null; sortOrder: number; fields: Record<string, string> }>;
};

export type EventStoredStatus = "waiting" | "ongoing";

export type EventStatusSection = "current" | "toCome" | "past";
