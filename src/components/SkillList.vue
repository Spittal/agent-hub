<script setup lang="ts">
import { type RouteLocationRaw } from 'vue-router';
import { useSkillsStore } from '@/stores/skills';
import { storeToRefs } from 'pinia';

const store = useSkillsStore();
const { installedSkills } = storeToRefs(store);

function managedByRoute(managedBy: string): RouteLocationRaw {
  return { path: '/settings', query: { tab: managedBy } };
}
</script>

<template>
  <div>
    <router-link
      v-for="skill in installedSkills"
      :key="skill.id"
      :to="'/skills/' + skill.id"
      class="flex items-center gap-2 border-b border-border/50 px-3 py-2 transition-colors hover:bg-surface-2"
      active-class="bg-surface-2"
    >
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        :class="skill.enabled ? 'bg-status-connected' : 'bg-surface-3'"
      />
      <span class="truncate text-xs" :class="skill.enabled ? '' : 'text-text-muted'">{{ skill.name }}</span>
      <router-link
        v-if="skill.managedBy"
        :to="managedByRoute(skill.managedBy)"
        class="ml-auto shrink-0 rounded bg-status-connected/10 px-1.5 py-0.5 text-[9px] font-medium text-status-connected transition-colors hover:bg-status-connected/20"
        @click.stop
      >
        {{ skill.managedBy.charAt(0).toUpperCase() + skill.managedBy.slice(1) }}
      </router-link>
      <span
        v-else-if="skill.managed"
        class="ml-auto shrink-0 rounded bg-status-connected/10 px-1.5 py-0.5 text-[9px] font-medium text-status-connected"
      >
        Managed
      </span>
    </router-link>

    <div
      v-if="installedSkills.length === 0"
      class="px-3 py-6 text-center text-xs text-text-muted"
    >
      No skills found
    </div>
  </div>
</template>
