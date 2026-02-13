import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ServerConfig, ServerConfigInput } from '@/types/server';
import type { OAuthStatus } from '@/types/oauth';

export const useServersStore = defineStore('servers', () => {
  const servers = ref<ServerConfig[]>([]);
  const selectedServerId = ref<string | null>(null);
  const lastError = ref<Record<string, string>>({});
  const oauthStatus = ref<Record<string, OAuthStatus>>({});

  async function loadServers() {
    try {
      servers.value = await invoke<ServerConfig[]>('list_servers');
    } catch (e) {
      console.error('Failed to load servers:', e);
    }
  }

  async function addServer(input: ServerConfigInput) {
    try {
      const server = await invoke<ServerConfig>('add_server', { input });
      servers.value.push(server);
      selectedServerId.value = server.id;
      // Auto-connect
      connectServer(server.id);
    } catch (e) {
      console.error('Failed to add server:', e);
      throw e;
    }
  }

  async function updateServer(id: string, input: ServerConfigInput) {
    try {
      const updated = await invoke<ServerConfig>('update_server', { id, input });
      const idx = servers.value.findIndex(s => s.id === id);
      if (idx !== -1) servers.value[idx] = updated;
    } catch (e) {
      console.error('Failed to update server:', e);
      throw e;
    }
  }

  async function removeServer(id: string) {
    try {
      await invoke('remove_server', { id });
      servers.value = servers.value.filter(s => s.id !== id);
      if (selectedServerId.value === id) {
        selectedServerId.value = servers.value[0]?.id ?? null;
      }
    } catch (e) {
      console.error('Failed to remove server:', e);
    }
  }

  async function autoConnectServers() {
    for (const server of servers.value) {
      if (server.enabled && (!server.status || server.status === 'disconnected')) {
        connectServer(server.id);
      }
    }
  }

  async function connectServer(id: string) {
    clearError(id);
    const server = servers.value.find(s => s.id === id);
    if (server) server.status = 'connecting';
    try {
      await invoke('connect_server', { id });
    } catch (e) {
      const errorMsg = String(e);
      setError(id, errorMsg);
      if (server) server.status = 'error';
    }
  }

  async function disconnectServer(id: string) {
    try {
      await invoke('disconnect_server', { id });
    } catch (e) {
      console.error('Failed to disconnect server:', e);
    }
  }

  function selectServer(id: string) {
    selectedServerId.value = id;
  }

  function updateServerStatus(id: string, status: ServerConfig['status']) {
    const server = servers.value.find(s => s.id === id);
    if (server) server.status = status;
  }

  function setError(serverId: string, error: string) {
    lastError.value[serverId] = error;
  }

  function clearError(serverId: string) {
    delete lastError.value[serverId];
  }

  function setOAuthStatus(serverId: string, status: OAuthStatus) {
    oauthStatus.value[serverId] = status;
  }

  function clearOAuthStatus(serverId: string) {
    delete oauthStatus.value[serverId];
  }

  async function startOAuth(id: string) {
    oauthStatus.value[id] = 'discovering';
    clearError(id);
    try {
      await invoke('start_oauth_flow', { id });
    } catch (e) {
      oauthStatus.value[id] = 'error';
      setError(id, String(e));
    }
  }

  async function clearOAuthTokens(id: string) {
    try {
      await invoke('clear_oauth_tokens', { id });
      delete oauthStatus.value[id];
    } catch (e) {
      console.error('Failed to clear OAuth tokens:', e);
    }
  }

  return {
    servers,
    selectedServerId,
    lastError,
    oauthStatus,
    loadServers,
    autoConnectServers,
    addServer,
    updateServer,
    removeServer,
    connectServer,
    disconnectServer,
    selectServer,
    updateServerStatus,
    setError,
    clearError,
    setOAuthStatus,
    clearOAuthStatus,
    startOAuth,
    clearOAuthTokens,
  };
});
