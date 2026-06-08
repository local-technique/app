import type { AttachmentItem } from "../common/attachments";
import type { CategoryItem } from "../categories/types";

export type ProjectLocalizedText = {
  en?: string;
  fr?: string;
};

export type ProjectStoredStatus = "waiting" | "ongoing";
export type ProjectDisplayStatus = "waiting" | "ongoing" | "finished";
export type ProjectStatusSection = "ongoing" | "toCome" | "finished";

export type ProjectItem = {
  id: string;
  categoryCode: string;
  category?: Pick<CategoryItem, "id" | "code" | "icon" | "color" | "label">;
  title: ProjectLocalizedText;
  description: ProjectLocalizedText;
  startUtc?: string;
  endUtc?: string;
  statusType: ProjectStoredStatus;
  statusText: ProjectLocalizedText;
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

export type ProjectEditData = {
  id: string;
  categoryId: string;
  startUtc?: string;
  endUtc?: string;
  statusType: ProjectStoredStatus;
  locale: string;
  enabledLocales: string[];
  fields: EditFieldValue[];
};

export type ProjectSavePayload = {
  id: string;
  categoryId: string;
  startUtc?: string | null;
  endUtc?: string | null;
  statusType: ProjectStoredStatus;
  locale: string;
  fields: Record<string, string>;
};
