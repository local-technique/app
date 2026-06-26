import { formatLocalDate, formatLocalDateTime, parseUtc } from "../common/date";
import type { LocaleCode } from "../common/localeContent";
import { resolveLocalized } from "../common/localeContent";
import { fuzzyMatch } from "../common/search";
import type {
  IncidentItem,
  IncidentLocalizedText,
  IncidentStatusSection,
  IncidentStoredStatus,
  IncidentTimelineEntry,
} from "./types";

export type IncidentTimelineEntryViewModel = {
  id: string;
  atUtc: string | null;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
  lastModifiedBy?: { initials: string; fullName: string } | null;
};

export type IncidentViewModel = {
  id: string;
  status: IncidentStatusSection;
  statusType: IncidentStoredStatus | "finished" | "planned";
  statusText: string;
  title: string;
  description: string;
  location: string;
  dateLabel: string;
  startDateFormatted: string;
  endDateFormatted?: string;
  timeline: IncidentTimelineEntryViewModel[];
  raw: IncidentItem;
};

function resolve(value: IncidentLocalizedText | undefined, locale: LocaleCode): string {
  if (!value) {
    return "";
  }
  return resolveLocalized(value, locale);
}

function computeStatusType(stored: IncidentStoredStatus, endUtc: string | undefined, startUtc: string): IncidentStoredStatus | "finished" | "planned" {
  if (endUtc && Date.parse(endUtc) < Date.now()) {
    return "finished";
  }
  if (stored === "ongoing" && Date.parse(startUtc) > Date.now()) {
    return "planned";
  }
  return stored;
}

function toIncidentStatus(input: IncidentItem): IncidentStatusSection {
  const nowMs = Date.now();
  const startMs = Date.parse(input.startUtc);
  if (!input.endUtc) {
    return "current";
  }
  return nowMs >= startMs && nowMs <= Date.parse(input.endUtc) ? "current" : "past";
}

function formatIncidentDateLabel(incident: IncidentItem, locale: LocaleCode): string {
  const start = formatLocalDate(parseUtc(incident.startUtc), locale);
  if (!incident.endUtc) {
    return start;
  }
  const end = formatLocalDate(parseUtc(incident.endUtc), locale);
  return `${start} - ${end}`;
}

function toTimelineEntryViewModel(entry: IncidentTimelineEntry, locale: LocaleCode): IncidentTimelineEntryViewModel {
  const atDate = entry.atUtc ? parseUtc(entry.atUtc) : null;
  const fb = entry.lastModifiedBy;
  const firstChar = fb?.firstName?.[0] ?? fb?.lastName?.[0] ?? fb?.email?.[0] ?? null;
  const lastChar = fb?.firstName && fb?.lastName ? fb.lastName[0] : null;
  const initials = firstChar && lastChar ? `${firstChar}${lastChar}`.toUpperCase() : firstChar?.toUpperCase() ?? null;
  const fullName = fb?.firstName && fb?.lastName ? `${fb.firstName} ${fb.lastName}` : (fb?.firstName ?? fb?.lastName ?? fb?.email ?? null);
  return {
    id: entry.id,
    atUtc: entry.atUtc,
    atLabel: atDate ? formatLocalDateTime(atDate, locale) : "Pending",
    atDateLabel: atDate ? new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(atDate) : "",
    atTimeLabel: atDate ? new Intl.DateTimeFormat(locale, { timeStyle: "short" }).format(atDate) : "",
    isPending: !entry.atUtc,
    title: resolve(entry.title, locale),
    details: resolve(entry.details, locale),
    lastModifiedBy: initials || fullName ? { initials: initials ?? "", fullName: fullName ?? "" } : null,
  };
}

export function toIncidentViewModel(incident: IncidentItem, locale: LocaleCode): IncidentViewModel {
  const timeline = incident.timeline.map((entry) => toTimelineEntryViewModel(entry, locale));
  const statusType = computeStatusType(incident.statusType, incident.endUtc, incident.startUtc);

  return {
    id: incident.id,
    status: toIncidentStatus(incident),
    statusType,
    statusText: resolve(incident.statusText, locale),
    title: resolve(incident.title, locale),
    description: resolve(incident.description, locale),
    location: resolve(incident.location, locale),
    dateLabel: formatIncidentDateLabel(incident, locale),
    startDateFormatted: formatLocalDate(parseUtc(incident.startUtc), locale),
    endDateFormatted: incident.endUtc ? formatLocalDate(parseUtc(incident.endUtc), locale) : undefined,
    timeline,
    raw: incident,
  };
}

export function matchesIncidentQuery(incident: IncidentItem, query: string, locale: LocaleCode): boolean {
  if (!query.trim()) {
    return true;
  }

  const timelineText = incident.timeline
    .map((entry) => `${resolve(entry.title, locale)} ${resolve(entry.details, locale)}`)
    .join(" ");

  const haystack = [
    resolve(incident.title, locale),
    resolve(incident.description, locale),
    resolve(incident.location, locale),
    incident.categoryCode,
    timelineText,
  ]
    .join(" ")
    .trim();

  return fuzzyMatch(query, haystack);
}

export function groupByStatus(
  incidents: IncidentViewModel[],
): Record<IncidentStatusSection, IncidentViewModel[]> {
  return incidents.reduce<Record<IncidentStatusSection, IncidentViewModel[]>>(
    (groups, item) => {
      groups[item.status].push(item);
      return groups;
    },
    {
      current: [],
      past: [],
    },
  );
}
