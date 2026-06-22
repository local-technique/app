import type { LocaleCode } from "../../common/localeContent";
import { getAccessToken } from "../../auth/session";
import type { EditFieldValue, EventEditData, EventItem, EventLocalizedText, EventSavePayload, EventStoredStatus, EventTimelineEditItem, EventTimelineEntry } from "../types";
import type { EventsRepository } from "./eventsRepository";
import { mockEventsRepository } from "./mockEventsRepository";

type ApiMaintenanceTimelineItem = {
  id: string;
  at_utc: string | null;
  title: string;
  details: string;
};

type ApiMaintenanceListItem = {
  key: string;
  category_id: string;
  title: string;
  warning: string;
  description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  status_type: string;
  status_text: string;
  category?: { id: string; key: string; icon: string; color: string; label: string };
  timeline?: ApiMaintenanceTimelineItem[];
};

type ApiMaintenanceDetail = {
  key: string;
  category_id: string;
  title: string;
  warning: string;
  description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  status_type: string;
  status_text: string;
  category?: { id: string; key: string; icon: string; color: string; label: string };
  timeline: ApiMaintenanceTimelineItem[];
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

type ApiMaintenanceTimelineEditItem = {
  id: string;
  at_utc: string | null;
  sort_order: number;
  fields: ApiEditFieldValue[];
};

type ApiMaintenanceEditData = {
  key: string;
  category_id: string;
  start_utc: string;
  end_utc?: string;
  status_type: string;
  locale: string;
  enabled_locales: string[];
  fields: ApiEditFieldValue[];
  timeline: ApiMaintenanceTimelineEditItem[];
};

type ApiCreatedKeyResponse = { key: string };

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function useMockData(): boolean {
  return import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true";
}

function localized(locale: LocaleCode, value: string): EventLocalizedText {
  return locale === "en" ? { en: value } : { fr: value };
}

function toTimelineEntry(locale: LocaleCode, item: ApiMaintenanceTimelineItem): EventTimelineEntry {
  return {
    id: item.id,
    atUtc: item.at_utc,
    title: localized(locale, item.title ?? ""),
    details: localized(locale, item.details ?? ""),
  };
}

function toEventItem(locale: LocaleCode, value: ApiMaintenanceListItem | ApiMaintenanceDetail): EventItem {
  return {
    id: value.key,
    categoryCode: value.category_id,
    category: value.category,
    title: localized(locale, value.title ?? ""),
    description: localized(locale, value.description ?? ""),
    warning: localized(locale, value.warning ?? ""),
    location: localized(locale, value.location ?? ""),
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    statusType: value.status_type as EventStoredStatus,
    statusText: localized(locale, value.status_text ?? ""),
    timeline: "timeline" in value ? (value.timeline ?? []).map((item) => toTimelineEntry(locale, item)) : [],
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

function toTimelineEditItem(value: ApiMaintenanceTimelineEditItem): EventTimelineEditItem {
  return {
    id: value.id,
    atUtc: value.at_utc,
    sortOrder: value.sort_order,
    fields: value.fields.map(toEditField),
  };
}

function toEditData(value: ApiMaintenanceEditData): EventEditData {
  return {
    id: value.key,
    categoryId: value.category_id,
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    statusType: value.status_type as EventStoredStatus,
    locale: value.locale,
    enabledLocales: value.enabled_locales,
    fields: value.fields.map(toEditField),
    timeline: value.timeline.map(toTimelineEditItem),
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

async function sendJson<T>(url: string, method: string, body?: unknown): Promise<T | null> {
  const response = await fetch(url, {
    method,
    headers: { ...authHeaders(), "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return response.status === 204 ? null : ((await response.json()) as T);
}

function toApiPayload(payload: EventSavePayload, existingId?: string): Record<string, unknown> {
  return {
    ...(existingId ? { key: existingId } : {}),
    category_id: payload.categoryId,
    start_utc: payload.startUtc,
    end_utc: payload.endUtc ?? null,
    status_type: payload.statusType,
    locale: payload.locale,
    fields: payload.fields,
    replace_timeline: payload.replaceTimeline ?? false,
    timeline: payload.timeline.map((item) => ({
      id: item.id,
      at_utc: item.atUtc,
      sort_order: item.sortOrder,
      fields: item.fields,
    })),
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
        statusType: item.statusType,
        locale: preferredLanguage,
        enabledLocales: ["en", "fr"],
        fields: [
          { fieldKey: "title", value: item.title[preferredLanguage] ?? "" },
          { fieldKey: "description", value: item.description[preferredLanguage] ?? "" },
          { fieldKey: "warning", value: item.warning?.[preferredLanguage] ?? "" },
          { fieldKey: "location", value: item.location?.[preferredLanguage] ?? "" },
          { fieldKey: "status_text", value: item.statusText?.[preferredLanguage] ?? "" },
        ],
        timeline: item.timeline.map((entry, index) => ({
          id: entry.id,
          atUtc: entry.atUtc,
          sortOrder: index + 1,
          fields: [
            { fieldKey: "title", value: entry.title[preferredLanguage] ?? "" },
            { fieldKey: "details", value: entry.details?.[preferredLanguage] ?? "" },
          ],
        })),
      };
    }
    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiMaintenanceEditData>(`${apiBaseUrl()}/maintenances/${id}/edit?${params.toString()}`);
    return payload ? toEditData(payload) : null;
  }

  async save(payload: EventSavePayload, existingId?: string): Promise<string | void> {
    if (useMockData()) return;
    const path = existingId ? `/maintenances/${encodeURIComponent(existingId)}` : "/maintenances";
    const response = await sendJson<ApiCreatedKeyResponse>(`${apiBaseUrl()}${path}`, existingId ? "PUT" : "POST", toApiPayload(payload, existingId));
    return response?.key;
  }

  async delete(id: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/maintenances/${encodeURIComponent(id)}`, "DELETE");
  }

  async createTimelineEntry(id: string, preferredLanguage: LocaleCode, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<ApiMaintenanceTimelineItem> {
    if (useMockData()) return { id: crypto.randomUUID(), at_utc: payload.atUtc, title: payload.fields.title ?? "", details: payload.fields.details ?? "" };
    const params = new URLSearchParams({ locale: preferredLanguage });
    const result = await sendJson<ApiMaintenanceTimelineItem>(`${apiBaseUrl()}/maintenances/${encodeURIComponent(id)}/timeline?${params.toString()}`, "POST", {
      at_utc: payload.atUtc,
      sort_order: payload.sortOrder,
      fields: payload.fields,
    });
    if (!result) throw new Error("failed to create timeline entry");
    return result;
  }

  async updateTimelineEntry(id: string, entryId: string, preferredLanguage: LocaleCode, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<ApiMaintenanceTimelineItem> {
    if (useMockData()) return { id: entryId, at_utc: payload.atUtc, title: payload.fields.title ?? "", details: payload.fields.details ?? "" };
    const params = new URLSearchParams({ locale: preferredLanguage });
    const result = await sendJson<ApiMaintenanceTimelineItem>(`${apiBaseUrl()}/maintenances/${encodeURIComponent(id)}/timeline/${encodeURIComponent(entryId)}?${params.toString()}`, "PUT", {
      at_utc: payload.atUtc,
      sort_order: payload.sortOrder,
      fields: payload.fields,
    });
    if (!result) throw new Error("failed to update timeline entry");
    return result;
  }

  async deleteTimelineEntry(id: string, entryId: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/maintenances/${encodeURIComponent(id)}/timeline/${encodeURIComponent(entryId)}`, "DELETE");
  }
}

export const apiEventsRepository = new ApiEventsRepository();
