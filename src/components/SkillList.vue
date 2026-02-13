<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useSkillsStore } from '@/stores/skills';
import { useServersStore } from '@/stores/servers';
import { storeToRefs } from 'pinia';

const router = useRouter();
const store = useSkillsStore();
const serversStore = useServersStore();
const { skills, selectedSkillId } = storeToRefs(store);

function onSelect(id: string) {
  store.selectSkill(id);
  serversStore.selectedServerId = null;
  router.push('/skills');
}
</script>

<template>
  <div>
    <div
      v-for="skill in skills"
      :key="skill.id"
      class="flex cursor-pointer items-center gap-2 border-b border-border/50 px-3 py-2 transition-colors hover:bg-surface-2"
      :class="selectedSkillId === skill.id ? 'bg-surface-2' : ''"
      @click="onSelect(skill.id)"
    >
      <span class="truncate text-xs">{{ skill.name }}</span>
      <span class="ml-auto shrink-0 text-[10px] text-text-muted">{{ skill.pluginName }}</span>
    </div>
    <div
      v-if="skills.length === 0"
      class="px-3 py-6 text-center text-xs text-text-muted"
    >
      No skills installed
    </div>
  </div>
</template>
