<script setup lang="ts">
import { ref, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { useProfilesStore } from '@/stores/profiles';

const profilesStore = useProfilesStore();
const { profiles } = storeToRefs(profilesStore);

const expanded = ref(false);

const profileCount = computed(() => profiles.value.length);
</script>

<template>
  <div v-if="profileCount > 0" class="border-b border-border">
    <button
      class="flex w-full items-center gap-2 px-3 py-2 text-xs text-text-muted transition-colors hover:bg-surface-2 hover:text-text-secondary"
      @click="expanded = !expanded"
    >
      <span
        class="inline-block text-[10px] leading-none transition-transform"
        :class="expanded ? '' : '-rotate-90'"
      >&#9662;</span>
      <span>{{ profileCount }} {{ profileCount === 1 ? 'profile' : 'profiles' }}</span>
    </button>
    <div v-if="expanded" class="pb-1">
      <router-link
        v-for="profile in profiles"
        :key="profile.id"
        :to="{ path: '/settings', query: { tab: 'profiles', profile: profile.id } }"
        class="block px-3 py-1.5 text-xs text-text-muted transition-colors hover:bg-surface-2 hover:text-text-secondary"
        active-class="bg-surface-2 text-text-secondary"
      >
        <div class="font-medium">{{ profile.name }}</div>
        <div
          v-for="dir in profile.directoryPaths"
          :key="dir"
          class="truncate pl-3 text-[11px]"
          :title="dir"
        >
          {{ dir }}
        </div>
        <div
          v-if="profile.directoryPaths.length === 0"
          class="pl-3 text-[11px] italic"
        >
          No directories
        </div>
      </router-link>
    </div>
  </div>
</template>
