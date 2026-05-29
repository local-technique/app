import type { LocaleCode } from "../../common/localeContent";
import { getAccessToken } from "../../auth/session";
import type { IncidentItem, IncidentLocalizedText, IncidentTimelineEntry } from "../types";
import type { IncidentsRepository } from "./incidentsRepository";
import { mockIncidentsRepository } from "./mockIncidentsRepository";

type ApiIncidentListItem = {
  id: string;
  category_code: string;
  title: string;
  short_description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
};

type ApiIncidentTimelineItem = {
  id: string;
  at_utc: string;
  title: string;
  details: string;
};

type ApiIncidentDetail = {
  id: string;
  category_code: string;
  title: string;
  short_description: string;
  long_description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  timeline: ApiIncidentTimelineItem[];
};

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function useMockData(): boolean {
  return import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true";
}

function localized(locale: LocaleCode, value: string): IncidentLocalizedText {
  return locale === "en" ? { en: value } : { fr: value };
}

function toTimelineEntry(locale: LocaleCode, item: ApiIncidentTimelineItem): IncidentTimelineEntry {
  return {
    id: item.id,
    atUtc: item.at_utc,
    title: localized(locale, item.title ?? ""),
    details: localized(locale, item.details ?? ""),
  };
}

function toIncidentItem(locale: LocaleCode, value: ApiIncidentListItem | ApiIncidentDetail): IncidentItem {
  return {
    id: value.id,
    categoryCode: value.category_code,
    title: localized(locale, value.title ?? ""),
    shortDescription: localized(locale, value.short_description ?? ""),
    longDescription: localized(locale, "long_description" in value ? (value.long_description ?? "") : ""),
    location: localized(locale, value.location ?? ""),
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    timeline: "timeline" in value ? value.timeline.map((item) => toTimelineEntry(locale, item)) : [],
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

export class ApiIncidentsRepository implements IncidentsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<IncidentItem[]> {
    if (useMockData()) {
      return mockIncidentsRepository.list(preferredLanguage, query);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    if (query.trim()) {
      params.set("q", query.trim());
    }

    const payload = await fetchJson<ApiIncidentListItem[]>(`${apiBaseUrl()}/incidents?${params.toString()}`);
    return payload.map((value) => toIncidentItem(preferredLanguage, value));
  }

  async byId(id: string, preferredLanguage: LocaleCode): Promise<IncidentItem | null> {
    if (useMockData()) {
      return mockIncidentsRepository.byId(id, preferredLanguage);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiIncidentDetail>(`${apiBaseUrl()}/incidents/${id}?${params.toString()}`);
    return payload ? toIncidentItem(preferredLanguage, payload) : null;
  }
}

export const apiIncidentsRepository = new ApiIncidentsRepository();
