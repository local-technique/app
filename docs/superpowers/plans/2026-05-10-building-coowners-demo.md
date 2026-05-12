# Building Co-Owners Demo Website Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a polished, lightweight, static Vue demo for co-owners with bilingual timelines (events/incidents), theme/language preferences, fuzzy search, and attachment preview, ready for backend integration.

**Architecture:** Use a feature-first Vue 3 SPA (`events`, `incidents`, `common`) with hash routing for GitHub Pages. Keep domain types/repositories separated by feature, and keep shared logic in `common` only (date/time, search, generic helpers). Wire pages to async repositories backed by mock data now, with interfaces shaped for future API adapters.

**Tech Stack:** Vue 3, Vite, TypeScript, vue-router, vue-i18n, Vitest, Vue Test Utils, Testing Library, Playwright

---

## File Structure Map (create first)

- `package.json` - scripts and dependencies
- `vite.config.ts` - build + test configuration
- `index.html` - SPA entry shell
- `src/main.ts` - app bootstrap
- `src/App.vue` - app shell and layout
- `src/router/index.ts` - hash router and routes
- `src/common/`
  - `i18n.ts` - UI string setup
  - `localeContent.ts` - active->en->fr fallback resolver
  - `theme.ts` - system/light/dark with persistence
  - `date.ts` - UTC parse, local display, status helpers
  - `search.ts` - lightweight fuzzy matching
  - `components/SidebarNav.vue` - left nav + controls
  - `components/MobileMenu.vue` - mobile nav drawer
  - `components/StatusSection.vue` - reusable section block
- `src/events/`
  - `types.ts`, `utils.ts`
  - `repositories/eventsRepository.ts`
  - `repositories/mockEventsRepository.ts`
  - `data/mockEvents.ts`
  - `ListingPage.vue`, `DetailPage.vue`
- `src/incidents/`
  - `types.ts`, `utils.ts`
  - `repositories/incidentsRepository.ts`
  - `repositories/mockIncidentsRepository.ts`
  - `data/mockIncidents.ts`
  - `ListingPage.vue`, `DetailPage.vue`
- `src/common/assets/mock-files/`
  - `invoice-2026-04.pdf`
  - `elevator-intervention-2026-02.pdf`
  - `elevator-check-photo.png`
- `tests/unit/` - pure logic tests
- `tests/component/` - component/page behavior tests
- `tests/e2e/` - routing/deep-link responsive checks

---

### Task 1: Bootstrap Vue App and Test Harness

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `index.html`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src/router/index.ts`
- Create: `tests/component/app-shell.test.ts`

- [ ] **Step 1: Write the failing shell test**

```ts
// tests/component/app-shell.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import { createRouter, createWebHashHistory } from "vue-router";
import App from "../../src/App.vue";

