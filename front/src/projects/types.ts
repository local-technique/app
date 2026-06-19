import type { AttachmentItem } from "../common/attachments";
import type { CategoryItem } from "../categories/types";

export type ProjectLocalizedText = {
  en?: string;
  fr?: string;
};

export type ProjectStoredStatus = "waiting" | "ongoing";
export type ProjectDisplayStatus = "waiting" | "ongoing" | "finished";
export type ProjectStatusSection = "ongoing" | "toCome" | "finished";

export type ProjectTimelineEntry = {
  id: string;
  atUtc: string | null;
  title: ProjectLocalizedText;
  details?: ProjectLocalizedText;
};

export type ProjectItem = {
  id: string;
  categoryCode: string;
  category?: Pick<CategoryItem, "id" | "key" | "icon" | "color" | "label">;
  title: ProjectLocalizedText;
  description: ProjectLocalizedText;
  startUtc?: string;
  endUtc?: string;
  statusType: ProjectStoredStatus;
  statusText: ProjectLocalizedText;
  timeline: ProjectTimelineEntry[];
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

export type ProjectTimelineEditItem = {
  id: string;
  atUtc: string | null;
  sortOrder: number;
  fields: EditFieldValue[];
};

export type ProjectEditData = {
  id: string;
  categoryId: string;
  startUtc?: string;
  endUtc?: string;
  statusType: ProjectStoredStatus;
  locale: string;
  enabledLocales: string[];
  fields: EditFieldValue[];
  timeline: ProjectTimelineEditItem[];
};

export type ProjectSavePayload = {
  id?: string;
  categoryId: string;
  startUtc?: string | null;
  endUtc?: string | null;
  statusType: ProjectStoredStatus;
  locale: string;
  fields: Record<string, string>;
  replaceTimeline?: boolean;
  timeline: Array<{ id: string; atUtc: string | null; sortOrder: number; fields: Record<string, string> }>;
};
