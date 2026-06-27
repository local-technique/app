import { classifyEventStatus, formatLocalDate, formatLocalDateTime, parseUtc } from "../common/date";
import type { LocaleCode } from "../common/localeContent";
import { resolveLocalized } from "../common/localeContent";
import { fuzzyMatch } from "../common/search";
import type { EventItem, EventLocalizedText, EventStatusSection, EventStoredStatus, EventTimelineEntry } from "./types";

export type EventTimelineEntryViewModel = {
  id: string;
  atUtc: string | null;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
  createdBy?: { initials: string; fullName: string; id: string } | null;
  lastModifiedBy?: { initials: string; fullName: string } | null;
};

export type EventViewModel = {
  id: string;
  status: EventStatusSection;
  statusType: EventStoredStatus | "finished" | "planned";
  statusText: string;
  title: string;
  warning: string;
  description: string;
  location: string;
  dateLabel: string;
  startDateFormatted: string;
  endDateFormatted?: string;
  timeline: EventTimelineEntryViewModel[];
  raw: EventItem;
};

function resolve(value: EventLocalizedText | undefined, locale: LocaleCode): string {
  if (!value) {
    return "";
  }
  return resolveLocalized(value, locale);
}

function computeStatusType(stored: EventStoredStatus, endUtc: string | undefined, startUtc: string): EventStoredStatus | "finished" | "planned" {
  if (endUtc && Date.parse(endUtc) < Date.now()) {
    return "finished";
  }
  if (stored === "ongoing" && Date.parse(startUtc) > Date.now()) {
    return "planned";
  }
  return stored;
}

function formatEventDateLabel(event: EventItem, locale: LocaleCode): string {
  const start = formatLocalDate(parseUtc(event.startUtc), locale);
  if (!event.endUtc) {
    return start;
  }
  const end = formatLocalDate(parseUtc(event.endUtc), locale);
  return `${start} - ${end}`;
}

function toTimelineEntryViewModel(entry: EventTimelineEntry, locale: LocaleCode): EventTimelineEntryViewModel {
  const atDate = entry.atUtc ? parseUtc(entry.atUtc) : null;

  function toUserDisplay(user: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null | undefined): { initials: string; fullName: string; id: string } | null {
    if (!user) return null;
    const firstChar = user.firstName?.[0] ?? user.lastName?.[0] ?? user.email[0] ?? '';
    const lastChar = user.firstName && user.lastName ? user.lastName[0] : null;
    const initials = firstChar && lastChar ? `${firstChar}${lastChar}`.toUpperCase() : firstChar.toUpperCase();
    const fullName = user.firstName && user.lastName ? `${user.firstName} ${user.lastName}` : (user.firstName ?? user.lastName ?? user.email ?? '');
    return { initials, fullName, id: user.id };
  }

  const displayCreatedBy = toUserDisplay(entry.createdBy);
  const displayLastModifiedBy = toUserDisplay(entry.lastModifiedBy);
  const differentModifier = displayLastModifiedBy && (!displayCreatedBy || displayLastModifiedBy.id !== displayCreatedBy.id)
    ? { initials: displayLastModifiedBy.initials, fullName: displayLastModifiedBy.fullName }
    : null;

  return {
    id: entry.id,
    atUtc: entry.atUtc,
    atLabel: atDate ? formatLocalDateTime(atDate, locale) : "Pending",
    atDateLabel: atDate ? new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(atDate) : "",
    atTimeLabel: atDate ? new Intl.DateTimeFormat(locale, { timeStyle: "short" }).format(atDate) : "",
    isPending: !entry.atUtc,
    title: resolve(entry.title, locale),
    details: resolve(entry.details, locale),
    createdBy: displayCreatedBy ? { initials: displayCreatedBy.initials, fullName: displayCreatedBy.fullName, id: displayCreatedBy.id } : null,
    lastModifiedBy: differentModifier,
  };
}

export function toEventViewModel(event: EventItem, locale: LocaleCode): EventViewModel {
  const timeline = event.timeline.map((entry) => toTimelineEntryViewModel(entry, locale));
  const statusType = computeStatusType(event.statusType, event.endUtc, event.startUtc);
  return {
    id: event.id,
    status: classifyEventStatus({ startUtc: event.startUtc, endUtc: event.endUtc }),
    statusType,
    statusText: resolve(event.statusText, locale),
    title: resolve(event.title, locale),
    warning: resolve(event.warning, locale),
    description: resolve(event.description, locale),
    location: resolve(event.location, locale),
    dateLabel: formatEventDateLabel(event, locale),
    startDateFormatted: formatLocalDate(parseUtc(event.startUtc), locale),
    endDateFormatted: event.endUtc ? formatLocalDate(parseUtc(event.endUtc), locale) : undefined,
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
