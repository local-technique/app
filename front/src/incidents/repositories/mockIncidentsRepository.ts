import type { LocaleCode } from "../../common/localeContent";
import { MOCK_INCIDENTS } from "../data/mockIncidents";
import type { IncidentEditData, IncidentItem, IncidentSavePayload } from "../types";
import { matchesIncidentQuery } from "../utils";
import type { IncidentsRepository } from "./incidentsRepository";

export class MockIncidentsRepository implements IncidentsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<IncidentItem[]> {
    const filtered = MOCK_INCIDENTS.filter((incident) =>
      matchesIncidentQuery(incident, query, preferredLanguage),
    );
    return [...filtered].sort((a, b) => Date.parse(a.startUtc) - Date.parse(b.startUtc));
  }

  async byId(id: string, _preferredLanguage: LocaleCode): Promise<IncidentItem | null> {
    return MOCK_INCIDENTS.find((incident) => incident.id === id) ?? null;
  }

  async editData(id: string, preferredLanguage: LocaleCode): Promise<IncidentEditData | null> {
    const item = await this.byId(id, preferredLanguage);
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
        { fieldKey: "short_description", value: item.shortDescription[preferredLanguage] ?? "" },
        { fieldKey: "long_description", value: item.longDescription[preferredLanguage] ?? "" },
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

  async save(_payload: IncidentSavePayload, existingId?: string): Promise<string | void> { return existingId ?? "INC-10"; }

  async delete(_id: string): Promise<void> {}
}

export const mockIncidentsRepository = new MockIncidentsRepository();
