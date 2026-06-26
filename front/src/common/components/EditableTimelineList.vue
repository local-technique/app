<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Calendar, CircleCheck, Pencil, Save, Trash2, X } from "@lucide/vue";
import type { TimelineEntry } from "./TimelineList.vue";
import { toDateLocalInput, toUtcFromDateLocalInput, todayDateInput } from "../dateInput";
const { t } = useI18n();

const props = defineProps<{ entries: TimelineEntry[]; canEdit: boolean }>();
const emit = defineEmits<{
  (e: "add", payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): void;
  (e: "update", entryId: string, payload: { atUtc: string | null; sortOrder: number; fields: Record<string, string> }): void;
  (e: "delete", entryId: string): void;
}>();

const pad = (n: number) => n.toString().padStart(2, "0");
const nowTime = () => `${pad(new Date().getHours())}:${pad(new Date().getMinutes())}`;

function combToUtc(d: string, t: string) {
  return d && t ? new Date(`${d}T${t}:00`).toISOString() : null;
}

function makeDateUid() {
  return `tl-d-${Math.random().toString(36).slice(2, 9)}`;
}

function triggerCalendar(id: string) {
  const el = document.getElementById(id) as HTMLInputElement | null;
  if (el?.showPicker) el.showPicker();
}

function onDatePick(e: Event, setFn: (v: string) => void) {
  const target = e.target as HTMLInputElement;
  if (target.value) setFn(target.value);
}

const editingId = ref<string | null>(null);
const showNewForm = ref(false);

const editDate = ref("");
const editTime = ref("");
const editTitle = ref("");
const editDetails = ref("");
const editDateUid = ref(makeDateUid());

const newDate = ref(todayDateInput());
const newTime = ref(nowTime());
const newTitle = ref("");
const newDetails = ref("");
const newDateUid = ref(makeDateUid());

watch(editDate, (val, old) => {
  if (old && !val) editTime.value = "";
  else if (!old && val) editTime.value = nowTime();
});
watch(newDate, (val, old) => {
  if (old && !val) newTime.value = "";
  else if (!old && val) newTime.value = nowTime();
});

function startEdit(entry: TimelineEntry) {
  editingId.value = entry.id;
  if (entry.atUtc) {
    const d = new Date(entry.atUtc);
    editDate.value = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
    editTime.value = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  } else {
    editDate.value = "";
    editTime.value = "";
  }
  editTitle.value = entry.title;
  editDetails.value = entry.details;
}

function cancelEdit() {
  editingId.value = null;
}

function saveEdit(entryId: string) {
  if (!editTitle.value.trim()) return;
  const idx = props.entries.findIndex((e) => e.id === entryId);
  emit("update", entryId, {
    atUtc: combToUtc(editDate.value, editTime.value),
    sortOrder: idx >= 0 ? idx : 0,
    fields: { title: editTitle.value, details: editDetails.value },
  });
  editingId.value = null;
}

function saveNew() {
  if (!newTitle.value.trim()) return;
  emit("add", {
    atUtc: combToUtc(newDate.value, newTime.value),
    sortOrder: props.entries.length,
    fields: { title: newTitle.value, details: newDetails.value },
  });
  showNewForm.value = false;
  newTitle.value = "";
  newDetails.value = "";
  newDate.value = todayDateInput();
  newTime.value = nowTime();
}

function cancelNew() {
  showNewForm.value = false;
  newTitle.value = "";
  newDetails.value = "";
  newDate.value = todayDateInput();
  newTime.value = nowTime();
}
</script>

