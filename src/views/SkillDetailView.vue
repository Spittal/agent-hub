<script setup lang="ts">
import { useSkillsStore } from '@/stores/skills';
import { storeToRefs } from 'pinia';

const store = useSkillsStore();
const { skillDetail, selectedSkillId } = storeToRefs(store);
</script>

<template>
  <div v-if="skillDetail" class="flex h-full flex-col">
    <!-- Header -->
    <header class="flex items-center gap-3 border-b border-border px-4 py-3">
      <h1 class="text-sm font-medium">{{ skillDetail.info.name }}</h1>
      <span class="font-mono text-xs text-text-muted">{{ skillDetail.info.pluginName }}</span>
      <span class="rounded bg-accent/15 px-1.5 py-0.5 text-[10px] font-medium text-accent">Claude</span>
    </header>

    <!-- Content -->
    <div class="min-h-0 flex-1 overflow-y-auto p-4">
      <!-- Metadata -->
      <section class="mb-6">
        <h2 class="mb-2 font-mono text-xs font-medium tracking-wide text-text-muted uppercase">Info</h2>
        <div class="rounded border border-border bg-surface-1 p-3 font-mono text-xs text-text-secondary">
          <div v-if="skillDetail.info.description" class="mb-2">
            <span class="text-text-muted">description:</span>
            <span class="ml-1">{{ skillDetail.info.description }}</span>
          </div>
          <div>
            <span class="text-text-muted">plugin:</span>
            <span class="ml-1">{{ skillDetail.info.pluginName }}</span>
            <span v-if="skillDetail.info.pluginAuthor" class="text-text-muted"> by {{ skillDetail.info.pluginAuthor }}</span>
          </div>
          <div v-if="skillDetail.info.version">
            <span class="text-text-muted">version:</span>
            <span class="ml-1">{{ skillDetail.info.version }}</span>
          </div>
          <div v-if="skillDetail.info.tools">
            <span class="text-text-muted">tools:</span>
            <span class="ml-1">{{ skillDetail.info.tools }}</span>
          </div>
        </div>
      </section>

      <!-- Markdown content -->
      <section>
        <h2 class="mb-2 font-mono text-xs font-medium tracking-wide text-text-muted uppercase">Content</h2>
        <pre class="whitespace-pre-wrap break-words rounded border border-border bg-surface-1 p-3 font-mono text-xs leading-relaxed text-text-secondary">{{ skillDetail.content }}</pre>
      </section>
    </div>
  </div>

  <div v-else-if="!selectedSkillId" class="flex h-full items-center justify-center text-text-muted">
    <div class="text-center">
      <p class="mb-1 text-sm">No skill selected</p>
      <p class="text-xs">Select a skill from the sidebar to view its details.</p>
    </div>
  </div>
</template>
