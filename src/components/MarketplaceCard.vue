<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { RegistryServerSummary } from '@/types/registry';
import MarkdownContent from '@/components/MarkdownContent.vue';

const props = defineProps<{
  server: RegistryServerSummary;
  installing: boolean;
}>();

const emit = defineEmits<{
  install: [];
}>();

const router = useRouter();

const installable = computed(() => props.server.transportTypes.length > 0);

const githubOwner = computed(() => {
  const url = props.server.repositoryUrl;
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

function openExternal(url: string, event: Event) {
  event.stopPropagation();
  openUrl(url);
}

function navigateToDetail() {
  router.push(`/marketplace/${props.server.id}`);
}

function onInstall(event: Event) {
  event.stopPropagation();
  emit('install');
}

function onManualSetup(event: Event) {
  event.stopPropagation();
  if (props.server.repositoryUrl) {
    openUrl(props.server.repositoryUrl);
  }
}
</script>

<template>
  <div
    class="flex cursor-pointer items-start gap-3 rounded-lg border border-border bg-surface-1 px-4 py-3 transition-colors hover:border-border-active"
    @click="navigateToDetail"
  >
    <!-- Icon -->
    <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-surface-3 text-text-muted">
      <img
        v-if="server.iconUrl"
        :src="server.iconUrl"
        :alt="server.displayName"
        class="h-9 w-9 rounded-md object-cover"
      />
      <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M2 5a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2V5zm14 1a1 1 0 11-2 0 1 1 0 012 0zM2 13a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2v-2zm14 1a1 1 0 11-2 0 1 1 0 012 0z" clip-rule="evenodd" />
      </svg>
    </div>

    <!-- Content -->
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span class="truncate text-sm font-medium text-text-primary">{{ server.displayName }}</span>
        <span
          v-if="server.registryType"
          class="shrink-0 rounded bg-surface-3 px-1.5 py-0.5 font-mono text-[10px] text-text-muted"
        >
          {{ server.registryType }}
        </span>
        <span
          v-for="t in server.transportTypes"
          :key="t"
          class="shrink-0 rounded bg-surface-2 px-1.5 py-0.5 font-mono text-[10px] text-text-muted"
        >
          {{ transportLabel(t) }}
        </span>
      </div>
      <MarkdownContent
        v-if="server.description"
        :content="server.description"
        inline
        class="mt-0.5 line-clamp-2 text-xs leading-relaxed text-text-secondary"
      />
      <div class="mt-1 flex items-center gap-2 text-[11px] text-text-muted">
        <span v-if="githubOwner">@{{ githubOwner }}</span>
        <span v-if="server.stars != null" class="flex items-center gap-0.5">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          {{ formatStars(server.stars) }}
        </span>
        <span v-if="server.version">v{{ server.version }}</span>
        <button
          v-if="server.repositoryUrl"
          class="hover:text-text-secondary"
          @click="openExternal(server.repositoryUrl!, $event)"
        >source</button>
      </div>
    </div>

    <!-- Action -->
    <div class="shrink-0 pt-0.5">
      <span
        v-if="server.installed"
        class="inline-flex items-center rounded-md bg-status-connected/10 px-2.5 py-1 text-xs font-medium text-status-connected"
        @click.stop
      >
        Installed
      </span>
      <button
        v-else-if="installing"
        disabled
        class="rounded-md bg-surface-3 px-3 py-1 text-xs text-text-muted"
        @click.stop
      >
        Installing...
      </button>
      <button
        v-else-if="!installable && server.repositoryUrl"
        class="inline-flex items-center gap-1 rounded-md border border-border px-2.5 py-1 text-[11px] text-text-muted transition-colors hover:border-border-active hover:text-text-secondary"
        title="This server requires manual setup — click to view instructions"
        @click="onManualSetup($event)"
      >
        Manual Setup
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
          <path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
          <path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
        </svg>
      </button>
      <span
        v-else-if="!installable"
        class="inline-flex items-center rounded-md px-2.5 py-1 text-[11px] text-text-muted"
        @click.stop
      >
        Manual Setup
      </span>
      <button
        v-else
        class="rounded-md bg-accent px-3 py-1 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
        @click="onInstall($event)"
      >
        Install
      </button>
    </div>
  </div>
</template>
