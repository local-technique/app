import { classifyEventStatus, formatLocalDateTime, parseUtc } from "../common/date";
import type { LocaleCode } from "../common/localeContent";
import { resolveLocalized } from "../common/localeContent";
import { fuzzyMatch } from "../common/search";
import type {
  IncidentItem,
  IncidentLocalizedText,
  IncidentStatusSection,
  IncidentTimelineEntry,
} from "./types";

export type IncidentTimelineEntryViewModel = {
  id: string;
  atLabel: string;
  title: string;
  details: string;
};

export type IncidentViewModel = {
  id: string;
  status: IncidentStatusSection;
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
  const status = classifyEventStatus({ startUtc: input.startUtc, endUtc: input.endUtc });
  return status === "current" ? "current" : "past";
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
  return {
    id: entry.id,
    atLabel: formatLocalDateTime(parseUtc(entry.atUtc), locale),
    title: resolve(entry.title, locale),
    details: resolve(entry.details, locale),
  };
}

export function toIncidentViewModel(incident: IncidentItem, locale: LocaleCode): IncidentViewModel {
  const timeline = [...incident.timeline]
    .sort((a, b) => Date.parse(a.atUtc) - Date.parse(b.atUtc))
    .map((entry) => toTimelineEntryViewModel(entry, locale));

  return {
    id: incident.id,
    status: toIncidentStatus(incident),
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
