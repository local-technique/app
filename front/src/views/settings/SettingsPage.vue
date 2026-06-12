<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Check, Copy, RefreshCcw, Trash2 } from "@lucide/vue";
import { createToken, getToken, revokeToken, type CreateTokenResponse, type TokenInfoResponse } from "./api";

const { t } = useI18n();

interface UserInfo {
  email: string;
}

const userInfo = ref<UserInfo | null>(null);
const tokenInfo = ref<TokenInfoResponse | null>(null);
const createdToken = ref<CreateTokenResponse | null>(null);
const loading = ref(true);
const error = ref(false);
const copyConfirm = ref(false);
const saving = ref(false);

async function loadData(): Promise<void> {
  loading.value = true;
  error.value = false;
  try {
    const [userRes, token] = await Promise.all([
      fetch(`${import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080"}/me`, {
        headers: { Authorization: `Bearer ${(await import("../../auth/session")).getAccessToken()}` },
      }),
      getToken(),
    ]);
    if (userRes.ok) {
      userInfo.value = await userRes.json();
    }
    tokenInfo.value = token;
  } catch {
    error.value = true;
  } finally {
    loading.value = false;
  }
}

async function handleGenerate(): Promise<void> {
  saving.value = true;
  error.value = false;
  try {
    const response = await createToken();
    createdToken.value = response;
    tokenInfo.value = {
      id: response.id,
      token_prefix: response.token_prefix,
      created_at: response.created_at,
      last_used_at: null,
    };
  } catch {
    error.value = true;
  } finally {
    saving.value = false;
  }
}

async function handleRevoke(): Promise<void> {
  if (!confirm(t("labels.tokenRevokeConfirm"))) {
    return;
  }
  saving.value = true;
  error.value = false;
  try {
    await revokeToken();
    tokenInfo.value = null;
    createdToken.value = null;
  } catch {
    error.value = true;
  } finally {
    saving.value = false;
  }
}

async function handleCopyToken(): Promise<void> {
  if (!createdToken.value) return;
  try {
    await navigator.clipboard.writeText(createdToken.value.token_full);
    copyConfirm.value = true;
    setTimeout(() => { copyConfirm.value = false; }, 2000);
  } catch {
    error.value = true;
  }
}

onMounted(loadData);
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("labels.settingsPageTitle") }}</h1>

    <p v-if="error" class="error-message">{{ t("labels.tokenError") }}</p>

    <section v-if="userInfo" class="settings-section">
      <h2>{{ t("labels.settingsEmail") }}</h2>
      <p class="email-display">{{ userInfo.email }}</p>
    </section>

    <section class="settings-section">
      <h2>{{ t("labels.apiToken") }}</h2>

      <div v-if="loading" class="empty-state">...</div>

      <div v-else-if="createdToken" class="token-card token-generated">
        <p class="token-full-label">{{ t("labels.tokenGeneratedNotice") }}</p>
        <code class="token-full-display">{{ createdToken.token_full }}</code>
        <button class="icon-button" type="button" :title="t('labels.copyToken')" @click="handleCopyToken">
          <Check v-if="copyConfirm" :size="16" />
          <Copy v-else :size="16" />
          <span>{{ copyConfirm ? t("labels.tokenCopied") : t("labels.copyToken") }}</span>
        </button>
      </div>

      <div v-else-if="tokenInfo" class="token-card">
        <div class="token-detail">
          <span class="detail-label">{{ t("labels.tokenPrefix") }}</span>
          <span class="detail-value">{{ tokenInfo.token_prefix }}...</span>
        </div>
        <div class="token-detail">
          <span class="detail-label">{{ t("labels.tokenCreated") }}</span>
          <span class="detail-value">{{ new Date(tokenInfo.created_at).toLocaleString() }}</span>
        </div>
        <div class="token-detail">
          <span class="detail-label">{{ t("labels.tokenLastUsed") }}</span>
          <span class="detail-value">{{ tokenInfo.last_used_at ? new Date(tokenInfo.last_used_at).toLocaleString() : t("labels.never") }}</span>
        </div>
        <div class="token-actions">
          <button class="secondary-button" type="button" :disabled="saving" @click="handleGenerate">
            <RefreshCcw :size="14" />
            {{ t("labels.renewToken") }}
          </button>
          <button class="danger-button" type="button" :disabled="saving" @click="handleRevoke">
            <Trash2 :size="14" />
            {{ t("labels.revokeToken") }}
          </button>
        </div>
      </div>

      <div v-else class="token-card token-empty">
        <p>{{ t("labels.noApiToken") }}</p>
        <button class="primary-button" type="button" :disabled="saving" @click="handleGenerate">
          {{ t("labels.generateToken") }}
        </button>
      </div>
    </section>
  </main>
</template>

<style scoped>
.settings-section {
  margin-top: 1.5rem;
}

.settings-section h2 {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--muted-fg);
  margin: 0 0 0.6rem;
}

.email-display {
  font-size: 1.1rem;
  font-weight: 600;
}

.token-card {
  border: 1px solid var(--border-color);
  border-radius: 0.9rem;
  padding: 1rem;
  background: var(--panel-bg);
  display: grid;
  gap: 0.75rem;
}

.token-empty {
  align-items: start;
  gap: 0.8rem;
}

.token-empty p {
  margin: 0;
  color: var(--muted-fg);
}

.token-generated {
  gap: 0.6rem;
}

.token-full-label {
  font-size: 0.82rem;
  color: var(--muted-fg);
  margin: 0;
}

.token-full-display {
  display: block;
  padding: 0.7rem;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 0.5rem;
  font-family: monospace;
  font-size: 0.82rem;
  word-break: break-all;
  line-height: 1.5;
}

html[data-theme="dark"] .token-full-display,
html[data-theme="system"][data-resolved-theme="dark"] .token-full-display {
  background: rgba(255, 255, 255, 0.05);
}

.token-detail {
  display: flex;
  gap: 0.5rem;
  align-items: baseline;
}

.detail-label {
  font-size: 0.82rem;
  color: var(--muted-fg);
  min-width: 80px;
}

.detail-value {
  font-weight: 600;
}

.token-actions {
  display: flex;
  gap: 0.6rem;
  margin-top: 0.3rem;
}

.icon-button {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  border: 1px solid var(--control-border);
  border-radius: 0.5rem;
  padding: 0.45rem 0.7rem;
  background: var(--control-bg);
  color: var(--control-fg);
  cursor: pointer;
  font-size: 0.85rem;
}

.secondary-button,
.primary-button,
.danger-button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid var(--control-border);
  border-radius: 0.55rem;
  padding: 0.45rem 0.7rem;
  cursor: pointer;
  font-size: 0.85rem;
}

.primary-button {
  border-color: rgba(72, 144, 255, 0.7);
  background: rgba(72, 144, 255, 0.22);
  color: var(--control-fg);
}

.danger-button {
  border-color: rgba(215, 58, 73, 0.5);
  color: #d73a49;
  background: transparent;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.error-message {
  color: #f35a67;
  font-weight: 700;
}

.empty-state {
  color: var(--muted-fg);
}
</style>
