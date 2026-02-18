<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useRegistryStore } from '@/stores/registry';
import { useServersStore } from '@/stores/servers';
import type { MarketplaceServerDetail } from '@/types/registry';
import MarkdownContent from '@/components/MarkdownContent.vue';
import MarketplaceInstallModal from '@/components/MarketplaceInstallModal.vue';

const route = useRoute();
const router = useRouter();
const registryStore = useRegistryStore();
const serversStore = useServersStore();

const serverId = computed(() => route.params.id as string);
const detail = ref<MarketplaceServerDetail | null>(null);
const loadingDetail = ref(true);
const loadError = ref<string | null>(null);

const readme = ref<string | null>(null);
const loadingReadme = ref(false);

const showInstallModal = ref(false);

const installed = computed(() =>
  serversStore.servers.some(s => s.registryName === serverId.value)
);

const githubOwner = computed(() => {
  const url = detail.value?.repositoryUrl;
  if (!url) return null;
  const match = url.match(/^https?:\/\/github\.com\/([^/]+)\//);
  return match ? match[1] : null;
});

function formatStars(count: number): string {
  if (count >= 1000) return `${(count / 1000).toFixed(1)}k`;
  return String(count);
}

function transportLabel(t: string): string {
  if (t === 'streamable-http') return 'HTTP';
  if (t === 'sse') return 'SSE';
  return t.toUpperCase();
}

onMounted(async () => {
  try {
    detail.value = await registryStore.fetchServerDetail(serverId.value);
  } catch (e) {
    loadError.value = String(e);
  } finally {
    loadingDetail.value = false;
  }

  // Fetch README separately
  if (detail.value?.repositoryUrl) {
    loadingReadme.value = true;
    try {
      readme.value = await invoke<string | null>('fetch_readme', {
        repositoryUrl: detail.value.repositoryUrl,
      });
    } catch {
      // README fetch is best-effort
    } finally {
      loadingReadme.value = false;
    }
  }
});
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-4">
      <!-- Back link -->
      <button
        class="mb-4 inline-flex items-center gap-1 text-xs text-text-muted transition-colors hover:text-text-secondary"
        @click="router.push('/add')"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
        </svg>
        Back to Marketplace
      </button>

      <!-- Loading -->
      <div v-if="loadingDetail" class="flex items-center justify-center py-16">
        <span class="text-xs text-text-muted">Loading server details...</span>
      </div>

      <!-- Error -->
      <div v-else-if="loadError" class="flex flex-col items-center justify-center gap-3 py-16">
        <p class="text-xs text-status-error">{{ loadError }}</p>
        <button
          class="rounded-md bg-surface-2 px-3 py-1.5 text-xs text-text-secondary transition-colors hover:bg-surface-3"
          @click="router.push('/add')"
        >
          Back to Marketplace
        </button>
      </div>

      <!-- Content -->
      <template v-else-if="detail">
        <!-- Header -->
        <div class="mb-6 flex items-start justify-between gap-4">
          <div class="min-w-0">
            <h1 class="text-lg font-semibold text-text-primary">{{ detail.name }}</h1>
            <div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-text-muted">
              <span v-if="githubOwner">@{{ githubOwner }}</span>
              <span v-if="detail.stars != null" class="flex items-center gap-0.5">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
                {{ formatStars(detail.stars) }}
              </span>
              <span v-if="detail.version">v{{ detail.version }}</span>
            </div>
            <div class="mt-2 flex flex-wrap items-center gap-1.5">
              <span
                v-for="t in detail.transportTypes"
                :key="t"
                class="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-text-muted"
              >
                {{ transportLabel(t) }}
              </span>
              <span
                v-if="detail.runtime"
                class="rounded bg-surface-3 px-1.5 py-0.5 font-mono text-[10px] text-text-muted"
              >
                {{ detail.runtime }}
              </span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex shrink-0 items-center gap-2">
            <button
              v-if="detail.repositoryUrl"
              class="inline-flex items-center gap-1 rounded-md border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-border-active hover:text-text-primary"
              @click="openUrl(detail.repositoryUrl!)"
            >
              View Source
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                <path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
                <path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
              </svg>
            </button>
            <span
              v-if="installed"
              class="inline-flex items-center rounded-md bg-status-connected/10 px-3 py-1.5 text-xs font-medium text-status-connected"
            >
              Installed
            </span>
            <button
              v-else
              class="rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
              @click="showInstallModal = true"
            >
              Install
            </button>
          </div>
        </div>

        <!-- Description -->
        <div v-if="detail.description" class="mb-6">
          <MarkdownContent :content="detail.description" />
        </div>

        <!-- README -->
        <div v-if="loadingReadme || readme" class="border-t border-border pt-6">
          <h2 class="mb-4 text-sm font-semibold text-text-primary">README</h2>
          <div v-if="loadingReadme" class="flex items-center justify-center py-8">
            <span class="text-xs text-text-muted">Loading README...</span>
          </div>
          <MarkdownContent v-else-if="readme" :content="readme" />
        </div>
      </template>
    </div>

    <!-- Install modal -->
    <MarketplaceInstallModal
      v-if="showInstallModal"
      :server-id="serverId"
      @close="showInstallModal = false"
    />
  </div>
</template>
