import type { LocaleCode } from "../../common/localeContent";
import { matchesEventQuery } from "../utils";
import { MOCK_EVENTS } from "../data/mockEvents";
import type { EventItem } from "../types";
import type { EventsRepository } from "./eventsRepository";

export class MockEventsRepository implements EventsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<EventItem[]> {
    const filtered = MOCK_EVENTS.filter((event) => matchesEventQuery(event, query, preferredLanguage));
    return [...filtered].sort((a, b) => Date.parse(a.startUtc) - Date.parse(b.startUtc));
  }

  async byId(id: string, _preferredLanguage: LocaleCode): Promise<EventItem | null> {
    return MOCK_EVENTS.find((event) => event.id === id) ?? null;
  }
}

export const mockEventsRepository = new MockEventsRepository();
