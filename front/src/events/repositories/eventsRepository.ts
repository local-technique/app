import type { LocaleCode } from "../../common/localeContent";
import type { EventEditData, EventItem, EventSavePayload } from "../types";

export interface EventsRepository {
  list(preferredLanguage: LocaleCode, query: string): Promise<EventItem[]>;
  byId(id: string, preferredLanguage: LocaleCode): Promise<EventItem | null>;
  editData(id: string, preferredLanguage: LocaleCode): Promise<EventEditData | null>;
  save(payload: EventSavePayload, existingId?: string): Promise<void>;
  delete(id: string): Promise<void>;
}
