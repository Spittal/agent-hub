<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSkillsStore } from '@/stores/skills';
import type { SkillToolInfo } from '@/types/skill';

const skillsStore = useSkillsStore();
const integrations = ref<SkillToolInfo[] | null>(null);
const error = ref<string | null>(null);
const togglingId = ref<string | null>(null);

const installedTools = computed(() =>
  integrations.value?.filter(t => t.installed) ?? []
);

const notInstalledTools = computed(() =>
  integrations.value?.filter(t => !t.installed) ?? []
);

async function fetchIntegrations() {
  try {
    integrations.value = await invoke<SkillToolInfo[]>('detect_skill_integrations');
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

async function enable(tool: SkillToolInfo) {
  togglingId.value = tool.id;
  try {
    await invoke('enable_skill_integration', { id: tool.id });
    await skillsStore.loadInstalled();
    await fetchIntegrations();
  } catch (e) {
    error.value = String(e);
  } finally {
    togglingId.value = null;
  }
}

async function disable(tool: SkillToolInfo) {
  togglingId.value = tool.id;
  try {
    await invoke('disable_skill_integration', { id: tool.id });
    await fetchIntegrations();
  } catch (e) {
    error.value = String(e);
  } finally {
    togglingId.value = null;
  }
}

function isBusy(tool: SkillToolInfo): boolean {
  return togglingId.value === tool.id;
}

onMounted(() => {
  fetchIntegrations();
});
</script>

<template>
  <div>
    <h2 class="mb-1 text-xs font-medium text-text-primary">Managed Skills</h2>
    <p class="mb-4 text-xs text-text-secondary">
      Choose which AI tools receive SKILL.md files when you install skills from the marketplace. Existing skills on disk will be imported when you enable a tool.
    </p>

    <div v-if="error" class="mb-3 rounded bg-status-error/10 px-3 py-2 text-xs text-status-error">
      {{ error }}
    </div>

    <div v-if="!integrations" class="text-xs text-text-muted">Detecting tools...</div>

    <template v-if="integrations">
      <!-- Installed tools -->
      <div v-if="installedTools.length" class="space-y-5">
        <div v-for="tool in installedTools" :key="tool.id">
          <h3 class="mb-2 font-mono text-[10px] font-medium tracking-wide text-text-muted uppercase">
            {{ tool.name }}
          </h3>
          <div class="rounded border border-border bg-surface-1">
            <div class="flex items-center justify-between px-3 py-2.5">
              <div class="min-w-0">
                <div class="mt-0.5 truncate text-[10px] text-text-muted">{{ tool.skillsPath }}</div>
              </div>
              <div class="ml-3 flex shrink-0 items-center gap-2">
                <!-- Managed badge -->
                <span
                  v-if="tool.enabled"
                  class="inline-flex items-center gap-1 rounded bg-status-connected/10 px-2 py-1 text-[11px] font-medium text-status-connected"
                >
                  <span class="h-1.5 w-1.5 rounded-full bg-status-connected" />
                  Managed
                </span>
                <!-- Import & Enable: shown when skills exist on disk and tool is not yet enabled -->
                <button
                  v-else-if="tool.existingSkills.length"
                  class="rounded bg-accent px-3 py-1 text-[11px] font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
                  :disabled="isBusy(tool)"
                  @click="enable(tool)"
                >
                  {{ isBusy(tool) ? 'Importing...' : 'Import & Enable' }}
                </button>
                <!-- Enable (no existing skills) -->
                <button
                  v-else-if="!tool.enabled"
                  class="rounded bg-accent px-3 py-1 text-[11px] font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
                  :disabled="isBusy(tool)"
                  @click="enable(tool)"
                >
                  {{ isBusy(tool) ? 'Enabling...' : 'Enable' }}
                </button>
                <!-- Disable -->
                <button
                  v-if="tool.enabled"
                  class="rounded bg-surface-3 px-3 py-1 text-[11px] text-text-secondary transition-colors hover:bg-surface-2 disabled:opacity-50"
                  :disabled="isBusy(tool)"
                  @click="disable(tool)"
                >
                  Disable
                </button>
              </div>
            </div>

            <!-- Existing skills list (only when not enabled) -->
            <div v-if="!tool.enabled && tool.existingSkills.length" class="border-t border-border/50 px-3 py-2">
              <div class="space-y-1">
                <div
                  v-for="skill in tool.existingSkills"
                  :key="skill.skillId"
                  class="rounded bg-surface-0 px-2 py-1.5"
                >
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-[11px] font-medium text-text-secondary">{{ skill.name }}</span>
                    <span v-if="skill.description" class="truncate text-[10px] text-text-muted">{{ skill.description }}</span>
                  </div>
                </div>
              </div>
            </div>
            <div v-else-if="!tool.enabled" class="border-t border-border/50 px-3 py-2">
              <span class="text-[10px] text-text-muted">No existing skills found</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Not installed tools (grayed out) -->
      <div v-if="notInstalledTools.length" class="mt-4">
        <h3 class="mb-2 font-mono text-[10px] font-medium tracking-wide text-text-muted uppercase">
          Not Detected
        </h3>
        <div class="space-y-2">
          <div
            v-for="tool in notInstalledTools"
            :key="tool.id"
            class="rounded border border-border/50 bg-surface-1 opacity-50"
          >
            <div class="flex items-center justify-between px-3 py-2.5">
              <div class="min-w-0">
                <div class="text-xs text-text-muted">{{ tool.name }}</div>
                <div class="mt-0.5 truncate text-[10px] text-text-muted">{{ tool.skillsPath }}</div>
              </div>
              <span class="text-[10px] text-text-muted">Not installed</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="!installedTools.length && !notInstalledTools.length" class="text-xs text-text-muted">
        No supported AI tools detected.
      </div>
    </template>
  </div>
</template>
