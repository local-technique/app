# Markdown Renderer Feature Completion Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the custom markdown parser in `front/src/projects/utils.ts` to support all features from the markdownguide.org cheat sheet except HTML inclusion.

**Architecture:** Single-file change to the existing regex-based parser. A pre-processing step extracts fenced code blocks before the `\n{2,}` block split. New block-level checks (horizontal rules, blockquotes, ordered lists) and inline regexes (strikethrough, highlight, subscript, superscript, images, auto-links, task lists) are added. CSS is added to the three DetailPage components for styling new elements.

**Tech Stack:** TypeScript, Vue 3, Vitest, no external markdown libraries.

---

### Task 1: Write comprehensive tests first

**Files:**
- Modify: `front/src/projects/utils.test.ts`
- Read: `front/src/projects/utils.ts`

- [ ] **Step 1: Add test imports and describe block**

At the bottom of `utils.test.ts`, add a new `describe("renderProjectMarkdown extended features")` block before adding tests.

- [ ] **Step 2: Add test for image rendering**

```typescript
it("renders images", () => {
  const html = renderProjectMarkdown("![logo](https://example.com/logo.png)");
  expect(html).toContain('<img src="https://example.com/logo.png" alt="logo">');
});
```

- [ ] **Step 3: Add test for auto-link bare URLs**

```typescript
it("renders bare URLs as auto-links", () => {
  const html = renderProjectMarkdown("Visit https://example.com for info");
  expect(html).toContain('<a href="https://example.com"');
  expect(html).toContain(">https://example.com</a>");
});
```

- [ ] **Step 4: Add test for strikethrough**

```typescript
it("renders strikethrough", () => {
  const html = renderProjectMarkdown("This is ~~deleted~~ text");
  expect(html).toContain("<del>deleted</del>");
});
```

- [ ] **Step 5: Add test for highlight**

```typescript
it("renders highlight", () => {
  const html = renderProjectMarkdown("This is ==highlighted== text");
  expect(html).toContain("<mark>highlighted</mark>");
});
```

- [ ] **Step 6: Add test for subscript**

```typescript
it("renders subscript", () => {
  const html = renderProjectMarkdown("H~2~O");
  expect(html).toContain("<sub>2</sub>");
});
```

- [ ] **Step 7: Add test for superscript**

```typescript
it("renders superscript", () => {
  const html = renderProjectMarkdown("X^2^");
  expect(html).toContain("<sup>2</sup>");
});
```

- [ ] **Step 8: Add test for horizontal rules**

```typescript
it("renders horizontal rules from ---, ***, and ___", () => {
  expect(renderProjectMarkdown("---")).toContain("<hr>");
  expect(renderProjectMarkdown("***")).toContain("<hr>");
  expect(renderProjectMarkdown("___")).toContain("<hr>");
});
```

- [ ] **Step 9: Add test for blockquotes**

```typescript
it("renders blockquotes", () => {
  const html = renderProjectMarkdown("> This is a quote");
  expect(html).toContain("<blockquote>");
  expect(html).toContain("This is a quote");
});
```

- [ ] **Step 10: Add test for ordered lists**

```typescript
it("renders ordered lists", () => {
  const html = renderProjectMarkdown("1. First\n2. Second\n3. Third");
  expect(html).toContain("<ol>");
  expect(html).toContain("<li>First</li>");
  expect(html).toContain("<li>Second</li>");
  expect(html).toContain("</ol>");
});
```

- [ ] **Step 11: Add test for ordered list with custom start**

```typescript
it("renders ordered lists with correct start attribute", () => {
  const html = renderProjectMarkdown("3. Third\n4. Fourth");
  expect(html).toContain('<ol start="3">');
  expect(html).toContain("<li>Third</li>");
  expect(html).toContain("<li>Fourth</li>");
});
```

- [ ] **Step 12: Add test for task lists (unchecked and checked)**

```typescript
it("renders task lists", () => {
  const html = renderProjectMarkdown("- [ ] Todo\n- [x] Done");
  expect(html).toContain('<input type="checkbox" disabled>');
  expect(html).toContain('<input type="checkbox" disabled checked>');
  expect(html).toContain("Todo");
  expect(html).toContain("Done");
});
```

- [ ] **Step 13: Add test for fenced code blocks**

```typescript
it("renders fenced code blocks", () => {
  const html = renderProjectMarkdown("```\nconst x = 1;\n```");
  expect(html).toContain("<pre><code>");
  expect(html).toContain("const x = 1;");
  expect(html).toContain("</code></pre>");
});
```

- [ ] **Step 14: Add test for fenced code blocks with language**

