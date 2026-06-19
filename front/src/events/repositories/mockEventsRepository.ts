import type { LocaleCode } from "../../common/localeContent";
import { matchesEventQuery } from "../utils";
import { MOCK_EVENTS } from "../data/mockEvents";
import type { EventEditData, EventItem, EventSavePayload } from "../types";
import type { EventsRepository } from "./eventsRepository";

export class MockEventsRepository implements EventsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<EventItem[]> {
    const filtered = MOCK_EVENTS.filter((event) => matchesEventQuery(event, query, preferredLanguage));
    return [...filtered].sort((a, b) => Date.parse(a.startUtc) - Date.parse(b.startUtc));
  }

  async byId(id: string, _preferredLanguage: LocaleCode): Promise<EventItem | null> {
    return MOCK_EVENTS.find((event) => event.id === id) ?? null;
  }

  async editData(id: string, preferredLanguage: LocaleCode): Promise<EventEditData | null> {
    const item = await this.byId(id, preferredLanguage);
    if (!item) return null;
    return {
      id: item.id,
      categoryId: item.categoryCode,
      startUtc: item.startUtc,
      endUtc: item.endUtc,
      notifiedAtUtc: item.notifiedAtUtc,
      statusType: item.statusType,
      locale: preferredLanguage,
      enabledLocales: ["en", "fr"],
      fields: [
        { fieldKey: "title", value: item.title[preferredLanguage] ?? "" },
        { fieldKey: "short_description", value: item.shortDescription[preferredLanguage] ?? "" },
        { fieldKey: "long_description", value: item.longDescription[preferredLanguage] ?? "" },
        { fieldKey: "warning", value: item.warning?.[preferredLanguage] ?? "" },
        { fieldKey: "location", value: item.location?.[preferredLanguage] ?? "" },
        { fieldKey: "status_text", value: item.statusText?.[preferredLanguage] ?? "" },
      ],
      timeline: [],
    };
  }

  async save(_payload: EventSavePayload, existingId?: string): Promise<string | void> { return existingId ?? "EVT-10"; }

  async delete(_id: string): Promise<void> {}
}

export const mockEventsRepository = new MockEventsRepository();
