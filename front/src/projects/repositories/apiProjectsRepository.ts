import type { LocaleCode } from "../../common/localeContent";
import { mapUserRef } from "../../common/apiMapping";
import { authenticatedFetch } from "../../auth/authenticatedFetch";
import type { EditFieldValue, ProjectEditData, ProjectItem, ProjectLocalizedText, ProjectSavePayload, ProjectTimelineEditItem, ProjectTimelineEntry } from "../types";
import { mockProjectsRepository } from "./mockProjectsRepository";
import type { ProjectsRepository } from "./projectsRepository";

type ApiProjectTimelineItem = {
  id: string;
  at_utc: string | null;
  title: string;
  details: string;
  created_by?: { id: string; email: string; first_name?: string | null; last_name?: string | null } | null;
  last_modified_by?: { id: string; email: string; first_name?: string | null; last_name?: string | null } | null;
};

type ApiProjectListItem = {
  key: string;
  category_id: string;
  title: string;
  description: string;
  start_utc?: string | null;
  end_utc?: string | null;
  status_type: "waiting" | "ongoing";
  status_text: string;
  category?: { id: string; key: string; icon: string; color: string; label: string };
  timeline?: ApiProjectTimelineItem[];
};

type ApiProjectDetail = ApiProjectListItem & {
  timeline: ApiProjectTimelineItem[];
  last_modified_at?: string | null;
  last_modified_by?: { id: string; email: string; first_name?: string | null; last_name?: string | null } | null;
};

type ApiEditFieldValue = {
  field_key: string;
  value: string;
  exact_value?: string | null;
  fallback_locale?: string | null;
  fallback_value?: string | null;
};

type ApiProjectTimelineEditItem = {
  id: string;
  at_utc: string | null;
  sort_order: number;
  fields: ApiEditFieldValue[];
};

type ApiProjectEditData = {
  key: string;
  category_id: string;
  start_utc?: string | null;
  end_utc?: string | null;
  status_type: "waiting" | "ongoing";
  locale: string;
  enabled_locales: string[];
  fields: ApiEditFieldValue[];
  timeline: ApiProjectTimelineEditItem[];
};

type ApiCreatedKeyResponse = { key: string };

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function useMockData(): boolean {
  return import.meta.env.MODE === "test" || import.meta.env.VITE_USE_MOCK_DATA === "true";
}

function localized(locale: LocaleCode, value: string): ProjectLocalizedText {
  return locale === "en" ? { en: value } : { fr: value };
}

function toTimelineEntry(locale: LocaleCode, item: ApiProjectTimelineItem): ProjectTimelineEntry {
  return {
    id: item.id,
    atUtc: item.at_utc,
    title: localized(locale, item.title ?? ""),
    details: localized(locale, item.details ?? ""),
    createdBy: mapUserRef(item.created_by),
    lastModifiedBy: mapUserRef(item.last_modified_by),
  };
}

