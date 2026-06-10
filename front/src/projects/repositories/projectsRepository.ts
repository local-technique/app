import type { LocaleCode } from "../../common/localeContent";
import type { ProjectEditData, ProjectItem, ProjectSavePayload } from "../types";

export interface ProjectsRepository {
  list(preferredLanguage: LocaleCode, query: string): Promise<ProjectItem[]>;
  byId(id: string, preferredLanguage: LocaleCode): Promise<ProjectItem | null>;
  editData(id: string, preferredLanguage: LocaleCode): Promise<ProjectEditData | null>;
  save(payload: ProjectSavePayload, existingId?: string): Promise<string | void>;
  delete(id: string): Promise<void>;
}
