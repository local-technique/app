import type { AttachmentItem } from "../common/attachments";
import type { CategoryItem } from "../categories/types";

export type IncidentLocalizedText = {
  en?: string;
  fr?: string;
};

export type IncidentTimelineEntry = {
  id: string;
  atUtc: string | null;
  title: IncidentLocalizedText;
  details?: IncidentLocalizedText;
  createdBy?: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null;
  lastModifiedBy?: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null;
};

export type IncidentItem = {
  id: string;
  categoryCode: string;
  category?: Pick<CategoryItem, "id" | "key" | "icon" | "color" | "label">;
  title: IncidentLocalizedText;
  description: IncidentLocalizedText;
  location?: IncidentLocalizedText;
  startUtc: string;
  endUtc?: string;
  statusType: IncidentStoredStatus;
  statusText: IncidentLocalizedText;
  timeline: IncidentTimelineEntry[];
  attachments: AttachmentItem[];
  lastModifiedAt?: string;
  lastModifiedBy?: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null;
};

export type EditFieldValue = {
  fieldKey: string;
  value: string;
  exactValue?: string | null;
  fallbackLocale?: string | null;
  fallbackValue?: string | null;
};

export type IncidentTimelineEditItem = {
  id: string;
  atUtc: string | null;
  sortOrder: number;
  fields: EditFieldValue[];
};

export type IncidentEditData = {
  id: string;
  categoryId: string;
  startUtc: string;
  endUtc?: string;
  statusType: IncidentStoredStatus;
  locale: string;
  enabledLocales: string[];
  fields: EditFieldValue[];
  timeline: IncidentTimelineEditItem[];
};

export type IncidentSavePayload = {
  id?: string;
  categoryId: string;
  startUtc: string;
  endUtc?: string | null;
  statusType: IncidentStoredStatus;
  locale: string;
  fields: Record<string, string>;
  replaceTimeline?: boolean;
  timeline: Array<{ id: string; atUtc: string | null; sortOrder: number; fields: Record<string, string> }>;
};

export type IncidentStoredStatus = "waiting" | "ongoing";

export type IncidentStatusSection = "current" | "past";