```typescript
it("renders fenced code blocks with language class", () => {
  const html = renderProjectMarkdown("```js\nconst x = 1;\n```");
  expect(html).toContain('<pre><code class="language-js">');
  expect(html).toContain("const x = 1;");
});
```

- [ ] **Step 15: Add test for fenced code blocks containing blank lines**

```typescript
it("renders fenced code blocks with blank lines inside", () => {
  const markdown = "```\nline1\n\nline2\n```";
  const html = renderProjectMarkdown(markdown);
  expect(html).toContain("line1");
  expect(html).toContain("line2");
});
```

- [ ] **Step 16: Run tests to see them fail**

```bash
cd front
npx vitest run src/projects/utils.test.ts
```

Expected: all new tests fail (feature not implemented yet), existing 7 tests pass.

- [ ] **Step 17: Commit test file**

```bash
git add front/src/projects/utils.test.ts
git commit -m "test: add markdown feature tests (expected to fail)"
```

---

### Task 2: Implement inline features (images, auto-links, strikethrough, highlight, subscript, superscript)

**Files:**
- Modify: `front/src/projects/utils.ts` (function `renderInlineMarkdown`, lines 160-167)

- [ ] **Step 1: Reorder and extend `renderInlineMarkdown`**

Replace the existing inline rendering function (lines 160-167):

```typescript
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
    .replace(/(https?:\/\/[^\s<>"')]+)/g, (match: string, offset: number, full: string) => {
      const before = full.slice(0, offset);
      if (before.lastIndexOf("<") > before.lastIndexOf(">")) {
        return match;
      }
      return `<a href="${match}" rel="noopener noreferrer" target="_blank">${match}</a>`;
    });
}
```

Key design decisions:
- Image (`![]()`) before link (`[]()`) — they don't conflict syntax-wise but logical ordering
- Strikethrough (`~~`) before subscript (`~`) — so `~~` is consumed first, leaving `~` for subscript
- Subscript regex uses negative lookbehind/lookahead to prevent matching inside `~~` (strikethrough)
- Auto-link runs last with a context check to avoid double-wrapping URLs already inside `<a>` or `<img>` tags

- [ ] **Step 2: Run tests**

```bash
cd front
npx vitest run src/projects/utils.test.ts
```

Expected: image, auto-link, strikethrough, highlight, subscript, superscript tests pass. Block-level and fenced code block tests still fail.

- [ ] **Step 3: Commit**

```bash
git add front/src/projects/utils.ts
git commit -m "feat: add inline markdown features (images, auto-links, strikethrough, highlight, subscript, superscript)"
```

---

### Task 3: Implement block-level features (horizontal rules, blockquotes, ordered lists)

**Files:**
- Modify: `front/src/projects/utils.ts` (function `renderProjectMarkdown`, lines 222-267)

- [ ] **Step 1: Add helper functions before `renderProjectMarkdown`**

Add these helpers before `renderProjectMarkdown` (before line 222):

```typescript
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
```

- [ ] **Step 2: Add horizontal rule detection in `renderProjectMarkdown`**

After the heading check (line 233, after `if (heading) { ... }`), add:

```typescript
if (/^(?:[-*_]){3,}\s*$/.test(block.trim())) {
  return "<hr>";
}
```

- [ ] **Step 3: Add blockquote detection**

After the horizontal rule check and before the table check, add:

```typescript
if (lines.every((line) => isBlockquoteLine(line) || line.trim() === "")) {
  const content = lines
    .filter((line) => line.trim() !== "")
    .map((line) => renderInlineMarkdown(line.trim().replace(/^>\s*/, "")))
    .join("<br>");
  return `<blockquote><p>${content}</p></blockquote>`;
}
```

- [ ] **Step 4: Extend list detection to handle ordered lists**

Replace the existing list detection block (lines 241-261). Change the condition from `if (lines.some((line) => isBulletLine(line)))` to `if (lines.some((line) => isBulletLine(line) || isOrderedLine(line)))` and add an ordered list branch:

```typescript
if (lines.some((line) => isBulletLine(line) || isOrderedLine(line))) {
  let result = "";
  let i = 0;
  while (i < lines.length) {
    if (isBulletLine(lines[i])) {
      const items: string[] = [];
      while (i < lines.length && isBulletLine(lines[i])) {
        items.push(`<li>${renderInlineMarkdown(bulletContent(lines[i]))}</li>`);
        i++;
      }
      result += `<ul>${items.join("")}</ul>`;
    } else if (isOrderedLine(lines[i])) {
      const items: string[] = [];
      const start = orderedStart(lines[i]);
      const openTag = start !== 1 ? `<ol start="${start}">` : "<ol>";
      while (i < lines.length && isOrderedLine(lines[i])) {
        items.push(`<li>${renderInlineMarkdown(orderedContent(lines[i]))}</li>`);
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
```