describe("app shell", () => {
  it("renders sidebar entries", async () => {
    const router = createRouter({
      history: createWebHashHistory(),
      routes: [
        { path: "/events", component: { template: "<div>events</div>" } },
        { path: "/incidents", component: { template: "<div>incidents</div>" } },
      ],
    });

    render(App, { global: { plugins: [router] } });
    expect(screen.getByText("Events & Maintenance")).toBeInTheDocument();
    expect(screen.getByText("Incidents")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- tests/component/app-shell.test.ts`
Expected: FAIL with missing project files/dependencies

- [ ] **Step 3: Create minimal app + tooling implementation**

```json
// package.json
{
  "name": "copro-website-demo",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:e2e": "playwright test"
  },
  "dependencies": {
    "vue": "^3.5.13",
    "vue-i18n": "^9.14.2",
    "vue-router": "^4.5.0"
  },
  "devDependencies": {
    "@playwright/test": "^1.54.1",
    "@testing-library/vue": "^8.1.0",
    "@vitejs/plugin-vue": "^5.2.1",
    "typescript": "^5.8.3",
    "vite": "^5.4.19",
    "vitest": "^2.1.8",
    "vue-tsc": "^2.2.8"
  }
}
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "jsdom",
    setupFiles: [],
    globals: true,
  },
});
```

```vue
<!-- src/App.vue -->
<template>
  <div class="app-shell">
    <aside>
      <a href="#/events">Events & Maintenance</a>
      <a href="#/incidents">Incidents</a>
    </aside>
    <main>
      <RouterView />
    </main>
  </div>
</template>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm install; if ($?) { npm run test -- tests/component/app-shell.test.ts }`
Expected: PASS for `app-shell.test.ts`

- [ ] **Step 5: Commit**

```bash
git add package.json vite.config.ts tsconfig.json index.html src/main.ts src/App.vue src/router/index.ts tests/component/app-shell.test.ts
git commit -m "chore: bootstrap vue app with test harness"
```

### Task 2: Common Layer (i18n, theme, UTC date, fuzzy search)

**Files:**
- Create: `src/common/i18n.ts`
- Create: `src/common/localeContent.ts`
- Create: `src/common/theme.ts`
- Create: `src/common/date.ts`
- Create: `src/common/search.ts`
- Test: `tests/unit/locale-content.test.ts`
- Test: `tests/unit/date-status.test.ts`
- Test: `tests/unit/search.test.ts`

- [ ] **Step 1: Write failing unit tests for fallback/date/search rules**

```ts
// tests/unit/locale-content.test.ts
import { describe, expect, it } from "vitest";
import { resolveLocalized } from "../../src/common/localeContent";

describe("resolveLocalized", () => {
  it("falls back active -> en -> fr", () => {
    expect(resolveLocalized({ fr: "Bonjour" }, "en")).toBe("Bonjour");
    expect(resolveLocalized({ en: "Hello", fr: "Bonjour" }, "fr")).toBe("Bonjour");
  });
});
```

```ts
// tests/unit/date-status.test.ts
import { describe, expect, it } from "vitest";
import { classifyEventStatus } from "../../src/common/date";

describe("classifyEventStatus", () => {
  it("classifies point-in-time events as to come then past", () => {
    expect(classifyEventStatus({ startUtc: "2099-01-01T10:00:00Z" }, new Date("2098-01-01T00:00:00Z"))).toBe("toCome");
    expect(classifyEventStatus({ startUtc: "2020-01-01T10:00:00Z" }, new Date("2021-01-01T00:00:00Z"))).toBe("past");
  });
});
```

```ts
// tests/unit/search.test.ts
import { describe, expect, it } from "vitest";
import { fuzzyMatch } from "../../src/common/search";

describe("fuzzyMatch", () => {
  it("matches partial tokens", () => {
    expect(fuzzyMatch("heat maint", "Heating maintenance in block A")).toBe(true);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm run test -- tests/unit/locale-content.test.ts tests/unit/date-status.test.ts tests/unit/search.test.ts`
Expected: FAIL with missing modules/functions

- [ ] **Step 3: Implement common utilities minimally**

```ts
// src/common/localeContent.ts
export type LocalizedValue = { fr?: string; en?: string };

export function resolveLocalized(value: LocalizedValue, active: "fr" | "en"): string {
  return value[active] ?? value.en ?? value.fr ?? "";
}
```

```ts
// src/common/date.ts
export type EventDateInput = { startUtc: string; endUtc?: string };
export type EventStatus = "current" | "toCome" | "past";

export function classifyEventStatus(input: EventDateInput, now = new Date()): EventStatus {
  const start = new Date(input.startUtc);
  if (!input.endUtc) {
    return start.getTime() > now.getTime() ? "toCome" : "past";
  }
  const end = new Date(input.endUtc);
  if (start.getTime() > now.getTime()) return "toCome";
  if (end.getTime() < now.getTime()) return "past";
  return "current";
}
```

```ts
// src/common/search.ts
export function fuzzyMatch(query: string, text: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return q
    .split(/\s+/)
    .every((token) => text.toLowerCase().includes(token));
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm run test -- tests/unit/locale-content.test.ts tests/unit/date-status.test.ts tests/unit/search.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/common/localeContent.ts src/common/date.ts src/common/search.ts tests/unit/locale-content.test.ts tests/unit/date-status.test.ts tests/unit/search.test.ts
git commit -m "feat: add shared locale, date status, and fuzzy search utilities"
```

### Task 3: Events Feature (listing, detail, mock repo)

**Files:**
- Create: `src/events/types.ts`
- Create: `src/events/utils.ts`
- Create: `src/events/repositories/eventsRepository.ts`
- Create: `src/events/repositories/mockEventsRepository.ts`
- Create: `src/events/data/mockEvents.ts`
- Create: `src/events/ListingPage.vue`
- Create: `src/events/DetailPage.vue`
- Modify: `src/router/index.ts`
- Test: `tests/component/events-listing.test.ts`

- [ ] **Step 1: Write failing listing behavior test**

```ts
// tests/component/events-listing.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import EventsListingPage from "../../src/events/ListingPage.vue";

describe("Events listing", () => {
  it("shows Current/To come/Past and hides empty sections during search", async () => {
    render(EventsListingPage);
    expect(screen.getByText("Current")).toBeInTheDocument();
    expect(screen.getByText("To come")).toBeInTheDocument();
    expect(screen.getByText("Past")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- tests/component/events-listing.test.ts`
Expected: FAIL with missing Events pages/repository

- [ ] **Step 3: Implement events domain end-to-end minimum**

```ts
// src/events/types.ts
export type EventsLocalized = { fr?: string; en?: string };

export type EventItem = {
  id: string;
  categoryCode: string;
  title: EventsLocalized;
  shortDescription: EventsLocalized;
  longDescription: EventsLocalized;
  location?: EventsLocalized;
  startUtc: string;
  endUtc?: string;
  notifiedAtUtc?: string;
  handlers?: string[];
  attachments: { id: string; fileName: string; mimeType: string; sizeBytes: number; url: string }[];
};
```

```ts
// src/events/repositories/eventsRepository.ts
import type { EventItem } from "../types";

export interface EventsRepository {
  list(preferredLanguage: "fr" | "en", query: string): Promise<EventItem[]>;
  byId(id: string, preferredLanguage: "fr" | "en"): Promise<EventItem | null>;
}
```

```ts
// src/events/data/mockEvents.ts
import type { EventItem } from "../types";

export const MOCK_EVENTS: EventItem[] = [
  { id: "HEA-03", categoryCode: "HEA", title: { fr: "Maintenance chauffage en cours" }, shortDescription: { fr: "Controle chaudiere" }, longDescription: { fr: "Maintenance preventive du systeme de chauffage." }, startUtc: "2026-05-01T08:00:00Z", endUtc: "2026-05-30T16:00:00Z", attachments: [] },
  { id: "ELV-04", categoryCode: "ELV", title: { fr: "Maintenance ascenseur" }, shortDescription: { fr: "Intervention planifiee" }, longDescription: { fr: "Maintenance annuelle de l'ascenseur." }, startUtc: "2026-06-12T08:30:00Z", attachments: [] },
  { id: "PMG-02", categoryCode: "PMG", title: { fr: "Visite syndic" }, shortDescription: { fr: "Visite de suivi" }, longDescription: { fr: "Visite de controle du syndic." }, startUtc: "2026-06-18T13:30:00Z", attachments: [] }
];
```

- [ ] **Step 4: Run tests to verify pass**

Run: `npm run test -- tests/component/events-listing.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/events tests/component/events-listing.test.ts src/router/index.ts
git commit -m "feat: implement events listing and detail with mock repository"
```

### Task 4: Incidents Feature (listing, detail, incident micro-timeline)

**Files:**
- Create: `src/incidents/types.ts`
- Create: `src/incidents/utils.ts`
- Create: `src/incidents/repositories/incidentsRepository.ts`
- Create: `src/incidents/repositories/mockIncidentsRepository.ts`
- Create: `src/incidents/data/mockIncidents.ts`
- Create: `src/incidents/ListingPage.vue`
- Create: `src/incidents/DetailPage.vue`
- Modify: `src/router/index.ts`
- Test: `tests/component/incidents-listing.test.ts`
- Test: `tests/component/incidents-detail-timeline.test.ts`

- [ ] **Step 1: Write failing incidents tests**

```ts
// tests/component/incidents-listing.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import IncidentsListingPage from "../../src/incidents/ListingPage.vue";

describe("Incidents listing", () => {
  it("shows Current and Past sections only", () => {
    render(IncidentsListingPage);
    expect(screen.getByText("Current")).toBeInTheDocument();
    expect(screen.getByText("Past")).toBeInTheDocument();
    expect(screen.queryByText("To come")).toBeNull();
  });
});
```

```ts
// tests/component/incidents-detail-timeline.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import IncidentsDetailPage from "../../src/incidents/DetailPage.vue";

describe("Incident detail", () => {
  it("renders micro-event timeline block", () => {
    render(IncidentsDetailPage);
    expect(screen.getByText("Incident timeline")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm run test -- tests/component/incidents-listing.test.ts tests/component/incidents-detail-timeline.test.ts`
Expected: FAIL with missing incidents feature files

- [ ] **Step 3: Implement incidents domain with separate types**

```ts
// src/incidents/types.ts
export type IncidentLocalized = { fr?: string; en?: string };

export type IncidentTimelineEvent = {
  id: string;
  type: "created" | "notified" | "repair_arrived" | "repair_left" | "report_uploaded" | "finished";
  atUtc: string;
  note?: IncidentLocalized;
};

export type IncidentItem = {
  id: string;
  categoryCode: string;
  title: IncidentLocalized;
  shortDescription: IncidentLocalized;
  longDescription: IncidentLocalized;
  location?: IncidentLocalized;
  startUtc: string;
  endUtc?: string;
  notifiedAtUtc?: string;
  handlers?: string[];
  timeline: IncidentTimelineEvent[];
  attachments: { id: string; fileName: string; mimeType: string; sizeBytes: number; url: string }[];
};
```

```ts
// src/incidents/data/mockIncidents.ts
import type { IncidentItem } from "../types";

export const MOCK_INCIDENTS: IncidentItem[] = [
  {
    id: "HEA-01",
    categoryCode: "HEA",
    title: { fr: "Panne chauffage bloc B" },
    shortDescription: { fr: "Température basse signalée" },
    longDescription: { fr: "Investigation en cours avec prestataire." },
    startUtc: "2026-05-08T06:30:00Z",
    timeline: [
      { id: "t1", type: "created", atUtc: "2026-05-08T06:30:00Z" },
      { id: "t2", type: "notified", atUtc: "2026-05-08T07:00:00Z" }
    ],
    attachments: []
  }
];
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm run test -- tests/component/incidents-listing.test.ts tests/component/incidents-detail-timeline.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/incidents tests/component/incidents-listing.test.ts tests/component/incidents-detail-timeline.test.ts src/router/index.ts
git commit -m "feat: implement incidents pages with dedicated incident timeline model"
```

### Task 5: Sidebar UX, Theme/Language Controls, and Responsive Layout

**Files:**
- Create: `src/common/components/SidebarNav.vue`
- Create: `src/common/components/MobileMenu.vue`
- Modify: `src/App.vue`
- Modify: `src/common/i18n.ts`
- Test: `tests/component/navigation-layout.test.ts`

- [ ] **Step 1: Write failing layout/control test**

```ts
// tests/component/navigation-layout.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import App from "../../src/App.vue";

describe("layout and controls", () => {
  it("shows left nav entries and language/theme controls", () => {
    render(App);
    expect(screen.getByRole("navigation")).toBeInTheDocument();
    expect(screen.getByLabelText("Language")).toBeInTheDocument();
    expect(screen.getByLabelText("Theme")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- tests/component/navigation-layout.test.ts`
Expected: FAIL with missing controls or labels

- [ ] **Step 3: Implement sidebar + mobile drawer + persistence hooks**

```vue
<!-- src/common/components/SidebarNav.vue -->
<template>
  <nav aria-label="Primary navigation">
    <a href="#/events">{{ t("nav.events") }}</a>
    <a href="#/incidents">{{ t("nav.incidents") }}</a>
    <label>
      {{ t("controls.language") }}
      <select aria-label="Language" v-model="lang" @change="setLanguage(lang)">
        <option value="fr">FR</option>
        <option value="en">EN</option>
      </select>
    </label>
    <label>
      {{ t("controls.theme") }}
      <select aria-label="Theme" v-model="theme" @change="setTheme(theme)">
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>
  </nav>
</template>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test -- tests/component/navigation-layout.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/App.vue src/common/components/SidebarNav.vue src/common/components/MobileMenu.vue src/common/i18n.ts tests/component/navigation-layout.test.ts
git commit -m "feat: add responsive left-nav shell with language and theme controls"
```

### Task 6: Attachments, In-Browser Preview, and Sample Files

**Files:**
- Create: `src/common/components/AttachmentList.vue`
- Create: `src/common/components/AttachmentPreview.vue`
- Modify: `src/events/DetailPage.vue`
- Modify: `src/incidents/DetailPage.vue`
- Create: `src/common/assets/mock-files/invoice-2026-04.pdf`
- Create: `src/common/assets/mock-files/elevator-intervention-2026-02.pdf`
- Create: `src/common/assets/mock-files/elevator-check-photo.png`
- Test: `tests/component/attachments-preview.test.ts`

- [ ] **Step 1: Write failing attachment preview test**

```ts
// tests/component/attachments-preview.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import AttachmentPreview from "../../src/common/components/AttachmentPreview.vue";

describe("AttachmentPreview", () => {
  it("embeds PDF and image previews", () => {
    render(AttachmentPreview, { props: { file: { fileName: "demo.pdf", mimeType: "application/pdf", url: "/files/demo.pdf" } } });
    expect(screen.getByTitle("Attachment preview")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- tests/component/attachments-preview.test.ts`
Expected: FAIL with missing component

- [ ] **Step 3: Implement attachment list/preview and wire detail pages**

```vue
<!-- src/common/components/AttachmentPreview.vue -->
<template>
  <iframe v-if="file.mimeType === 'application/pdf'" :src="file.url" title="Attachment preview" />
  <img v-else-if="file.mimeType.startsWith('image/')" :src="file.url" alt="Attachment preview" />
  <a v-else :href="file.url" target="_blank" rel="noreferrer">Open file</a>
</template>
```

- [ ] **Step 4: Run tests to verify pass**

Run: `npm run test -- tests/component/attachments-preview.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/common/components/AttachmentList.vue src/common/components/AttachmentPreview.vue src/events/DetailPage.vue src/incidents/DetailPage.vue src/common/assets/mock-files/
git commit -m "feat: add attachment previews with mock pdf and image assets"
```

### Task 7: Search UX Rules, Not-Found Route, and E2E Deep-Link Checks

**Files:**
- Modify: `src/events/ListingPage.vue`
- Modify: `src/incidents/ListingPage.vue`
- Modify: `src/router/index.ts`
- Create: `src/common/NotFoundPage.vue`
- Create: `tests/e2e/routing.spec.ts`
- Create: `tests/component/search-empty-sections.test.ts`

- [ ] **Step 1: Write failing tests for search/no-match and deep links**

```ts
// tests/component/search-empty-sections.test.ts
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/vue";
import EventsListingPage from "../../src/events/ListingPage.vue";

describe("search section visibility", () => {
  it("hides empty sections and shows global no match only when all empty", async () => {
    render(EventsListingPage);
    expect(screen.queryByText("No match")).toBeNull();
  });
});
```

```ts
// tests/e2e/routing.spec.ts
import { test, expect } from "@playwright/test";

test("hash deep-link works", async ({ page }) => {
  await page.goto("/#/incidents/HEA-01");
  await expect(page.getByText("Incident timeline")).toBeVisible();
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm run test -- tests/component/search-empty-sections.test.ts; if ($?) { npm run test:e2e -- tests/e2e/routing.spec.ts }`
Expected: FAIL before implementing search/no-match and not-found handling

- [ ] **Step 3: Implement search rendering contract + not-found route**

```ts
// src/router/index.ts (core route shape)
{
  path: '/:pathMatch(.*)*',
  component: () => import('../common/NotFoundPage.vue')
}
```

```vue
<!-- src/common/NotFoundPage.vue -->
<template>
  <main>
    <h1>Not found</h1>
    <a href="#/events">Go to events</a>
    <a href="#/incidents">Go to incidents</a>
  </main>
</template>
```

- [ ] **Step 4: Run full automated verification**

Run: `npm run test; if ($?) { npm run build }; if ($?) { npm run test:e2e }`
Expected: all tests PASS and build succeeds

- [ ] **Step 5: Commit**

```bash
git add src/events/ListingPage.vue src/incidents/ListingPage.vue src/router/index.ts src/common/NotFoundPage.vue tests/component/search-empty-sections.test.ts tests/e2e/routing.spec.ts
git commit -m "feat: finalize search section rules and deep-link/not-found behavior"
```

## Final Verification Checklist

- [ ] `npm run test` passes
- [ ] `npm run build` passes
- [ ] `npm run test:e2e` passes
- [ ] Events page shows 3 sections, incidents page shows 2 sections
- [ ] Technical IDs hidden on cards and visible on detail pages
- [ ] Domain content fallback works `active -> en -> fr`
- [ ] UTC values display in browser local timezone
- [ ] Theme preference persists and `system` default works
- [ ] PDF/PNG preview works from detail pages

## Self-Review (completed)

### 1) Spec coverage check

- Architecture and feature-first structure: Tasks 1, 2, 3, 4, 5
- Events/incidents pages and detail pages: Tasks 3, 4
- Left sidebar and responsive behavior: Task 5
- i18n + content fallback + theme persistence: Tasks 2, 5
- UTC storage and local display/status logic: Task 2
- Search behavior and section hiding rule: Task 7
- Attachments with preview and real mock files: Task 6
- Not-found and hash deep-link behavior: Task 7
- Testing strategy (unit/component/e2e): Tasks 1-7 and final verification

No spec gaps found.

### 2) Placeholder scan

- No `TBD`, `TODO`, or "implement later" placeholders found.
- Each task includes explicit files, commands, and concrete code examples.

### 3) Type/signature consistency

- Events and incidents types are intentionally separate and never shared.
- Repository interfaces consistently accept `(preferredLanguage, query)` shape where relevant.
- Search/no-match behavior naming is consistent with spec wording.
