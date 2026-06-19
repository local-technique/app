import { formatLocalDateTime, parseUtc } from "../common/date";
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
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
};

export type IncidentViewModel = {
  id: string;
  status: IncidentStatusSection;
  statusType: IncidentStoredStatus;
  statusText: string;
  title: string;
  shortDescription: string;
  longDescription: string;
  location: string;
  dateLabel: string;
  timeline: IncidentTimelineEntryViewModel[];
  raw: IncidentItem;
};

function resolve(value: IncidentLocalizedText | undefined, locale: LocaleCode): string {
  if (!value) {
    return "";
  }
  return resolveLocalized(value, locale);
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
  const start = formatLocalDateTime(parseUtc(incident.startUtc), locale);
  if (!incident.endUtc) {
    return start;
  }
  const end = formatLocalDateTime(parseUtc(incident.endUtc), locale);
  return `${start} - ${end}`;
}

function toTimelineEntryViewModel(entry: IncidentTimelineEntry, locale: LocaleCode): IncidentTimelineEntryViewModel {
  const atDate = entry.atUtc ? parseUtc(entry.atUtc) : null;
  return {
    id: entry.id,
    atLabel: atDate ? formatLocalDateTime(atDate, locale) : "Pending",
    atDateLabel: atDate ? new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(atDate) : "",
    atTimeLabel: atDate ? new Intl.DateTimeFormat(locale, { timeStyle: "short" }).format(atDate) : "",
    isPending: !entry.atUtc,
    title: resolve(entry.title, locale),
    details: resolve(entry.details, locale),
  };
}

export function toIncidentViewModel(incident: IncidentItem, locale: LocaleCode): IncidentViewModel {
  const timeline = incident.timeline.map((entry) => toTimelineEntryViewModel(entry, locale));

  return {
    id: incident.id,
    status: toIncidentStatus(incident),
    statusType: incident.statusType,
    statusText: resolve(incident.statusText, locale),
    title: resolve(incident.title, locale),
    shortDescription: resolve(incident.shortDescription, locale),
    longDescription: resolve(incident.longDescription, locale),
    location: resolve(incident.location, locale),
    dateLabel: formatIncidentDateLabel(incident, locale),
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
    resolve(incident.shortDescription, locale),
    resolve(incident.longDescription, locale),
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
