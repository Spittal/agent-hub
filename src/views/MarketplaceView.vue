<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useRegistryStore } from '@/stores/registry';
import { useServersStore } from '@/stores/servers';
import type { RegistryServerSummary } from '@/types/registry';
import MarketplaceCard from '@/components/MarketplaceCard.vue';
import MarketplaceInstallModal from '@/components/MarketplaceInstallModal.vue';

const router = useRouter();
const store = useRegistryStore();
const serversStore = useServersStore();

const searchInput = ref('');
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const modalServerId = ref<string | null>(null);

onMounted(() => {
  store.search('');
  store.checkDeps();
});

function onSearchInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    store.search(searchInput.value);
  }, 300);
}

async function handleInstall(server: RegistryServerSummary) {
  if (server.installed) return;

  // One-click install for servers with no required config
  if (!server.requiresConfig) {
    try {
      const created = await store.installServer(server.id);
      await serversStore.loadServers();
      serversStore.connectServer(created.id);
      serversStore.selectServer(created.id);
      router.push('/');
    } catch (e) {
      console.error('Quick install failed:', e);
      // Fall back to modal
      modalServerId.value = server.id;
    }
    return;
  }

  modalServerId.value = server.id;
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden px-6 py-4">
    <!-- Header -->
    <div class="mb-4">
      <h1 class="text-base font-semibold text-text-primary">Marketplace</h1>
    </div>

    <!-- Search -->
    <div class="relative mb-4">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-text-muted"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
      </svg>
      <input
        v-model="searchInput"
        type="text"
        placeholder="Search MCP servers..."
        class="w-full rounded-lg border border-border bg-surface-1 py-2 pl-9 pr-3 text-xs text-text-primary placeholder-text-muted outline-none transition-colors focus:border-accent"
        @input="onSearchInput"
      />
    </div>

    <!-- Content -->
    <div class="min-h-0 flex-1 overflow-y-auto">
      <!-- Loading (initial) -->
      <div v-if="store.loading && store.servers.length === 0" class="flex items-center justify-center py-12">
        <span class="text-xs text-text-muted">Loading servers...</span>
      </div>

      <!-- Error -->
      <div v-else-if="store.error && store.servers.length === 0" class="flex flex-col items-center justify-center gap-3 py-12">
        <p class="text-xs text-status-error">{{ store.error }}</p>
        <button
          class="rounded-md bg-surface-2 px-3 py-1.5 text-xs text-text-secondary transition-colors hover:bg-surface-3"
          @click="store.search(searchInput)"
        >
          Retry
        </button>
      </div>

      <!-- Empty -->
      <div v-else-if="!store.loading && store.servers.length === 0" class="flex items-center justify-center py-12">
        <span class="text-xs text-text-muted">No servers found</span>
      </div>

      <!-- Server grid -->
      <div v-else class="grid gap-2 pb-4" style="grid-template-columns: repeat(auto-fill, minmax(380px, 1fr))">
        <MarketplaceCard
          v-for="server in store.servers"
          :key="server.id"
          :server="server"
          :installing="store.installingServer === server.id"
          @install="handleInstall(server)"
        />
      </div>

      <!-- Load more -->
      <div v-if="store.hasMore && !store.loading" class="flex justify-center pb-4 pt-2">
        <button
          class="rounded-md bg-surface-2 px-4 py-1.5 text-xs text-text-secondary transition-colors hover:bg-surface-3"
          @click="store.loadMore()"
        >
          Load more
        </button>
      </div>

      <!-- Loading more -->
      <div v-if="store.loading && store.servers.length > 0" class="flex justify-center pb-4 pt-2">
        <span class="text-xs text-text-muted">Loading more...</span>
      </div>
    </div>

    <!-- Install modal -->
    <MarketplaceInstallModal
      v-if="modalServerId"
      :server-id="modalServerId"
      @close="modalServerId = null"
    />
  </div>
</template>
