import { classifyEventStatus, formatLocalDateTime, parseUtc } from "../common/date";
import type { LocaleCode } from "../common/localeContent";
import { resolveLocalized } from "../common/localeContent";
import { fuzzyMatch } from "../common/search";
import type { EventItem, EventLocalizedText, EventStatusSection, EventStoredStatus, EventTimelineEntry } from "./types";

export type EventTimelineEntryViewModel = {
  id: string;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
};

export type EventViewModel = {
  id: string;
  status: EventStatusSection;
  statusType: EventStoredStatus;
  statusText: string;
  title: string;
  warning: string;
  description: string;
  location: string;
  dateLabel: string;
  timeline: EventTimelineEntryViewModel[];
  raw: EventItem;
};

function resolve(value: EventLocalizedText | undefined, locale: LocaleCode): string {
  if (!value) {
    return "";
  }
  return resolveLocalized(value, locale);
}

function formatEventDateLabel(event: EventItem, locale: LocaleCode): string {
  const start = formatLocalDateTime(parseUtc(event.startUtc), locale);
  if (!event.endUtc) {
    return start;
  }
  const end = formatLocalDateTime(parseUtc(event.endUtc), locale);
  return `${start} - ${end}`;
}

function toTimelineEntryViewModel(entry: EventTimelineEntry, locale: LocaleCode): EventTimelineEntryViewModel {
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

export function toEventViewModel(event: EventItem, locale: LocaleCode): EventViewModel {
  const timeline = event.timeline.map((entry) => toTimelineEntryViewModel(entry, locale));
  return {
    id: event.id,
    status: classifyEventStatus({ startUtc: event.startUtc, endUtc: event.endUtc }),
    statusType: event.statusType,
    statusText: resolve(event.statusText, locale),
    title: resolve(event.title, locale),
    warning: resolve(event.warning, locale),
    description: resolve(event.description, locale),
    location: resolve(event.location, locale),
    dateLabel: formatEventDateLabel(event, locale),
    timeline,
    raw: event,
  };
}

export function matchesEventQuery(event: EventItem, query: string, locale: LocaleCode): boolean {
  if (!query.trim()) {
    return true;
  }

  const timelineText = event.timeline
    .map((entry) => `${resolve(entry.title, locale)} ${resolve(entry.details, locale)}`)
    .join(" ");

  const haystack = [
    resolve(event.title, locale),
    resolve(event.description, locale),
    resolve(event.location, locale),
    event.categoryCode,
    timelineText,
  ]
    .join(" ")
    .trim();

  return fuzzyMatch(query, haystack);
}

export function groupByStatus(events: EventViewModel[]): Record<EventStatusSection, EventViewModel[]> {
  return events.reduce<Record<EventStatusSection, EventViewModel[]>>(
    (groups, item) => {
      groups[item.status].push(item);
      return groups;
    },
    {
      current: [],
      toCome: [],
      past: [],
    },
  );
}