function toProjectItem(locale: LocaleCode, value: ApiProjectListItem | ApiProjectDetail): ProjectItem {
  return {
    id: value.key,
    categoryCode: value.category_id,
    category: value.category,
    title: localized(locale, value.title ?? ""),
    description: localized(locale, value.description ?? ""),
    startUtc: value.start_utc ?? undefined,
    endUtc: value.end_utc ?? undefined,
    statusType: value.status_type,
    statusText: localized(locale, value.status_text ?? ""),
    timeline: "timeline" in value ? (value.timeline ?? []).map((item) => toTimelineEntry(locale, item)) : [],
    attachments: [],
    lastModifiedAt: "last_modified_at" in value ? (value.last_modified_at ?? undefined) : undefined,
    lastModifiedBy: "last_modified_by" in value ? mapUserRef(value.last_modified_by) : undefined,
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

function toTimelineEditItem(value: ApiProjectTimelineEditItem): ProjectTimelineEditItem {
  return {
    id: value.id,
    atUtc: value.at_utc,
    sortOrder: value.sort_order,
    fields: value.fields.map(toEditField),
  };
}

function toEditData(value: ApiProjectEditData): ProjectEditData {
  return {
    id: value.key,
    categoryId: value.category_id,
    startUtc: value.start_utc ?? undefined,
    endUtc: value.end_utc ?? undefined,
    statusType: value.status_type,
    locale: value.locale,
    enabledLocales: value.enabled_locales,
    fields: value.fields.map(toEditField),
    timeline: value.timeline.map(toTimelineEditItem),
  };
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await authenticatedFetch(url);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function fetchJsonOrNull<T>(url: string): Promise<T | null> {
  const response = await authenticatedFetch(url);
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function sendJson<T>(url: string, method: string, body?: unknown): Promise<T | null> {
  const response = await authenticatedFetch(url, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return response.status === 204 ? null : ((await response.json()) as T);
}

function toApiPayload(payload: ProjectSavePayload, existingId?: string): Record<string, unknown> {
  return {
    ...(existingId ? { key: existingId } : {}),
    category_id: payload.categoryId,
    start_utc: payload.startUtc ?? null,
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

export class ApiProjectsRepository implements ProjectsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<ProjectItem[]> {
    if (useMockData()) {
      return mockProjectsRepository.list(preferredLanguage, query);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    if (query.trim()) {
      params.set("q", query.trim());
    }
    const payload = await fetchJson<ApiProjectListItem[]>(`${apiBaseUrl()}/projects?${params.toString()}`);
    return payload.map((value) => toProjectItem(preferredLanguage, value));
  }

  async byId(id: string, preferredLanguage: LocaleCode): Promise<ProjectItem | null> {
    if (useMockData()) {
      return mockProjectsRepository.byId(id, preferredLanguage);
    }

    const params = new URLSearchParams({ locale: preferredLanguage });
    const payload = await fetchJsonOrNull<ApiProjectDetail>(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}?${params.toString()}`);
    return payload ? toProjectItem(preferredLanguage, payload) : null;
  }

  async editData(id: string, preferredLanguage: LocaleCode): Promise<ProjectEditData | null> {
    if (useMockData()) {
      const item = await mockProjectsRepository.byId(id, preferredLanguage);
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
          { fieldKey: "status_text", value: item.statusText[preferredLanguage] ?? "" },
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
    const payload = await fetchJsonOrNull<ApiProjectEditData>(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}/edit?${params.toString()}`);
    return payload ? toEditData(payload) : null;
  }

  async save(payload: ProjectSavePayload, existingId?: string): Promise<string | void> {
    if (useMockData()) return;
    const path = existingId ? `/projects/${encodeURIComponent(existingId)}` : "/projects";
    const response = await sendJson<ApiCreatedKeyResponse>(`${apiBaseUrl()}${path}`, existingId ? "PUT" : "POST", toApiPayload(payload, existingId));
    return response?.key;
  }

  async delete(id: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}`, "DELETE");
  }

  async createTimelineEntry(id: string, preferredLanguage: LocaleCode, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<ApiProjectTimelineItem> {
    if (useMockData()) return { id: crypto.randomUUID(), at_utc: payload.atUtc, title: payload.fields.title ?? "", details: payload.fields.details ?? "" };
    const params = new URLSearchParams({ locale: preferredLanguage });
    const result = await sendJson<ApiProjectTimelineItem>(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}/timeline?${params.toString()}`, "POST", {
      at_utc: payload.atUtc,
      sort_order: payload.sortOrder,
      fields: payload.fields,
    });
    if (!result) throw new Error("failed to create timeline entry");
    return result;
  }

  async updateTimelineEntry(id: string, entryId: string, preferredLanguage: LocaleCode, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): Promise<ApiProjectTimelineItem> {
    if (useMockData()) return { id: entryId, at_utc: payload.atUtc, title: payload.fields.title ?? "", details: payload.fields.details ?? "" };
    const params = new URLSearchParams({ locale: preferredLanguage });
    const result = await sendJson<ApiProjectTimelineItem>(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}/timeline/${encodeURIComponent(entryId)}?${params.toString()}`, "PUT", {
      at_utc: payload.atUtc,
      sort_order: payload.sortOrder,
      fields: payload.fields,
    });
    if (!result) throw new Error("failed to update timeline entry");
    return result;
  }

  async deleteTimelineEntry(id: string, entryId: string): Promise<void> {
    if (useMockData()) return;
    await sendJson(`${apiBaseUrl()}/projects/${encodeURIComponent(id)}/timeline/${encodeURIComponent(entryId)}`, "DELETE");
  }
}

export const apiProjectsRepository = new ApiProjectsRepository();
