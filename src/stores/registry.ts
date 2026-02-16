import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type {
  RegistryServerSummary,
  RegistrySearchResult,
  MarketplaceServerDetail,
  RuntimeDeps,
} from '@/types/registry';
import type { ServerConfig } from '@/types/server';

export const useRegistryStore = defineStore('registry', () => {
  const servers = ref<RegistryServerSummary[]>([]);
  const hasMore = ref(false);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref('');
  const runtimeDeps = ref<RuntimeDeps | null>(null);
  const installingServer = ref<string | null>(null);

  async function search(query: string, reset = true) {
    if (reset) {
      servers.value = [];
      hasMore.value = false;
    }
    searchQuery.value = query;
    loading.value = true;
    error.value = null;

    try {
      const result = await invoke<RegistrySearchResult>('search_registry', {
        search: query || null,
        offset: reset ? 0 : servers.value.length,
        limit: 40,
      });
      if (reset) {
        servers.value = result.servers;
      } else {
        servers.value.push(...result.servers);
      }
      hasMore.value = result.hasMore;
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadMore() {
    if (!hasMore.value || loading.value) return;
    await search(searchQuery.value, false);
  }

  async function fetchServerDetail(id: string): Promise<MarketplaceServerDetail> {
    return await invoke<MarketplaceServerDetail>('get_registry_server', { id });
  }

  async function checkDeps() {
    try {
      runtimeDeps.value = await invoke<RuntimeDeps>('check_runtime_deps');
    } catch (e) {
      console.error('Failed to check runtime deps:', e);
    }
  }

  async function installServer(
    id: string,
    envVars?: Record<string, string>,
  ): Promise<ServerConfig> {
    installingServer.value = id;
    try {
      const server = await invoke<ServerConfig>('install_registry_server', {
        id,
        envVars: envVars || null,
      });
      // Mark as installed in local state
      const idx = servers.value.findIndex(s => s.id === id);
      if (idx !== -1) {
        servers.value[idx].installed = true;
      }
      return server;
    } finally {
      installingServer.value = null;
    }
  }

  return {
    servers,
    hasMore,
    loading,
    error,
    searchQuery,
    runtimeDeps,
    installingServer,
    search,
    loadMore,
    fetchServerDetail,
    checkDeps,
    installServer,
  };
});
