import { formatLocalDate, formatLocalDateTime, parseUtc } from "../common/date";
import type { LocaleCode } from "../common/localeContent";
import { resolveLocalized } from "../common/localeContent";
import { fuzzyMatch } from "../common/search";
import { computeDisplayStatus, computeTimeStatus } from "../common/timeStatus";
import type {
  ProjectDisplayStatus,
  ProjectItem,
  ProjectLocalizedText,
  ProjectStatusSection,
  ProjectTimelineEntry,
} from "./types";

export type ProjectTimelineEntryViewModel = {
  id: string;
  atUtc: string | null;
  atLabel: string;
  atDateLabel: string;
  atTimeLabel: string;
  isPending: boolean;
  title: string;
  details: string;
  createdBy?: { initials: string; fullName: string; id: string } | null;
  lastModifiedBy?: { initials: string; fullName: string } | null;
};

export type ProjectViewModel = {
  id: string;
  section: ProjectStatusSection;
  status: ProjectStatusSection;
  statusType: ProjectDisplayStatus;
  statusText: string;
  title: string;
  description: string;
  dateLabel: string;
  startDateFormatted?: string;
  endDateFormatted?: string;
  timeline: ProjectTimelineEntryViewModel[];
  raw: ProjectItem;
};

function resolve(value: ProjectLocalizedText | undefined, locale: LocaleCode): string {
  if (!value) {
    return "";
  }
  return resolveLocalized(value, locale);
}

function classifyProject(project: ProjectItem, now = new Date()): { section: ProjectStatusSection; statusType: ProjectDisplayStatus } {
  const timeStatus = computeTimeStatus(project.startUtc, project.endUtc, now);

  if (timeStatus === "PAST") {
    return { section: "finished", statusType: "finished" };
  }

  if (project.statusType === "waiting") {
    return { section: "toCome", statusType: "waiting" };
  }

  return {
    section: "ongoing",
    statusType: computeDisplayStatus("ongoing", timeStatus),
  };
}

function formatProjectDateLabel(project: ProjectItem, locale: LocaleCode): string {
  if (!project.startUtc && !project.endUtc) {
    return locale === "fr" ? "Dates à confirmer" : "Dates to be confirmed";
  }
  if (project.startUtc && project.endUtc) {
    return `${formatLocalDate(parseUtc(project.startUtc), locale)} - ${formatLocalDate(parseUtc(project.endUtc), locale)}`;
  }
  if (project.startUtc) {
    return formatLocalDate(parseUtc(project.startUtc), locale);
  }
  const end = formatLocalDate(parseUtc(project.endUtc ?? ""), locale);
  return locale === "fr" ? `jusqu'au ${end}` : `until ${end}`;
}

function toTimelineEntryViewModel(entry: ProjectTimelineEntry, locale: LocaleCode): ProjectTimelineEntryViewModel {
  const atDate = entry.atUtc ? parseUtc(entry.atUtc) : null;

  function toUserDisplay(user: { id: string; email: string; firstName?: string | null; lastName?: string | null } | null | undefined): { initials: string; fullName: string; id: string } | null {
    if (!user) return null;
    const firstChar = user.firstName?.[0] ?? user.lastName?.[0] ?? user.email[0] ?? '';
    const lastChar = user.firstName && user.lastName ? user.lastName[0] : null;
    const initials = firstChar && lastChar ? `${firstChar}${lastChar}`.toUpperCase() : firstChar.toUpperCase();
    const fullName = user.firstName && user.lastName ? `${user.firstName} ${user.lastName}` : (user.firstName ?? user.lastName ?? user.email ?? '');
    return { initials, fullName, id: user.id };
  }

  const displayCreatedBy = toUserDisplay(entry.createdBy);
  const displayLastModifiedBy = toUserDisplay(entry.lastModifiedBy);
  const differentModifier = displayLastModifiedBy && (!displayCreatedBy || displayLastModifiedBy.id !== displayCreatedBy.id)
    ? { initials: displayLastModifiedBy.initials, fullName: displayLastModifiedBy.fullName }
    : null;

  return {
    id: entry.id,
    atUtc: entry.atUtc,
    atLabel: atDate ? formatLocalDateTime(atDate, locale) : "Pending",
    atDateLabel: atDate ? new Intl.DateTimeFormat(locale, { dateStyle: "medium" }).format(atDate) : "",
    atTimeLabel: atDate ? new Intl.DateTimeFormat(locale, { timeStyle: "short" }).format(atDate) : "",
    isPending: !entry.atUtc,
    title: resolve(entry.title, locale),
    details: resolve(entry.details, locale),
    createdBy: displayCreatedBy ? { initials: displayCreatedBy.initials, fullName: displayCreatedBy.fullName, id: displayCreatedBy.id } : null,
    lastModifiedBy: differentModifier,
  };
}

