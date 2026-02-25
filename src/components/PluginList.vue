<script setup lang="ts">
import { usePluginsStore } from '@/stores/plugins';
import { storeToRefs } from 'pinia';

const store = usePluginsStore();
const { installedPlugins } = storeToRefs(store);
</script>

<template>
  <div>
    <router-link
      v-for="plugin in installedPlugins"
      :key="plugin.id"
      :to="'/plugins/' + plugin.id"
      class="flex items-center gap-2 border-b border-border/50 px-3 py-2 transition-colors hover:bg-surface-2"
      active-class="bg-surface-2"
    >
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        :class="plugin.enabled ? 'bg-status-connected' : 'bg-surface-3'"
      />
      <span class="truncate text-xs" :class="plugin.enabled ? '' : 'text-text-muted'">{{ plugin.name }}</span>
    </router-link>
    <div
      v-if="installedPlugins.length === 0"
      class="px-3 py-6 text-center text-xs text-text-muted"
    >
      No plugins installed
    </div>
  </div>
</template>
