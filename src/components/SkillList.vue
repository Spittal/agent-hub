<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useSkillsStore } from '@/stores/skills';
import { useServersStore } from '@/stores/servers';
import { storeToRefs } from 'pinia';

const router = useRouter();
const store = useSkillsStore();
const serversStore = useServersStore();
const { installedSkills, selectedSkillId } = storeToRefs(store);

function onSelect(id: string) {
  store.selectSkill(id);
  serversStore.selectedServerId = null;
  router.push('/skills');
}
</script>

<template>
  <div>
    <div
      v-for="skill in installedSkills"
      :key="skill.id"
      class="flex cursor-pointer items-center gap-2 border-b border-border/50 px-3 py-2 transition-colors hover:bg-surface-2"
      :class="selectedSkillId === skill.id ? 'bg-surface-2' : ''"
      @click="onSelect(skill.id)"
    >
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        :class="skill.enabled ? 'bg-status-connected' : 'bg-surface-3'"
      />
      <span class="truncate text-xs" :class="skill.enabled ? '' : 'text-text-muted'">{{ skill.name }}</span>
      <span
        v-if="skill.managedBy"
        class="ml-auto shrink-0 rounded bg-status-connected/10 px-1.5 py-0.5 text-[9px] font-medium text-status-connected"
      >
        {{ skill.managedBy.charAt(0).toUpperCase() + skill.managedBy.slice(1) }}
      </span>
      <span
        v-else-if="skill.managed"
        class="ml-auto shrink-0 rounded bg-status-connected/10 px-1.5 py-0.5 text-[9px] font-medium text-status-connected"
      >
        Managed
      </span>
    </div>

    <div
      v-if="installedSkills.length === 0"
      class="px-3 py-6 text-center text-xs text-text-muted"
    >
      No skills found
    </div>
  </div>
</template>
