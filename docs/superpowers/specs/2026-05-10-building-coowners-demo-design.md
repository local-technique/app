# Building Co-Owners Demo Website Design

## 1) Goal and Scope

Build a lightweight, responsive static demo website for building co-owners, deployed on GitHub Pages, that demonstrates two timelines:

- Events and maintenance (`current`, `to come`, `past`)
- Incidents (`current`, `past`)

Primary success criterion: convince co-owners with polish and clarity.

Out of scope for this phase:

- Backend implementation
- Authentication/roles
- Advanced analytics or admin tooling

## 2) Technical Direction

### Chosen approach

- Framework: Vue 3
- Build tool: Vite
- Routing: `vue-router` with hash mode for GitHub Pages compatibility
- i18n: `vue-i18n` for UI strings
- Hosting: GitHub Pages (static assets)

### Why this approach

- Lightweight enough for demo use
- Clear structure and maintainability
- Easy to transition from mocked repositories to backend APIs later
- Hash routing avoids deep-link refresh issues on GitHub Pages

## 3) Information Architecture and Navigation

### Navigation shell

- Left sidebar on desktop/tablet
- Sidebar contains:
  - Main navigation entries: `Events & Maintenance`, `Incidents`
  - Language switcher
  - Theme switcher (`System`, `Light`, `Dark`)
- On mobile: compact top bar with menu drawer for the same controls

### Routes

- `/#/events`
- `/#/events/:id`
- `/#/incidents`
- `/#/incidents/:id`

## 4) Project Structure (Feature-First)

Domain files are intentionally not shared between `events` and `incidents`.
Duplication in TypeScript domain types is acceptable to preserve independent evolution.

- `events/`
  - `ListingPage`
  - `DetailPage`
  - `repositories/`
  - `types.ts`
  - `utils.ts`
- `incidents/`
  - `ListingPage`
  - `DetailPage`
  - `repositories/`
  - `types.ts`
  - `utils.ts`
- `common/`
  - Shared date utilities (UTC parse + local display + status helpers)
  - Shared fuzzy search utility
  - Shared UI primitives/helpers that are truly cross-domain

## 5) Data Design

### General rules

- Mocked data is not hardcoded in page components
- Pages consume repository interfaces only
- Repositories are async-shaped to mirror future backend usage

### Events and maintenance domain

Card content:

- Title
- Date or date range
- Optional location
- Category icon

Detail content includes card content plus:

- Short technical ID (e.g., `HEA-03`) shown only on detail pages
- Multi-line description
- Optional notified date/time
- Optional handlers (co-ownership board)
- Attachments

### Incidents domain

Current detail requirements match events where applicable, with incidents-specific behavior.

Additionally, incident details are designed to evolve with an internal timeline of micro-events, such as:

- Incident created
- Property management company notified
- Repair technician arrived
- Repair technician left
- Intervention report uploaded
- Incident finished

### Dataset size for demo

- Incidents: 4 total
  - 1 current
  - 3 past
- Events/maintenance: 6 total
  - 1 current heating maintenance
  - 1 future elevator maintenance
  - 1 future property management company visit (time displayed in card)
  - 3 past

## 6) Timeline Status Rules

### Events and maintenance

- For items with both `start` and `end`:
  - `to come`: `start` is after now
  - `current`: `start` is at or before now and `end` is at or after now
  - `past`: `end` is before now
- For items with `start` only (point-in-time events):
  - `to come`: `start` is after now
  - `past`: `start` is at or before now
  - (point-in-time items are never classified as `current`)

### Incidents

- Only `current` and `past` sections are shown
- `current`: incident is ongoing
- `past`: incident closed/ended by date semantics

All status classification is date-based.

## 7) Date and Time Strategy

- All mocked/stored date-time values use UTC (`ISO 8601` with `Z`)
- UI displays date/time in browser local timezone (expected mostly CET/CEST users)
- Date utility layer in `common` performs:
  - UTC parsing
  - Localized rendering
  - Consistent status comparisons relative to local "now"

## 8) Internationalization Strategy

### UI strings

- Managed with `vue-i18n`
- Locales: French and English

### Domain content localization

- Content fields (title, descriptions, location, etc.) are language-aware in mocked data
- Fallback chain for content resolution:
  1. Active language
  2. English (`en`)
  3. French (`fr`)

### Source and future backend behavior

- For demo, content can be primarily in French
- Repositories should accept a preferred language parameter now
- Future backend may return already-resolved localized content while frontend keeps fallback safety logic

## 9) Theme Strategy

- Supported modes: `system` (default), `light`, `dark`
- User override is persisted in local storage and reused on this device
- Theme applied via root data attribute and CSS variables for minimal overhead
- Theme preference applied before app mount to reduce flash of incorrect theme

## 10) Listing and Detail UX

### Events listing

- Sections shown in order: `Current`, `To come`, `Past`

### Incidents listing

- Sections shown in order: `Current`, `Past`

### Card behavior

- Entire card is clickable
- Technical ID hidden on listing cards
- Technical ID visible on detail pages

### Detail behavior

- Structured blocks for schedule, location, metadata, long description, attachments
- Incidents include a dedicated timeline block (ready for micro-events)

## 11) Search Behavior

- Single fuzzy text search per listing page
- Search scope is per-domain page (events or incidents)
- Search targets localized title and descriptions after fallback resolution
- Small debounce for typing smoothness

Search rendering rules:

- During search, empty sections are hidden
- Show global "no match" state with quick "clear search" only if no card remains across all sections

## 12) Attachments and File Preview

- Real sample files committed to repository assets (including mock invoice and elevator intervention report PDFs)
- Optional PNG sample attachment can be included
- Detail page lists file name, type, and size
- Click behavior:
  - In-browser preview when supported (PDF/PNG)
  - Fallback to new-tab open/download for unsupported formats

## 13) Responsiveness and Accessibility Baseline

### Responsive layout

- Mobile: vertical flow, one card per row, no horizontal scrolling
- Desktop/tablet: left sidebar + spacious content region

### Accessibility

- Keyboard navigable controls and cards
- Visible focus states
- Adequate color contrast in light and dark themes
- Icons paired with text labels
- Semantic landmarks and correct language attributes

## 14) Error and Edge Handling

- Async repositories return typed success/error outcomes
- Listing/detail pages show translated recoverable error states with retry option
- Invalid item IDs route to not-found state with navigation back links
- Central logging utility for developer visibility (console in demo)

## 15) Testing and Verification Scope

Test durable logic and behavior, not temporary mock dataset specifics.

- Unit tests:
  - UTC/local date handling and status classification
  - Locale fallback chain (`active -> en -> fr`)
  - Fuzzy search matching behavior
- Component/view tests:
  - Section visibility and ordering rules
  - Search behavior (hide empty sections; global no-match only when all empty)
  - Detail rendering essentials (ID visibility policy, attachments block, incidents timeline block)
- Router tests:
  - Hash-route deep links and reload behavior
  - Not-found routes for invalid IDs
- Manual QA:
  - Mobile responsiveness
  - Sidebar/drawer behavior
  - Language and theme switching + persistence
  - Attachment preview behavior for PDF/PNG

## 16) Backend Transition Plan (Design-Level)

- Keep repository interfaces stable and async from the start
- Start with `Mock*Repository` implementations in each domain
- Replace with `Api*Repository` implementations later without changing page/component contracts
- Allow backend-side search later while preserving existing frontend search UX contracts