- [ ] **Step 5: Run tests**

```bash
cd front
npx vitest run src/projects/utils.test.ts
```

Expected: horizontal rule, blockquote, ordered list tests pass. Fenced code block and task list tests still fail.

- [ ] **Step 6: Commit**

```bash
git add front/src/projects/utils.ts
git commit -m "feat: add block-level markdown features (HR, blockquotes, ordered lists)"
```

---

### Task 4: Implement task lists

**Files:**
- Modify: `front/src/projects/utils.ts`

- [ ] **Step 1: Add task list helper function**

Add before `renderProjectMarkdown`:

```typescript
interface TaskItem {
  checked: boolean;
  content: string;
}

function parseTaskItem(line: string, contentExtractor: (line: string) => string): TaskItem | null {
  const content = contentExtractor(line);
  const trimmed = content.trim();
  const match = trimmed.match(/^\[([ xX])\]\s*(.*)$/);
  if (!match) return null;
  return { checked: match[1] !== " ", content: match[2] };
}
```

- [ ] **Step 2: Modify list item rendering to detect task items**

In the bullet list branch of `renderProjectMarkdown`, change:
```typescript
items.push(`<li>${renderInlineMarkdown(bulletContent(lines[i]))}</li>`);
```
to:
```typescript
const bullet = bulletContent(lines[i]);
const task = parseTaskItem(bullet, (s) => s);
if (task) {
  items.push(`<li>${task.checked ? '<input type="checkbox" disabled checked>' : '<input type="checkbox" disabled>'} ${renderInlineMarkdown(task.content)}</li>`);
} else {
  items.push(`<li>${renderInlineMarkdown(bullet)}</li>`);
}
```

In the ordered list branch, change:
```typescript
items.push(`<li>${renderInlineMarkdown(orderedContent(lines[i]))}</li>`);
```
to:
```typescript
const ordered = orderedContent(lines[i]);
const task = parseTaskItem(ordered, (s) => s);
if (task) {
  items.push(`<li>${task.checked ? '<input type="checkbox" disabled checked>' : '<input type="checkbox" disabled>'} ${renderInlineMarkdown(task.content)}</li>`);
} else {
  items.push(`<li>${renderInlineMarkdown(ordered)}</li>`);
}
```

- [ ] **Step 3: Run tests**

```bash
cd front
npx vitest run src/projects/utils.test.ts
```

Expected: task list test passes. Fenced code block tests still fail.

- [ ] **Step 4: Commit**

```bash
git add front/src/projects/utils.ts
git commit -m "feat: add task list support"
```

---

### Task 5: Implement fenced code blocks

**Files:**
- Modify: `front/src/projects/utils.ts`

- [ ] **Step 1: Add fenced code block pre-processing**

Replace the beginning of `renderProjectMarkdown` (the block split logic at lines 222-227) to add a pre-processing step for fenced code blocks:

```typescript
export function renderProjectMarkdown(markdown: string): string {
  const normalized = markdown.replace(/\r\n/g, "\n");

  // Extract fenced code blocks and replace with placeholders
  const codeBlocks: string[] = [];
  const withoutFences = normalized.replace(
    /```(\w*)\n([\s\S]*?)```/g,
    (_match: string, lang: string, code: string) => {
      const index = codeBlocks.length;
      const langAttr = lang ? ` class="language-${lang}"` : "";
      codeBlocks.push(`<pre><code${langAttr}>${escapeHtml(code.trimEnd())}</code></pre>`);
      return `\n\n__CODEBLOCK_${index}__\n\n`;
    },
  );

  // Also support ~~~ fences (but not ~~ which is strikethrough)
  const withoutTildeFences = withoutFences.replace(
    /~~~(\w*)\n([\s\S]*?)~~~/g,
    (_match: string, lang: string, code: string) => {
      const index = codeBlocks.length;
      const langAttr = lang ? ` class="language-${lang}"` : "";
      codeBlocks.push(`<pre><code${langAttr}>${escapeHtml(code.trimEnd())}</code></pre>`);
      return `\n\n__CODEBLOCK_${index}__\n\n`;
    },
  );

  const blocks = withoutTildeFences
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean);

  return blocks
    .map((block) => {
      // Check for fenced code block placeholder
      const codeMatch = block.match(/^__CODEBLOCK_(\d+)__$/);
      if (codeMatch) {
        return codeBlocks[parseInt(codeMatch[1], 10)];
      }

      const lines = block.split("\n");
      // ... rest of existing logic unchanged
    })
    .join("");
}
```

