export type EventDateInput = {
  startUtc: string;
  endUtc?: string;
};

export type EventStatus = "toCome" | "current" | "past";

export function classifyEventStatus(input: EventDateInput, now = new Date()): EventStatus {
  const nowMs = now.getTime();
  const startMs = Date.parse(input.startUtc);

  if (!input.endUtc) {
    return nowMs < startMs ? "toCome" : "past";
  }

  const endMs = Date.parse(input.endUtc);
  if (nowMs < startMs) {
    return "toCome";
  }
  if (nowMs > endMs) {
    return "past";
  }
  return "current";
}

export function parseUtc(utcIso: string): Date {
  return new Date(utcIso);
}

export function formatLocalDate(value: Date | string, locale: string): string {
  const date = value instanceof Date ? value : new Date(value);
  return new Intl.DateTimeFormat(locale, {
    dateStyle: "long",
  }).format(date);
}

export function formatLocalDateTimeLong(value: Date | string, locale: string): string {
  const date = value instanceof Date ? value : new Date(value);
  return new Intl.DateTimeFormat(locale, {
    dateStyle: "long",
    timeStyle: "short",
  }).format(date);
}

export function formatLocalDateTime(value: Date | string, locale: string): string {
  const date = value instanceof Date ? value : new Date(value);
  return new Intl.DateTimeFormat(locale, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
