<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { useProfilesStore } from '@/stores/profiles';
import { useServersStore } from '@/stores/servers';
import { useSkillsStore } from '@/stores/skills';
import { usePluginsStore } from '@/stores/plugins';
import type { Profile, UpdateProfileInput } from '@/types/profile';
import type { AiToolInfo } from '@/types/integration';

const route = useRoute();
const profilesStore = useProfilesStore();
const serversStore = useServersStore();
const skillsStore = useSkillsStore();
const pluginsStore = usePluginsStore();

const integrations = ref<AiToolInfo[]>([]);
const expandedId = ref<string | null>((route.query.profile as string) || null);
const newProfileName = ref('');
const creating = ref(false);
const saving = ref<string | null>(null);
const deleting = ref<string | null>(null);
const confirmingDeleteId = ref<string | null>(null);
const error = ref<string | null>(null);
const importResult = ref<string | null>(null);

async function fetchIntegrations() {
  try {
    const all = await invoke<AiToolInfo[]>('detect_integrations');
    integrations.value = all.filter(t => t.installed);
  } catch (e) {
    console.error('Failed to detect integrations:', e);
  }
}

async function createProfile() {
  if (!newProfileName.value.trim()) return;
  creating.value = true;
  error.value = null;
  try {
    const profile = await profilesStore.createProfile({
      name: newProfileName.value.trim(),
    });
    newProfileName.value = '';
    expandedId.value = profile.id;
  } catch (e) {
    error.value = String(e);
  } finally {
    creating.value = false;
  }
}

async function saveProfile(profile: Profile, updates: UpdateProfileInput) {
  saving.value = profile.id;
  error.value = null;
  try {
    await profilesStore.updateProfile(profile.id, updates);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = null;
  }
}

async function deleteProfile(id: string) {
  deleting.value = id;
  error.value = null;
  try {
    await profilesStore.deleteProfile(id);
    if (expandedId.value === id) expandedId.value = null;
    confirmingDeleteId.value = null;
  } catch (e) {
    error.value = String(e);
  } finally {
    deleting.value = null;
  }
}

function toggleExpanded(id: string) {
  expandedId.value = expandedId.value === id ? null : id;
}

// --- Checkbox toggles ---

function toggleServer(profile: Profile, serverId: string) {
  const ids = [...profile.serverIds];
  const idx = ids.indexOf(serverId);
  if (idx === -1) ids.push(serverId);
  else ids.splice(idx, 1);
  saveProfile(profile, { serverIds: ids });
}

function toggleSkill(profile: Profile, skillId: string) {
  const ids = [...profile.skillIds];
  const idx = ids.indexOf(skillId);
  if (idx === -1) ids.push(skillId);
  else ids.splice(idx, 1);
  saveProfile(profile, { skillIds: ids });
}

function togglePlugin(profile: Profile, pluginId: string) {
  const ids = [...profile.pluginIds];
  const idx = ids.indexOf(pluginId);
  if (idx === -1) ids.push(pluginId);
  else ids.splice(idx, 1);
  saveProfile(profile, { pluginIds: ids });
}

function toggleIntegration(profile: Profile, integrationId: string) {
  const ids = [...profile.integrationIds];
  const idx = ids.indexOf(integrationId);
  if (idx === -1) ids.push(integrationId);
  else ids.splice(idx, 1);
  saveProfile(profile, { integrationIds: ids });
}

function toggleMemory(profile: Profile) {
  saveProfile(profile, {
    features: { ...profile.features, memory: !profile.features.memory },
  });
}

function toggleDiscovery(profile: Profile) {
  saveProfile(profile, {
    features: { ...profile.features, discovery: !profile.features.discovery },
  });
}

function updateMemoryDb(profile: Profile, value: string) {
  const db = Math.max(0, Math.min(15, parseInt(value) || 0));
  saveProfile(profile, {
    features: { ...profile.features, memoryDb: db },
  });
}

function updateName(profile: Profile, name: string) {
  if (name.trim() && name.trim() !== profile.name) {
    saveProfile(profile, { name: name.trim() });
  }
}

// --- Directory management ---

