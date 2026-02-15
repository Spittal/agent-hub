<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useServersStore } from '@/stores/servers';
import type { AiToolInfo } from '@/types/integration';
import type { ProxyStatus } from '@/types/proxy';
import ToggleCard from './ToggleCard.vue';

const store = useServersStore();

const integrations = ref<AiToolInfo[] | null>(null);
const proxyStatus = ref<ProxyStatus | null>(null);
const error = ref<string | null>(null);
const togglingId = ref<string | null>(null);

const installed = computed(() =>
  integrations.value?.filter(t => t.installed) ?? []
);

async function fetchIntegrations() {
  try {
    integrations.value = await invoke<AiToolInfo[]>('detect_integrations');
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

async function fetchProxyStatus() {
  try {
    proxyStatus.value = await invoke<ProxyStatus>('get_proxy_status');
  } catch {
    // Non-critical for this section
  }
}

async function enable(id: string) {
  togglingId.value = id;
  try {
    await invoke('enable_integration', { id });
    await store.loadServers();
    store.autoConnectServers();
    await fetchIntegrations();
  } catch (e) {
    error.value = String(e);
  } finally {
    togglingId.value = null;
  }
}

async function disable(id: string) {
  togglingId.value = id;
  try {
    await invoke('disable_integration', { id });
    await fetchIntegrations();
  } catch (e) {
    error.value = String(e);
  } finally {
    togglingId.value = null;
  }
}

function serverSummary(server: AiToolInfo['existingServers'][number]): string {
  if (server.transport === 'http' && server.url) return server.url;
  if (server.command) {
    const parts = [server.command, ...(server.args ?? [])];
    const full = parts.join(' ');
    return full.length > 50 ? full.slice(0, 50) + '...' : full;
  }
  return server.transport;
}

onMounted(() => {
  fetchIntegrations();
  fetchProxyStatus();
});
</script>

<template>
  <div>
    <h2 class="mb-1 text-xs font-medium text-text-primary">Connected Apps</h2>
    <p class="mb-4 text-xs text-text-secondary">
      Automatically configure AI tools to use MCP Manager as their MCP server.
    </p>

    <div v-if="error" class="mb-3 rounded bg-status-error/10 px-3 py-2 text-xs text-status-error">
      {{ error }}
    </div>

    <div v-if="integrations && installed.length" class="space-y-2">
      <ToggleCard
        v-for="tool in installed"
        :key="tool.id"
        :label="tool.name"
        :enabled="tool.enabled"
        :toggling="togglingId === tool.id"
        :can-enable="proxyStatus?.running ?? false"
        :enable-label="tool.existingServers.length ? 'Migrate & Enable' : 'Enable'"
        @toggle="tool.enabled ? disable(tool.id) : enable(tool.id)"
      >
        <template #subtitle>
          <span
            v-if="tool.enabled && proxyStatus && tool.configuredPort !== proxyStatus.port"
            class="text-[10px] text-status-connecting"
          >Port outdated — restart app to fix</span>
          <span v-else-if="tool.enabled" class="text-[10px] text-text-muted">Port {{ tool.configuredPort }}</span>
        </template>

        <!-- Existing servers to migrate -->
        <div v-if="!tool.enabled && tool.existingServers.length" class="border-t border-border/50 px-3 py-2">
          <p class="mb-1.5 text-[10px] font-medium text-text-muted uppercase tracking-wide">
            Existing MCP servers to import
          </p>
          <div class="space-y-1">
            <div
              v-for="server in tool.existingServers"
              :key="server.name"
              class="flex items-center gap-2 rounded bg-surface-0 px-2 py-1.5"
            >
              <span class="font-mono text-[11px] font-medium text-text-secondary">{{ server.name }}</span>
              <span class="truncate text-[10px] text-text-muted">{{ serverSummary(server) }}</span>
            </div>
          </div>
          <p class="mt-1.5 text-[10px] text-text-muted">
            These will be imported into MCP Manager and managed through the proxy.
          </p>
        </div>
      </ToggleCard>
    </div>

    <div v-else-if="integrations && !installed.length" class="text-xs text-text-muted">
      No supported AI tools detected. Install one of the supported tools to get started.
    </div>

    <div v-else class="text-xs text-text-muted">Detecting installed tools...</div>
  </div>
</template>
