import type { LocaleCode } from "../../common/localeContent";
import { getAccessToken } from "../../auth/session";
import type {
  EditFieldValue,
  IncidentEditData,
  IncidentItem,
  IncidentLocalizedText,
  IncidentSavePayload,
  IncidentStoredStatus,
  IncidentTimelineEditItem,
  IncidentTimelineEntry,
} from "../types";
import type { IncidentsRepository } from "./incidentsRepository";
import { mockIncidentsRepository } from "./mockIncidentsRepository";

type ApiIncidentListItem = {
  key: string;
  category_id: string;
  title: string;
  description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  status_type: string;
  status_text: string;
  timeline?: ApiIncidentTimelineItem[];
  category?: { id: string; key: string; icon: string; color: string; label: string };
};

type ApiIncidentTimelineItem = {
  id: string;
  at_utc: string | null;
  title: string;
  details: string;
};

type ApiIncidentDetail = {
  key: string;
  category_id: string;
  title: string;
  description: string;
  location: string;
  start_utc: string;
  end_utc?: string;
  status_type: string;
  status_text: string;
  timeline: ApiIncidentTimelineItem[];
  category?: { id: string; key: string; icon: string; color: string; label: string };
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

type ApiIncidentTimelineEditItem = {
  id: string;
  at_utc: string | null;
  sort_order: number;
  fields: ApiEditFieldValue[];
};

type ApiIncidentEditData = {
  key: string;
  category_id: string;
  start_utc: string;
  end_utc?: string;
  status_type: string;
  locale: string;
  enabled_locales: string[];
  fields: ApiEditFieldValue[];
  timeline: ApiIncidentTimelineEditItem[];
};

type ApiCreatedKeyResponse = { key: string };

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
    id: value.key,
    categoryCode: value.category_id,
    category: value.category,
    title: localized(locale, value.title ?? ""),
    description: localized(locale, value.description ?? ""),
    location: localized(locale, value.location ?? ""),
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    statusType: value.status_type as IncidentStoredStatus,
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

function toTimelineEditItem(value: ApiIncidentTimelineEditItem): IncidentTimelineEditItem {
  return {
    id: value.id,
    atUtc: value.at_utc,
    sortOrder: value.sort_order,
    fields: value.fields.map(toEditField),
  };
}

function toEditData(value: ApiIncidentEditData): IncidentEditData {
  return {
    id: value.key,
    categoryId: value.category_id,
    startUtc: value.start_utc,
    endUtc: value.end_utc,
    statusType: value.status_type as IncidentStoredStatus,
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

function toApiPayload(payload: IncidentSavePayload, existingId?: string): Record<string, unknown> {
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

  async editData(id: string, preferredLanguage: LocaleCode): Promise<IncidentEditData | null> {
    if (useMockData()) {
      const item = await mockIncidentsRepository.byId(id, preferredLanguage);
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
    const payload = await fetchJsonOrNull<ApiIncidentEditData>(`${apiBaseUrl()}/incidents/${id}/edit?${params.toString()}`);
    return payload ? toEditData(payload) : null;
  }

  async save(payload: IncidentSavePayload, existingId?: string): Promise<string | void> {
    if (useMockData()) return;
    const path = existingId ? `/incidents/${encodeURIComponent(existingId)}` : "/incidents";
    const response = await sendJson<ApiCreatedKeyResponse>(`${apiBaseUrl()}${path}`, existingId ? "PUT" : "POST", toApiPayload(payload, existingId));
    return response?.key;
  }

  async delete(id: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/incidents/${encodeURIComponent(id)}`, "DELETE");
  }
}

export const apiIncidentsRepository = new ApiIncidentsRepository();
