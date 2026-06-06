import type { LocaleCode } from "../../common/localeContent";
import { getAccessToken } from "../../auth/session";
import type { EditFieldValue, EventEditData, EventItem, EventLocalizedText, EventSavePayload } from "../types";
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
  category?: { id: string; code: string; icon: string; label: string };
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
  category?: { id: string; code: string; icon: string; label: string };
  last_modified_at?: string | null;
  last_modified_by?: { id: string; email: string } | null;
};

type ApiEditFieldValue = {
  field_key: string;
  value: string;
  exact_value?: string | null;
  fallback_locale?: string | null;
  fallback_value?: string | null;
};

type ApiMaintenanceEditData = {
  id: string;
  category_id: string;
  start_utc: string;
  end_utc?: string;
  notified_at_utc?: string;
  locale: string;
  enabled_locales: string[];
  fields: ApiEditFieldValue[];
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
    category: value.category,
    title: localized(locale, value.title ?? ""),
    shortDescription: localized(locale, value.short_description ?? ""),
    longDescription: localized(locale, "long_description" in value ? (value.long_description ?? "") : ""),
    warning: localized(locale, value.warning ?? ""),
    location: localized(locale, value.location ?? ""),
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    notifiedAtUtc: value.notified_at_utc,
    attachments: [],
    lastModifiedAt: "last_modified_at" in value ? (value.last_modified_at ?? undefined) : undefined,
    lastModifiedBy: "last_modified_by" in value ? (value.last_modified_by ?? null) : undefined,
  };
}

function toEditField(value: ApiEditFieldValue): EditFieldValue {
  return {
    fieldKey: value.field_key,
    value: value.value,
    exactValue: value.exact_value,
    fallbackLocale: value.fallback_locale,
    fallbackValue: value.fallback_value,
  };
}

function toEditData(value: ApiMaintenanceEditData): EventEditData {
  return {
    id: value.id,
    categoryId: value.category_id,
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    notifiedAtUtc: value.notified_at_utc,
    locale: value.locale,
    enabledLocales: value.enabled_locales,
    fields: value.fields.map(toEditField),
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

async function sendJson(url: string, method: string, body?: unknown): Promise<void> {
  const response = await fetch(url, {
    method,
    headers: { ...authHeaders(), "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
}

function toApiPayload(payload: EventSavePayload): Record<string, unknown> {
  return {
    id: payload.id,
    category_id: payload.categoryId,
    start_utc: payload.startUtc,
    end_utc: payload.endUtc ?? null,
    notified_at_utc: payload.notifiedAtUtc ?? null,
    locale: payload.locale,
    fields: payload.fields,
  };
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

  async editData(id: string, preferredLanguage: LocaleCode): Promise<EventEditData | null> {
    if (useMockData()) {
      const item = await mockEventsRepository.byId(id, preferredLanguage);
      if (!item) return null;
      return {
        id: item.id,
        categoryId: item.categoryCode,
        startUtc: item.startUtc,
        endUtc: item.endUtc,
        notifiedAtUtc: item.notifiedAtUtc,
        locale: preferredLanguage,
        enabledLocales: ["en", "fr"],
        fields: [
          { fieldKey: "title", value: item.title[preferredLanguage] ?? "" },
          { fieldKey: "short_description", value: item.shortDescription[preferredLanguage] ?? "" },
          { fieldKey: "long_description", value: item.longDescription[preferredLanguage] ?? "" },
          { fieldKey: "warning", value: item.warning?.[preferredLanguage] ?? "" },
          { fieldKey: "location", value: item.location?.[preferredLanguage] ?? "" },
        ],
      };
    }
    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiMaintenanceEditData>(`${apiBaseUrl()}/maintenances/${id}/edit?${params.toString()}`);
    return payload ? toEditData(payload) : null;
  }

  async save(payload: EventSavePayload, existingId?: string): Promise<void> {
    if (useMockData()) return;
    const path = existingId ? `/maintenances/${encodeURIComponent(existingId)}` : "/maintenances";
    await sendJson(`${apiBaseUrl()}${path}`, existingId ? "PUT" : "POST", toApiPayload(payload));
  }

  async delete(id: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/maintenances/${encodeURIComponent(id)}`, "DELETE");
  }
}

export const apiEventsRepository = new ApiEventsRepository();