Note: `~~~` fences use 3+ tildes min (to distinguish from `~~` strikethrough). The regex `~~~` matches exactly 3 tildes — for more, the regex would need `~{3,}` but then we must ensure the closing fence has `~{3,}` too. For simplicity, support exactly ` ``` ` and ` ~~~ `.

- [ ] **Step 2: Run tests**

```bash
cd front
npx vitest run src/projects/utils.test.ts
```

Expected: all tests pass (including fenced code block tests).

- [ ] **Step 3: Commit**

```bash
git add front/src/projects/utils.ts
git commit -m "feat: add fenced code block support"
```

---

### Task 6: Add CSS styles for new elements

**Files:**
- Modify: `front/src/projects/DetailPage.vue`
- Modify: `front/src/incidents/DetailPage.vue`
- Modify: `front/src/events/DetailPage.vue`

- [ ] **Step 1: Add styles for new markdown elements to projects DetailPage**

In `front/src/projects/DetailPage.vue`, inside the scoped `<style>` block, extend the existing `:deep()` rules for `.project-description`:

```css
.project-description :deep(pre) {
  margin: 0.7rem 0 0;
  border-radius: 0.35rem;
  padding: 0.75rem 1rem;
  background: rgba(127, 127, 127, 0.1);
  overflow-x: auto;
}
.project-description :deep(pre code) {
  background: none;
  padding: 0;
}
.project-description :deep(blockquote) {
  margin: 0.7rem 0 0;
  padding: 0.25rem 0 0.25rem 1rem;
  border-left: 3px solid rgba(127, 127, 127, 0.3);
  color: rgba(127, 127, 127, 0.9);
}
.project-description :deep(hr) {
  margin: 0.7rem 0;
  border: none;
  border-top: 1px solid rgba(127, 127, 127, 0.25);
}
.project-description :deep(input[type="checkbox"]) {
  margin: 0 0.35rem 0 0;
  pointer-events: none;
}
.project-description :deep(ol) {
  margin: 0.7rem 0 0;
  padding-left: 1.3rem;
}
.project-description :deep(mark) {
  border-radius: 0.2rem;
  padding: 0.05rem 0.15rem;
  background: rgba(255, 230, 0, 0.35);
}
.project-description :deep(sub),
.project-description :deep(sup) {
  font-size: 0.75em;
}
.project-description :deep(del) {
  text-decoration: line-through;
}
```

- [ ] **Step 2: Repeat styles for incidents DetailPage**

Add identical rules to `front/src/incidents/DetailPage.vue`, replacing `.project-description` with `.rendered-description`.

- [ ] **Step 3: Repeat styles for events DetailPage**

Add identical rules to `front/src/events/DetailPage.vue`, replacing `.project-description` with `.rendered-description`.

- [ ] **Step 4: Run full test suite**

```bash
cd front
npx vitest run
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add front/src/projects/DetailPage.vue front/src/incidents/DetailPage.vue front/src/events/DetailPage.vue
git commit -m "style: add CSS for new markdown elements"
```

---

### Task 7: Verify build

- [ ] **Step 1: Run TypeScript check**

```bash
cd front
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 2: Run lint**

```bash
cd front
npm run lint
```

Expected: no errors.

- [ ] **Step 3: Run build**

```bash
cd front
npm run build
```

Expected: build succeeds.

- [ ] **Step 4: Run full test suite one final time**

```bash
cd front
npx vitest run
```

Expected: all tests pass.

---

### Self-Review Checklist

**Spec coverage:**
- ✅ Ordered lists (Task 3)
- ✅ Horizontal rules (Task 3)
- ✅ Fenced code blocks (Task 5)
- ✅ Strikethrough (Task 2)
- ✅ Task lists (Task 4)
- ✅ Highlight (Task 2)
- ✅ Subscript (Task 2)
- ✅ Superscript (Task 2)
- ✅ Images (Task 2)
- ✅ Auto-links/links (Task 2 — links were already done, auto-links added)
- ✅ Blockquotes (Task 3)
- ✅ Tables (already implemented before this work)
- ✅ Code/inline code (already implemented before this work)
- ❌ HTML inclusion (excluded by design)

**Placeholder check:** No TODOs, TBDs, or incomplete sections.

**Type consistency:** All helper function signatures are consistent. All regex patterns tested prior to inclusion.
