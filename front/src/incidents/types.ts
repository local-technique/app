import type { AttachmentItem } from "../common/attachments";

export type IncidentLocalizedText = {
  en?: string;
  fr?: string;
};

export type IncidentTimelineEntry = {
  id: string;
  atUtc: string;
  title: IncidentLocalizedText;
  details?: IncidentLocalizedText;
};

export type IncidentItem = {
  id: string;
  categoryCode: string;
  title: IncidentLocalizedText;
  shortDescription: IncidentLocalizedText;
  longDescription: IncidentLocalizedText;
  location?: IncidentLocalizedText;
  startUtc: string;
  endUtc?: string;
  timeline: IncidentTimelineEntry[];
  attachments: AttachmentItem[];
};

export type IncidentStatusSection = "current" | "past";