<template>
  <div class="TimelineList">
    <button v-if="canEdit && !showNewForm" class="timeline-add-btn" @click="showNewForm = true">
      + {{ t("labels.addTimelineEntry") }}
    </button>

    <article v-if="canEdit && showNewForm" class="timeline-row timeline-row--editing">
      <div class="timeline-date-slot">
        <div class="timeline-dt-inputs">
          <span class="tl-dwrap">
            <input :id="newDateUid" type="date" class="tl-dnative" @change="onDatePick($event, (v) => newDate = v)" />
            <input v-model="newDate" type="text" inputmode="numeric" placeholder="YYYY-MM-DD" class="tl-dtext" />
            <button type="button" class="tl-dbtn" @click="triggerCalendar(newDateUid)" tabindex="-1"><Calendar :size="12" /></button>
          </span>
          <input v-model="newTime" type="text" inputmode="numeric" placeholder="HH:MM" class="timeline-input tl-tinput" />
        </div>
      </div>
      <div class="timeline-axis" aria-hidden="true"><span class="timeline-dot" /></div>
      <div class="timeline-card timeline-entry-card">
        <div class="timeline-entry-form">
          <input v-model="newTitle" :placeholder="t('labels.title')" class="timeline-input" />
          <textarea v-model="newDetails" :placeholder="t('labels.details')" class="timeline-input timeline-input--details"></textarea>
        </div>
        <div class="timeline-entry-actions">
          <button class="timeline-action-btn" style="background: rgba(34, 197, 94, 0.85)" :disabled="!newTitle" @click="saveNew">
            <Save :size="14" />
          </button>
          <button class="timeline-action-btn" style="background: rgba(127, 127, 127, 0.6)" @click="cancelNew">
            <X :size="14" />
          </button>
        </div>
      </div>
    </article>

    <article
      class="timeline-row"
      v-for="entry in entries"
      :key="entry.id"
      :class="{ 'timeline-row--editing': editingId === entry.id }"
    >
      <div class="timeline-date-slot">
        <template v-if="canEdit && editingId === entry.id">
          <div class="timeline-dt-inputs">
            <span class="tl-dwrap">
              <input :id="editDateUid" type="date" class="tl-dnative" @change="onDatePick($event, (v) => editDate = v)" />
              <input v-model="editDate" type="text" inputmode="numeric" placeholder="YYYY-MM-DD" class="tl-dtext" />
              <button type="button" class="tl-dbtn" @click="triggerCalendar(editDateUid)" tabindex="-1"><Calendar :size="12" /></button>
            </span>
            <input v-model="editTime" type="text" inputmode="numeric" placeholder="HH:MM" class="timeline-input tl-tinput" />
          </div>
        </template>
        <template v-else>
          <span v-if="entry.isPending" class="pending-badge">{{ entry.atLabel }}</span>
          <span v-else class="timeline-date-label">
            <span>{{ entry.atDateLabel }}</span>
            <span class="timeline-time-label">{{ entry.atTimeLabel }}</span>
          </span>
        </template>
      </div>
      <div class="timeline-axis" aria-hidden="true"><span class="timeline-dot" /></div>
      <div class="timeline-card timeline-entry-card">
        <template v-if="canEdit && editingId === entry.id">
          <div class="timeline-entry-form">
            <input v-model="editTitle" :placeholder="t('labels.title')" class="timeline-input" />
            <textarea v-model="editDetails" :placeholder="t('labels.details')" class="timeline-input timeline-input--details"></textarea>
          </div>
          <div class="timeline-entry-actions">
            <button class="timeline-action-btn" style="background: rgba(34, 197, 94, 0.85)" :disabled="!editTitle" @click="saveEdit(entry.id)">
              <Save :size="14" />
            </button>
            <button class="timeline-action-btn" style="background: rgba(127, 127, 127, 0.6)" @click="cancelEdit">
              <X :size="14" />
            </button>
          </div>
        </template>
        <template v-else>
          <span v-if="entry.lastModifiedBy" class="tl-user-avatar" :title="entry.lastModifiedBy.fullName">{{ entry.lastModifiedBy.initials }}</span>
          <div class="tl-card-body">
            <h3 class="timeline-card-title timeline-entry-title">
              <CircleCheck v-if="!entry.isPending" class="timeline-entry-icon" :size="16" :stroke-width="2.4" aria-hidden="true" />
              <span>{{ entry.title }}</span>
            </h3>
            <p v-if="entry.details" class="timeline-entry-details">{{ entry.details }}</p>
            <div v-if="canEdit" class="timeline-entry-actions">
              <button class="timeline-action-btn" style="background: rgba(72, 144, 255, 0.78)" @click="startEdit(entry)">
                <Pencil :size="14" />
              </button>
              <button class="timeline-action-btn" style="background: rgba(220, 38, 38, 0.85)" @click="emit('delete', entry.id)">
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </template>
      </div>
    </article>
  </div>
</template>

<style scoped>
.TimelineList {
  --timeline-accent: rgba(72, 144, 255, 0.78);
  display: grid;
  gap: 0;
  margin-top: 0.75rem;
}

.TimelineList .timeline-row {
  display: grid;
  grid-template-columns: minmax(5.7rem, 8.2rem) 1.25rem minmax(0, 1fr);
  column-gap: 0.6rem;
  position: relative;
}

.TimelineList .timeline-date-slot {
  color: var(--muted-fg);
  font-size: 0.78rem;
  font-weight: 800;
  line-height: 1.25;
  padding-top: 0.35rem;
  text-align: right;
  white-space: nowrap;
}

.TimelineList .timeline-date-label { display: inline-block; }

.TimelineList .timeline-time-label::before { content: ", "; }

.TimelineList .timeline-axis { display: flex; justify-content: center; position: relative; }

.TimelineList .timeline-axis::before {
  background: var(--timeline-accent);
  bottom: -0.2rem;
  content: "";
  position: absolute;
  top: 0.55rem;
  width: 2px;
}

.TimelineList .timeline-row:last-child .timeline-axis::before { bottom: calc(100% - 0.6rem); }

