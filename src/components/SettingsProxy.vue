<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ProxyStatus, ManagedConfigPreview } from '@/types/proxy';

const status = ref<ProxyStatus | null>(null);
const previews = ref<ManagedConfigPreview[]>([]);
const error = ref<string | null>(null);
const copiedId = ref<string | null>(null);

async function fetchStatus() {
  try {
    status.value = await invoke<ProxyStatus>('get_proxy_status');
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

async function fetchPreviews() {
  try {
    previews.value = await invoke<ManagedConfigPreview[]>('get_managed_config_previews');
  } catch (e) {
    // Non-critical — just show empty
    previews.value = [];
  }
}

async function copyContent(toolId: string, content: string) {
  try {
    await navigator.clipboard.writeText(content);
    copiedId.value = toolId;
    setTimeout(() => (copiedId.value = null), 2000);
  } catch {
    // Clipboard may not be available in some webview contexts
  }
}

onMounted(() => {
  fetchStatus();
  fetchPreviews();
});
</script>

<template>
  <div>
    <h2 class="mb-1 text-xs font-medium text-text-primary">Proxy</h2>
    <p class="mb-4 text-xs text-text-secondary">
      Each connected server gets its own proxy endpoint, so AI tools see them individually.
      Use the Connected Apps toggles, or manually add the config snippet below.
    </p>

    <div v-if="error" class="rounded bg-status-error/10 px-3 py-2 text-xs text-status-error">
      Failed to get proxy status: {{ error }}
    </div>

    <div v-else-if="status" class="space-y-3">
      <div class="flex items-center gap-2">
        <span
          class="inline-block h-2 w-2 rounded-full"
          :class="status.running ? 'bg-status-connected' : 'bg-status-error'"
        />
        <span class="text-xs text-text-secondary">
          <template v-if="status.running">
            Running on port <span class="font-mono font-medium text-text-primary">{{ status.port }}</span>
          </template>
          <template v-else>Not running</template>
        </span>
      </div>

      <div v-if="status.running">
        <!-- Per-integration managed config previews -->
        <div v-if="previews.length > 0" class="space-y-4">
          <div v-for="preview in previews" :key="preview.toolId">
            <div class="mb-1.5 flex items-baseline gap-2">
              <span class="text-xs font-medium text-text-primary">{{ preview.toolName }}</span>
              <span v-if="preview.strategy === 'cli'" class="text-[10px] text-text-muted">(managed via CLI)</span>
            </div>
            <p class="mb-1 truncate text-[10px] text-text-muted">{{ preview.configPath }}</p>
            <div class="relative">
              <pre class="overflow-x-auto rounded bg-surface-2 p-3 font-mono text-xs text-text-primary">{{ preview.content }}</pre>
              <button
                class="absolute top-1.5 right-1.5 rounded bg-surface-3 px-2 py-0.5 text-[10px] text-text-muted transition hover:text-text-primary"
                @click="copyContent(preview.toolId, preview.content)"
              >
                {{ copiedId === preview.toolId ? 'Copied' : 'Copy' }}
              </button>
            </div>
          </div>
        </div>

        <!-- No integrations enabled hint -->
        <p v-else class="text-xs text-text-muted">
          Enable an integration in Connected Apps to see managed config.
        </p>
      </div>
    </div>

    <div v-else class="text-xs text-text-muted">Loading proxy status...</div>
  </div>
</template>
