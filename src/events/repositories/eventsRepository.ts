import type { LocaleCode } from "../../common/localeContent";
import type { EventItem } from "../types";

export interface EventsRepository {
  list(preferredLanguage: LocaleCode, query: string): Promise<EventItem[]>;
  byId(id: string, preferredLanguage: LocaleCode): Promise<EventItem | null>;
}
