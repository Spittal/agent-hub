<script setup lang="ts">
import { ref } from 'vue';
import { type RouteLocationRaw } from 'vue-router';
import { useServersStore } from '@/stores/servers';
import { storeToRefs } from 'pinia';
import { statusColor } from '@/composables/useServerStatus';

const store = useServersStore();
const { servers } = storeToRefs(store);

function managedByLabel(managedBy: string): string {
  if (managedBy === 'memory') return 'Memory';
  return managedBy.charAt(0).toUpperCase() + managedBy.slice(1);
}

function managedByRoute(managedBy: string): RouteLocationRaw {
  return { path: '/settings', query: { tab: managedBy } };
}

const contextMenuId = ref<string | null>(null);
const contextMenuPos = ref({ x: 0, y: 0 });

function onContextMenu(e: MouseEvent, id: string) {
  e.preventDefault();
  contextMenuId.value = id;
  contextMenuPos.value = { x: e.clientX, y: e.clientY };
  window.addEventListener('click', closeContextMenu, { once: true });
}

function closeContextMenu() {
  contextMenuId.value = null;
}

async function toggleServer(id: string) {
  closeContextMenu();
  const server = servers.value.find(s => s.id === id);
  if (!server) return;
  const newEnabled = !server.enabled;
  if (!newEnabled && server.status === 'connected') {
    await store.disconnectServer(id);
  }
  await store.updateServer(id, {
    name: server.name,
    transport: server.transport,
    enabled: newEnabled,
    command: server.command,
    args: server.args,
    env: server.env,
    url: server.url,
    headers: server.headers,
    tags: server.tags,
  });
  if (newEnabled) {
    store.connectServer(id);
  }
}

async function deleteServer(id: string) {
  closeContextMenu();
  const server = servers.value.find(s => s.id === id);
  if (server?.status === 'connected') {
    await store.disconnectServer(id);
  }
  await store.removeServer(id);
}
</script>

<template>
  <div>
    <router-link
      v-for="server in servers"
      :key="server.id"
      :to="'/servers/' + server.id"
      class="group flex items-center gap-2 border-b border-border/50 px-3 py-2 transition-colors hover:bg-surface-2"
      :class="!server.enabled ? 'opacity-50' : ''"
      active-class="bg-surface-2"
      @contextmenu="onContextMenu($event, server.id)"
    >
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        :class="statusColor(server.status ?? 'disconnected', server.enabled)"
      />
      <span class="truncate text-xs">{{ server.name }}</span>
      <router-link
        v-if="server.managedBy"
        :to="managedByRoute(server.managedBy)"
        class="ml-auto shrink-0 rounded bg-status-connected/10 px-1.5 py-0.5 text-[9px] font-medium text-status-connected transition-colors hover:bg-status-connected/20"
        @click.stop
      >{{ managedByLabel(server.managedBy) }}</router-link>
      <span v-if="!server.enabled" class="ml-auto text-[10px] text-text-muted">off</span>
    </router-link>
    <div
      v-if="servers.length === 0"
      class="px-3 py-6 text-center text-xs text-text-muted"
    >
      No servers configured
    </div>

    <!-- Context menu -->
    <Teleport to="body">
      <div
        v-if="contextMenuId"
        class="fixed z-50 min-w-[140px] rounded border border-border bg-surface-1 py-1 shadow-lg"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <button
          class="w-full px-3 py-1.5 text-left text-xs text-text-secondary transition-colors hover:bg-surface-2"
          @click="toggleServer(contextMenuId!)"
        >
          {{ servers.find(s => s.id === contextMenuId)?.enabled ? 'Disable' : 'Enable' }}
        </button>
        <button
          v-if="!servers.find(s => s.id === contextMenuId)?.managedBy"
          class="w-full px-3 py-1.5 text-left text-xs text-status-error transition-colors hover:bg-status-error/10"
          @click="deleteServer(contextMenuId!)"
        >
          Delete
        </button>
      </div>
    </Teleport>
  </div>
</template>
