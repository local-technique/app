import type { LocaleCode } from "../../common/localeContent";
import type { IncidentEditData, IncidentItem, IncidentSavePayload } from "../types";

export interface IncidentsRepository {
  list(preferredLanguage: LocaleCode, query: string): Promise<IncidentItem[]>;
  byId(id: string, preferredLanguage: LocaleCode): Promise<IncidentItem | null>;
  editData(id: string, preferredLanguage: LocaleCode): Promise<IncidentEditData | null>;
  save(payload: IncidentSavePayload, existingId?: string): Promise<string | void>;
  delete(id: string): Promise<void>;
}
