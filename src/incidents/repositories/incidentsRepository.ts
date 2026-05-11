import type { LocaleCode } from "../../common/localeContent";
import type { IncidentItem } from "../types";

export interface IncidentsRepository {
  list(preferredLanguage: LocaleCode, query: string): Promise<IncidentItem[]>;
  byId(id: string, preferredLanguage: LocaleCode): Promise<IncidentItem | null>;
}