async function addDirectory(profile: Profile) {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Select project directory',
    });
    if (selected && typeof selected === 'string') {
      await profilesStore.addDirectory(profile.id, selected);
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function removeDirectory(profile: Profile, dir: string) {
  try {
    await profilesStore.removeDirectory(profile.id, dir);
  } catch (e) {
    error.value = String(e);
  }
}

// --- Export / Import ---

async function handleExport(profileId: string) {
  try {
    // Get export data first for the suggested filename
    const data = await profilesStore.exportProfile(profileId);
    const defaultName = `${data.name.toLowerCase().replace(/\s+/g, '-')}-profile.json`;

    const selected = await saveDialog({
      defaultPath: defaultName,
      filters: [{ name: 'JSON', extensions: ['json'] }],
      title: 'Export profile',
    });
    if (!selected) return;

    await profilesStore.exportProfileToFile(profileId, selected);
  } catch (e) {
    error.value = String(e);
  }
}

async function handleImport() {
  try {
    const selected = await openDialog({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      multiple: false,
      title: 'Import profile',
    });
    if (!selected || typeof selected !== 'string') return;

    const result = await profilesStore.importProfileFromFile(selected);

    const parts = [];
    if (result.matchedServers > 0) parts.push(`${result.matchedServers} servers matched`);
    if (result.createdServers > 0) parts.push(`${result.createdServers} servers created`);
    if (result.matchedSkills > 0) parts.push(`${result.matchedSkills} skills matched`);
    if (result.unmatchedSkills.length > 0)
      parts.push(`${result.unmatchedSkills.length} skills not found`);
    importResult.value = `Imported "${result.profile.name}": ${parts.join(', ')}`;

    expandedId.value = result.profile.id;
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(async () => {
  await profilesStore.loadProfiles();
  await fetchIntegrations();
});
</script>

<template>
  <div class="space-y-4">
    <div>
      <h2 class="text-xs font-medium text-text-primary">Profiles</h2>
      <p class="mt-1 text-[11px] text-text-muted leading-relaxed">
        Profiles control which servers, skills, and plugins are visible to each AI tool.
        Switch profiles to change context (e.g. Work vs Personal), or map directories
        for automatic per-project isolation.
      </p>
    </div>

    <!-- Error display -->
    <div v-if="error" class="rounded border border-status-error/30 bg-status-error/5 px-3 py-2">
      <p class="text-[11px] text-status-error">{{ error }}</p>
    </div>

    <!-- Import result -->
    <div v-if="importResult" class="rounded border border-status-connected/30 bg-status-connected/5 px-3 py-2">
      <p class="text-[11px] text-status-connected">{{ importResult }}</p>
    </div>

    <!-- Create + Import row -->
    <div class="flex items-center gap-2">
      <input
        v-model="newProfileName"
        type="text"
        placeholder="New profile name..."
        class="flex-1 rounded border border-border bg-surface-0 px-2 py-1 text-xs text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
        @keydown.enter="createProfile"
      />
      <button
        class="rounded bg-accent px-3 py-1 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
        :disabled="!newProfileName.trim() || creating"
        @click="createProfile"
      >
        {{ creating ? 'Creating...' : 'Create' }}
      </button>
      <button
        class="rounded border border-border px-3 py-1 text-xs text-text-secondary transition-colors hover:bg-surface-2"
        @click="handleImport"
      >
        Import
      </button>
    </div>

    <!-- Profile list -->
    <div v-if="profilesStore.profiles.length === 0" class="rounded border border-border px-3 py-4 text-center">
      <p class="text-xs text-text-muted">No profiles yet. Create one to start filtering what AI tools see.</p>
    </div>

    <div
      v-for="profile in profilesStore.profiles"
      :key="profile.id"
      class="rounded border border-border bg-surface-1"
    >
      <!-- Profile header -->
      <button
        class="flex w-full items-center justify-between px-3 py-2 text-left transition-colors hover:bg-surface-2"
        @click="toggleExpanded(profile.id)"
      >
        <div class="flex items-center gap-2">
          <span
            class="inline-block text-[10px] text-text-muted transition-transform"
            :class="expandedId === profile.id ? '' : '-rotate-90'"
          >&#9662;</span>
          <span class="text-xs font-medium text-text-primary">{{ profile.name }}</span>
        </div>
        <span class="text-[10px] text-text-muted">
          {{ profile.serverIds.length }} servers,
          {{ profile.skillIds.length }} skills
        </span>
      </button>

      <!-- Expanded content -->
      <div v-if="expandedId === profile.id" class="border-t border-border">
        <!-- Name -->
        <div class="border-b border-border px-3 py-2">
          <label class="mb-1 block text-[10px] font-medium text-text-muted uppercase">Name</label>
          <input
            :value="profile.name"
            type="text"
            class="w-full rounded border border-border bg-surface-0 px-2 py-1 text-xs text-text-primary focus:border-accent focus:outline-none"
            @change="(e) => updateName(profile, (e.target as HTMLInputElement).value)"
          />
        </div>

        <!-- Directories -->
        <div class="border-b border-border px-3 py-2">
          <div class="mb-1 flex items-center justify-between">
            <label class="text-[10px] font-medium text-text-muted uppercase">Auto-enable in Directories</label>
            <button
              class="text-[10px] text-accent hover:underline"
              @click="addDirectory(profile)"
            >+ Add</button>
          </div>
          <div v-if="profile.directoryPaths.length === 0" class="text-[10px] text-text-muted">
            No directories mapped. AI tools in any directory will use the global profile.
          </div>
          <div v-for="dir in profile.directoryPaths" :key="dir" class="flex items-center justify-between py-0.5">
            <span class="truncate text-[11px] text-text-secondary font-mono">{{ dir }}</span>
            <button
              class="ml-2 shrink-0 text-[10px] text-text-muted hover:text-status-error"
              @click="removeDirectory(profile, dir)"
            >x</button>
          </div>
        </div>

        <!-- AI Tools -->
        <div class="border-b border-border px-3 py-2">
          <label class="mb-1 block text-[10px] font-medium text-text-muted uppercase">AI Tools</label>
          <div v-if="integrations.length === 0" class="text-[10px] text-text-muted">
            No AI tools detected.
          </div>
          <label
            v-for="tool in integrations"
            :key="tool.id"
            class="flex items-center gap-2 py-0.5"
          >
            <input
              type="checkbox"
              :checked="profile.integrationIds.includes(tool.id)"
              class="accent-accent"
              @change="toggleIntegration(profile, tool.id)"
            />
            <span class="text-[11px] text-text-secondary">{{ tool.name }}</span>
          </label>
        </div>

        <!-- Servers -->
        <div class="border-b border-border px-3 py-2">
          <label class="mb-1 block text-[10px] font-medium text-text-muted uppercase">Servers</label>
          <div v-if="serversStore.servers.length === 0" class="text-[10px] text-text-muted">
            No servers configured.
          </div>
          <label
            v-for="server in serversStore.servers.filter(s => !s.managedBy)"
            :key="server.id"
            class="flex items-center gap-2 py-0.5"
          >
            <input
              type="checkbox"
              :checked="profile.serverIds.includes(server.id)"
              class="accent-accent"
              @change="toggleServer(profile, server.id)"
            />
            <span class="text-[11px] text-text-secondary">{{ server.name }}</span>
          </label>
        </div>

        <!-- Skills -->
        <div class="border-b border-border px-3 py-2">
          <label class="mb-1 block text-[10px] font-medium text-text-muted uppercase">Skills</label>
          <div v-if="skillsStore.installedSkills.filter(s => !s.managedBy).length === 0" class="text-[10px] text-text-muted">
            No skills installed.
          </div>
          <label
            v-for="skill in skillsStore.installedSkills.filter(s => !s.managedBy)"
            :key="skill.id"
            class="flex items-center gap-2 py-0.5"
          >
            <input
              type="checkbox"
              :checked="profile.skillIds.includes(skill.id)"
              class="accent-accent"
              @change="toggleSkill(profile, skill.id)"
            />
            <span class="text-[11px] text-text-secondary">{{ skill.name }}</span>
          </label>
        </div>

        <!-- Plugins -->
        <div v-if="pluginsStore.installedPlugins.length > 0" class="border-b border-border px-3 py-2">
          <label class="mb-1 block text-[10px] font-medium text-text-muted uppercase">Plugins</label>
          <label
            v-for="plugin in pluginsStore.installedPlugins"
            :key="plugin.id"
            class="flex items-center gap-2 py-0.5"
          >
            <input
              type="checkbox"
              :checked="profile.pluginIds.includes(plugin.id)"
              class="accent-accent"
              @change="togglePlugin(profile, plugin.id)"
            />
            <span class="text-[11px] text-text-secondary">{{ plugin.id }}</span>
          </label>
        </div>

        <!-- Features -->
        <div class="border-b border-border px-3 py-2">
          <label class="mb-1 block text-[10px] font-medium text-text-muted uppercase">Features</label>
          <label class="flex items-center gap-2 py-0.5">
            <input
              type="checkbox"
              :checked="profile.features.memory"
              class="accent-accent"
              @change="toggleMemory(profile)"
            />
            <span class="text-[11px] text-text-secondary">Memory</span>
          </label>
          <div v-if="profile.features.memory" class="ml-5 mt-1 flex items-center gap-2 py-0.5">
            <label class="text-[10px] text-text-muted">Database</label>
            <input
              type="number"
              :value="profile.features.memoryDb ?? 0"
              min="0"
              max="15"
              class="w-12 rounded border border-border bg-surface-0 px-1.5 py-0.5 text-[11px] text-text-primary focus:border-accent focus:outline-none"
              @change="(e) => updateMemoryDb(profile, (e.target as HTMLInputElement).value)"
            />
            <span class="text-[10px] text-text-muted">
              {{ (profile.features.memoryDb ?? 0) === 0 ? '(shared)' : '(isolated)' }}
            </span>
          </div>
          <label class="flex items-center gap-2 py-0.5">
            <input
              type="checkbox"
              :checked="profile.features.discovery"
              class="accent-accent"
              @change="toggleDiscovery(profile)"
            />
            <span class="text-[11px] text-text-secondary">Discovery Mode</span>
          </label>
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-between px-3 py-2">
          <button
            class="text-[11px] text-accent hover:underline"
            @click="handleExport(profile.id)"
          >
            Export
          </button>
          <div>
            <template v-if="confirmingDeleteId === profile.id">
              <span class="mr-2 text-[10px] text-text-muted">Delete this profile?</span>
              <button
                class="mr-1 text-[11px] text-status-error hover:underline"
                :disabled="deleting === profile.id"
                @click="deleteProfile(profile.id)"
              >
                {{ deleting === profile.id ? 'Deleting...' : 'Yes, delete' }}
              </button>
              <button
                class="text-[11px] text-text-muted hover:underline"
                @click="confirmingDeleteId = null"
              >
                Cancel
              </button>
            </template>
            <button
              v-else
              class="text-[11px] text-status-error/70 hover:text-status-error hover:underline"
              @click="confirmingDeleteId = profile.id"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