export function toProjectViewModel(project: ProjectItem, locale: LocaleCode): ProjectViewModel {
  const classification = classifyProject(project);
  const timeline = project.timeline.map((entry) => toTimelineEntryViewModel(entry, locale));
  return {
    id: project.id,
    section: classification.section,
    status: classification.section,
    statusType: classification.statusType,
    statusText: classification.statusType === "finished" ? (locale === "fr" ? "Terminé" : "Finished") : resolve(project.statusText, locale),
    title: resolve(project.title, locale),
    description: resolve(project.description, locale),
    dateLabel: formatProjectDateLabel(project, locale),
    startDateFormatted: project.startUtc ? formatLocalDate(parseUtc(project.startUtc), locale) : undefined,
    endDateFormatted: project.endUtc ? formatLocalDate(parseUtc(project.endUtc), locale) : undefined,
    timeline,
    raw: project,
  };
}

export function groupByStatus(projects: ProjectViewModel[]): Record<ProjectStatusSection, ProjectViewModel[]> {
  return projects.reduce<Record<ProjectStatusSection, ProjectViewModel[]>>(
    (groups, item) => {
      groups[item.section].push(item);
      return groups;
    },
    { ongoing: [], toCome: [], finished: [] },
  );
}

export function matchesProjectQuery(project: ProjectItem, query: string, locale: LocaleCode): boolean {
  if (!query.trim()) {
    return true;
  }
  const model = toProjectViewModel(project, locale);
  const timelineText = project.timeline
    .map((entry) => `${resolve(entry.title, locale)} ${resolve(entry.details, locale)}`)
    .join(" ");
  const haystack = [
    project.id,
    resolve(project.title, locale),
    resolve(project.description, locale),
    project.categoryCode,
    project.category?.key,
    project.category?.label,
    project.statusType,
    model.statusType,
    model.statusText,
    timelineText,
  ]
    .join(" ")
    .trim();
  return fuzzyMatch(query, haystack);
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function isBulletLine(line: string): boolean {
  const trimmed = line.trim();
  return trimmed.startsWith("- ") || trimmed.startsWith("* ");
}

function bulletContent(line: string): string {
  return line.trim().slice(2);
}

function renderInlineMarkdown(value: string): string {
  return escapeHtml(value)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/!\[([^\]]*)]\((https?:\/\/[^\s)]+)\)/g, '<img src="$2" alt="$1">')
    .replace(/\[([^\]]+)]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" rel="noopener noreferrer" target="_blank">$1</a>')
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\*([^*]+)\*/g, "<em>$1</em>")
    .replace(/_([^_]+)_/g, "<em>$1</em>")
    .replace(/~~([^~]+)~~/g, "<del>$1</del>")
    .replace(/==([^=]+)==/g, "<mark>$1</mark>")
    .replace(/(?<![~])~([^~]+)~(?!~)/g, "<sub>$1</sub>")
    .replace(/\^([^^]+)\^/g, "<sup>$1</sup>")
    .replace(/(https?:\/\/[^\s<>"')]+)/g, (match: string, url: string, offset: number, full: string) => {
      const before = full.slice(0, offset);
      if (before.lastIndexOf("<") > before.lastIndexOf(">")) {
        return match;
      }
      return `<a href="${match}" rel="noopener noreferrer" target="_blank">${match}</a>`;
    });
}

function splitTableRow(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}

function isTableSeparator(line: string): boolean {
  const cells = splitTableRow(line);
  return cells.length > 0 && cells.every((cell) => /^:?-{3,}:?$/.test(cell));
}

function tableAlignClass(separator: string): string {
  if (separator.startsWith(":") && separator.endsWith(":")) {
    return ' class="align-center"';
  }
  if (separator.endsWith(":")) {
    return ' class="align-right"';
  }
  return "";
}

function renderTable(lines: string[]): string | null {
  if (lines.length < 2 || !isTableSeparator(lines[1])) {
    return null;
  }

  const headers = splitTableRow(lines[0]);
  const separators = splitTableRow(lines[1]);
  if (headers.length === 0 || headers.length !== separators.length) {
    return null;
  }

  const head = headers
    .map((header, index) => `<th${tableAlignClass(separators[index])}>${renderInlineMarkdown(header)}</th>`)
    .join("");
  const body = lines
    .slice(2)
    .filter((line) => line.trim().startsWith("|"))
    .map((line) => {
      const cells = splitTableRow(line);
      const renderedCells = headers
        .map((_, index) => `<td${tableAlignClass(separators[index])}>${renderInlineMarkdown(cells[index] ?? "")}</td>`)
        .join("");
      return `<tr>${renderedCells}</tr>`;
    })
    .join("");

  return `<table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`;
}

