import type { LocaleCode } from "../../common/localeContent";
import { MOCK_INCIDENTS } from "../data/mockIncidents";
import type { IncidentItem } from "../types";
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
}

export const mockIncidentsRepository = new MockIncidentsRepository();