.TimelineList .timeline-dot {
  background: var(--page-bg);
  border: 3px solid var(--timeline-accent);
  border-radius: 999px;
  height: 0.75rem;
  margin-top: 0.28rem;
  width: 0.75rem;
  z-index: 1;
}

.TimelineList .timeline-entry-card { display: flex; gap: 0.5rem; align-items: flex-start; margin-bottom: calc(0.32rem + 10px); padding: 0.32rem 0.62rem; position: relative; }

.TimelineList .tl-card-body { flex: 1; min-width: 0; }

.TimelineList .timeline-entry-title { align-items: center; display: flex; gap: 0.38rem; line-height: 1.15; margin: 0; }

.TimelineList .timeline-entry-details { color: var(--muted-fg); font-size: 0.82rem; line-height: 1.2; margin: 0.12rem 0 0; }

.TimelineList .timeline-entry-icon { color: var(--timeline-accent); flex: 0 0 auto; }

.TimelineList .tl-user-avatar {
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 50%;
  background: rgba(127, 127, 127, 0.2);
  color: var(--muted-fg);
  cursor: default;
  flex: 0 0 auto;
  font-size: 0.55rem;
  font-weight: 700;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.TimelineList .pending-badge {
  background: rgba(255, 139, 26, 0.2);
  border: 1px solid rgba(255, 139, 26, 0.62);
  border-radius: 999px;
  color: #ff8b1a;
  display: inline-flex;
  padding: 0.18rem 0.45rem;
}

.TimelineList .timeline-entry-actions {
  position: absolute;
  right: 0.35rem;
  top: 0.35rem;
  display: flex;
  gap: 0.2rem;
}

.TimelineList .timeline-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 0.2rem;
  padding: 0.2rem;
  cursor: pointer;
  color: #fff;
  width: 1.5rem;
  height: 1.5rem;
  line-height: 1;
}

.TimelineList .timeline-input {
  border: 1px solid rgba(127, 127, 127, 0.35);
  border-radius: 0.4rem;
  padding: 0.3rem 0.45rem;
  background: var(--page-bg);
  color: var(--fg);
  font-size: 0.82rem;
  width: 100%;
  box-sizing: border-box;
}

.TimelineList .timeline-dt-inputs {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.TimelineList .tl-tinput {
  padding: 0.08rem 0.3rem;
  min-width: 0;
  text-align: center;
}

.TimelineList .tl-dwrap {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 100%;
}

.TimelineList .tl-dnative {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  overflow: hidden;
  pointer-events: none;
}

.TimelineList .tl-dtext {
  flex: 1;
  min-width: 0;
  width: 100%;
  border: 1px solid rgba(127, 127, 127, 0.35);
  border-radius: 0.4rem;
  padding: 0.08rem 1.2rem 0.08rem 0.3rem;
  text-align: center;
  background: var(--page-bg);
  color: var(--fg);
  box-sizing: border-box;
}

.TimelineList .tl-dbtn {
  position: absolute;
  right: 0.1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.1rem;
  cursor: pointer;
  border: none;
  background: none;
  color: var(--muted-fg);
  border-radius: 0.2rem;
}

.TimelineList .tl-dbtn:hover {
  color: var(--fg);
  background: rgba(127, 127, 127, 0.15);
}

.TimelineList .timeline-entry-form .timeline-input {
  padding: 0.12rem 0.45rem;
}

.TimelineList .timeline-entry-form .timeline-input--details {
  min-height: 0;
  resize: vertical;
}

.TimelineList .timeline-entry-form {
  display: grid;
  gap: 0;
  padding-right: 0;
  flex: 1;
}

.TimelineList .timeline-add-btn {
  border: 1.5px dashed rgba(127, 127, 127, 0.4);
  border-radius: 0.5rem;
  padding: 0.45rem 0.75rem;
  background: transparent;
  color: var(--muted-fg);
  cursor: pointer;
  font-size: 0.85rem;
  display: block;
  width: fit-content;
  margin-bottom: 0.4rem;
}

.TimelineList .timeline-add-btn:hover {
  border-color: var(--timeline-accent);
  color: var(--timeline-accent);
}

.TimelineList .timeline-row--editing {
  background: rgba(72, 144, 255, 0.06);
  border-radius: 0.4rem;
  padding: 0.2rem 0;
  margin-bottom: 0.75rem;
}

.TimelineList .timeline-action-btn:disabled { opacity: 0.4; cursor: default; }

@media (max-width: 560px) {
  .TimelineList .timeline-row { grid-template-columns: 5.7rem 1rem minmax(0, 1fr); column-gap: 0.45rem; }
  .TimelineList .timeline-date-slot { font-size: 0.7rem; white-space: normal; }
  .TimelineList .timeline-date-label { display: grid; gap: 0.05rem; justify-items: end; }
  .TimelineList .timeline-time-label::before { content: ""; }
}
</style>