function isOrderedLine(line: string): boolean {
  return /^\d+\.\s/.test(line.trim());
}

function orderedStart(line: string): number {
  const match = line.trim().match(/^\d+/);
  return match ? parseInt(match[0], 10) : 1;
}

function orderedContent(line: string): string {
  return line.trim().replace(/^\d+\.\s*/, "");
}

function isBlockquoteLine(line: string): boolean {
  return line.trim().startsWith("> ");
}

interface TaskItem {
  checked: boolean;
  content: string;
}

function parseTaskItem(content: string): TaskItem | null {
  const trimmed = content.trim();
  const match = trimmed.match(/^\[([ xX])\]\s+(.*)$/);
  if (!match) return null;
  return { checked: match[1] !== " ", content: match[2] };
}

function renderListItem(content: string): string {
  const task = parseTaskItem(content);
  if (task) {
    return `<li>${task.checked ? '<input type="checkbox" disabled checked>' : '<input type="checkbox" disabled>'} ${renderInlineMarkdown(task.content)}</li>`;
  }
  return `<li>${renderInlineMarkdown(content)}</li>`;
}

function extractFenceBlocks(text: string, fence: string, codeBlocks: string[]): string {
  const pattern = new RegExp(
    `${fence}(\\w*)\\n([\\s\\S]*?)${fence}\\s*`,
    "g",
  );
  return text.replace(pattern, (_match: string, lang: string, code: string) => {
    const index = codeBlocks.length;
    const langAttr = lang ? ` class="language-${lang}"` : "";
    codeBlocks.push(`<pre><code${langAttr}>${escapeHtml(code.trimEnd())}</code></pre>`);
    return `\n\n__CODEBLOCK_${index}__\n\n`;
  });
}

export function renderProjectMarkdown(markdown: string): string {
  const normalized = markdown.replace(/\r\n/g, "\n");

  const codeBlocks: string[] = [];
  let processed = extractFenceBlocks(normalized, "```", codeBlocks);
  processed = extractFenceBlocks(processed, "~~~", codeBlocks);

  const blocks = processed
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean);

  return blocks
    .map((block) => {
      const codeMatch = block.match(/^__CODEBLOCK_(\d+)__$/);
      if (codeMatch) {
        return codeBlocks[parseInt(codeMatch[1], 10)];
      }

      const lines = block.split("\n");
      const heading = block.match(/^(#{1,6})\s+(.+)$/);
      if (heading) {
        const level = heading[1].length;
        return `<h${level}>${renderInlineMarkdown(heading[2])}</h${level}>`;
      }
      if (/^(?:[-*_]){3,}\s*$/.test(block.trim())) {
        return "<hr>";
      }
      if (lines.every((line) => isBlockquoteLine(line) || line.trim() === "")) {
        const content = lines
          .filter((line) => line.trim() !== "")
          .map((line) => renderInlineMarkdown(line.trim().replace(/^>\s*/, "")))
          .join("<br>");
        return `<blockquote><p>${content}</p></blockquote>`;
      }
      const table = renderTable(lines);
      if (table) {
        return table;
      }
      if (lines.some((line) => isBulletLine(line) || isOrderedLine(line))) {
        let result = "";
        let i = 0;
        while (i < lines.length) {
          if (isBulletLine(lines[i])) {
            const items: string[] = [];
            while (i < lines.length && isBulletLine(lines[i])) {
              items.push(renderListItem(bulletContent(lines[i])));
              i++;
            }
            result += `<ul>${items.join("")}</ul>`;
          } else if (isOrderedLine(lines[i])) {
            const items: string[] = [];
            const start = orderedStart(lines[i]);
            const openTag = start !== 1 ? `<ol start="${start}">` : "<ol>";
            while (i < lines.length && isOrderedLine(lines[i])) {
              items.push(renderListItem(orderedContent(lines[i])));
              i++;
            }
            result += `${openTag}${items.join("")}</ol>`;
          } else {
            const paraLines: string[] = [];
            while (i < lines.length && !isBulletLine(lines[i]) && !isOrderedLine(lines[i])) {
              paraLines.push(renderInlineMarkdown(lines[i].replace(/^#+\s*/, "")));
              i++;
            }
            result += `<p>${paraLines.join("<br>")}</p>`;
          }
        }
        return result;
      }
      const paragraph = lines.map((line) => renderInlineMarkdown(line.replace(/^#+\s*/, ""))).join("<br>");
      return `<p>${paragraph}</p>`;
    })
    .join("");
}
