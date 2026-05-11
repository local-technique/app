import { classifyEventStatus, formatLocalDateTime, parseUtc } from "../common/date";
import type { LocaleCode } from "../common/localeContent";
import { resolveLocalized } from "../common/localeContent";
import { fuzzyMatch } from "../common/search";
import type { EventItem, EventLocalizedText, EventStatusSection } from "./types";

export type EventViewModel = {
  id: string;
  status: EventStatusSection;
  title: string;
  warning: string;
  shortDescription: string;
  longDescription: string;
  location: string;
  dateLabel: string;
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

export function toEventViewModel(event: EventItem, locale: LocaleCode): EventViewModel {
  return {
    id: event.id,
    status: classifyEventStatus({ startUtc: event.startUtc, endUtc: event.endUtc }),
    title: resolve(event.title, locale),
    warning: resolve(event.warning, locale),
    shortDescription: resolve(event.shortDescription, locale),
    longDescription: resolve(event.longDescription, locale),
    location: resolve(event.location, locale),
    dateLabel: formatEventDateLabel(event, locale),
    raw: event,
  };
}

export function matchesEventQuery(event: EventItem, query: string, locale: LocaleCode): boolean {
  if (!query.trim()) {
    return true;
  }

  const haystack = [
    resolve(event.title, locale),
    resolve(event.shortDescription, locale),
    resolve(event.longDescription, locale),
    resolve(event.location, locale),
    event.categoryCode,
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
