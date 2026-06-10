import type { LocaleCode } from "../../common/localeContent";
import { MOCK_PROJECTS } from "../data/mockProjects";
import type { ProjectEditData, ProjectItem, ProjectSavePayload } from "../types";
import { matchesProjectQuery } from "../utils";
import type { ProjectsRepository } from "./projectsRepository";

export class MockProjectsRepository implements ProjectsRepository {
  async list(preferredLanguage: LocaleCode, query: string): Promise<ProjectItem[]> {
    return MOCK_PROJECTS.filter((project) => matchesProjectQuery(project, query, preferredLanguage));
  }

  async byId(id: string, _preferredLanguage: LocaleCode): Promise<ProjectItem | null> {
    return MOCK_PROJECTS.find((project) => project.id === id) ?? null;
  }

  async editData(id: string, preferredLanguage: LocaleCode): Promise<ProjectEditData | null> {
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
        { fieldKey: "description", value: item.description[preferredLanguage] ?? "" },
        { fieldKey: "status_text", value: item.statusText[preferredLanguage] ?? "" },
      ],
    };
  }

  async save(_payload: ProjectSavePayload, existingId?: string): Promise<string | void> { return existingId ?? "PRJ-10"; }

  async delete(_id: string): Promise<void> {}
}

export const mockProjectsRepository = new MockProjectsRepository();
