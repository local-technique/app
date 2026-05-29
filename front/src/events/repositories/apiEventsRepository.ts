import type { LocaleCode } from "../../common/localeContent";
import { getAccessToken } from "../../auth/session";
import type { EventItem, EventLocalizedText } from "../types";
import type { EventsRepository } from "./eventsRepository";
import { mockEventsRepository } from "./mockEventsRepository";

type ApiMaintenanceListItem = {
  id: string;
  category_code: string;
  title: string;
  warning: string;
  short_description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  notified_at_utc?: string;
};

type ApiMaintenanceDetail = {
  id: string;
  category_code: string;
  title: string;
  warning: string;
  short_description: string;
  long_description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  notified_at_utc?: string;
};

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function useMockData(): boolean {
  return import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true";
}

function localized(locale: LocaleCode, value: string): EventLocalizedText {
  return locale === "en" ? { en: value } : { fr: value };
}

function toEventItem(locale: LocaleCode, value: ApiMaintenanceListItem | ApiMaintenanceDetail): EventItem {
  return {
    id: value.id,
    categoryCode: value.category_code,
    title: localized(locale, value.title ?? ""),
    shortDescription: localized(locale, value.short_description ?? ""),
    longDescription: localized(locale, "long_description" in value ? (value.long_description ?? "") : ""),
    warning: localized(locale, value.warning ?? ""),
    location: localized(locale, value.location ?? ""),
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    notifiedAtUtc: value.notified_at_utc,
    attachments: [],
  };
}

function authHeaders(): HeadersInit {
  const token = getAccessToken();
  if (!token) {
    throw new Error("missing access token");
  }
  return {
    Authorization: `Bearer ${token}`,
  };
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url, { headers: authHeaders() });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function fetchJsonOrNull<T>(url: string): Promise<T | null> {
  const response = await fetch(url, { headers: authHeaders() });
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export class ApiEventsRepository implements EventsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<EventItem[]> {
    if (useMockData()) {
      return mockEventsRepository.list(preferredLanguage, query);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    if (query.trim()) {
      params.set("q", query.trim());
    }
    const payload = await fetchJson<ApiMaintenanceListItem[]>(`${apiBaseUrl()}/maintenances?${params.toString()}`);
    return payload.map((value) => toEventItem(preferredLanguage, value));
  }

  async byId(id: string, preferredLanguage: LocaleCode): Promise<EventItem | null> {
    if (useMockData()) {
      return mockEventsRepository.byId(id, preferredLanguage);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiMaintenanceDetail>(`${apiBaseUrl()}/maintenances/${id}?${params.toString()}`);
    return payload ? toEventItem(preferredLanguage, payload) : null;
  }
}

export const apiEventsRepository = new ApiEventsRepository();
