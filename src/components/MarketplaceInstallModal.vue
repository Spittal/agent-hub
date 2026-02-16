<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useRegistryStore } from '@/stores/registry';
import { useServersStore } from '@/stores/servers';
import type { MarketplaceServerDetail } from '@/types/registry';

const props = defineProps<{
  serverId: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const router = useRouter();
const registryStore = useRegistryStore();
const serversStore = useServersStore();

const detail = ref<MarketplaceServerDetail | null>(null);
const loadingDetail = ref(true);
const loadError = ref<string | null>(null);
const installError = ref<string | null>(null);
const installing = ref(false);
const connectingAfterInstall = ref(false);

// Collected user inputs for placeholder env vars
const envVarValues = ref<Record<string, string>>({});

onMounted(async () => {
  try {
    detail.value = await registryStore.fetchServerDetail(props.serverId);
  } catch (e) {
    loadError.value = String(e);
  } finally {
    loadingDetail.value = false;
  }
});

// Missing runtime dependency check
const missingDep = computed(() => {
  if (!detail.value?.runtime || !registryStore.runtimeDeps) return null;
  const rt = detail.value.runtime;
  if (rt === 'npm' && !registryStore.runtimeDeps.npx) return 'node';
  if (rt === 'pypi' && !registryStore.runtimeDeps.uvx) return 'uv';
  if (rt === 'oci' && !registryStore.runtimeDeps.docker) return 'docker';
  return null;
});

const brewHint = computed(() => {
  if (!missingDep.value) return null;
  if (missingDep.value === 'node') return 'brew install node';
  if (missingDep.value === 'uv') return 'brew install uv';
  if (missingDep.value === 'docker') return 'brew install --cask docker';
  return null;
});

// Required env vars (placeholders the user must fill)
const requiredEnvVars = computed(() => {
  if (!detail.value) return [];
  return detail.value.envVars.filter(e => e.isRequired);
});

// All required fields filled?
const allRequiredFilled = computed(() => {
  for (const ev of requiredEnvVars.value) {
    if (!envVarValues.value[ev.name]?.trim()) return false;
  }
  return true;
});

const canInstall = computed(() => allRequiredFilled.value && !missingDep.value && !installing.value);

const statusText = computed(() => {
  if (connectingAfterInstall.value) return 'Connecting...';
  if (installing.value) return 'Installing...';
  return null;
});

async function handleInstall() {
  installError.value = null;
  installing.value = true;

  try {
    const envVars = Object.keys(envVarValues.value).length > 0 ? envVarValues.value : undefined;
    const server = await registryStore.installServer(props.serverId, envVars);

    // Refresh server list and auto-connect
    connectingAfterInstall.value = true;
    await serversStore.loadServers();
    serversStore.connectServer(server.id);
    serversStore.selectServer(server.id);

    emit('close');
    router.push('/');
  } catch (e) {
    installError.value = String(e);
  } finally {
    installing.value = false;
    connectingAfterInstall.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" @click.self="emit('close')">
      <div class="w-full max-w-lg rounded-xl border border-border bg-surface-0 shadow-2xl">
        <!-- Header -->
        <div class="flex items-center justify-between border-b border-border px-5 py-4">
          <h2 class="text-sm font-semibold text-text-primary">Install Server</h2>
          <button
            class="text-text-muted transition-colors hover:text-text-primary"
            @click="emit('close')"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>

        <!-- Body -->
        <div class="max-h-[60vh] overflow-y-auto px-5 py-4">
          <!-- Loading -->
          <div v-if="loadingDetail" class="flex items-center justify-center py-8">
            <span class="text-xs text-text-muted">Loading server details...</span>
          </div>

          <!-- Load error -->
          <div v-else-if="loadError" class="rounded-md bg-status-error/10 p-3 text-xs text-status-error">
            {{ loadError }}
          </div>

          <!-- Content -->
          <template v-else-if="detail">
            <p class="text-xs text-text-secondary">
              {{ detail.description }}
            </p>

            <!-- Missing dependency warning -->
            <div
              v-if="missingDep"
              class="mt-3 rounded-md border border-status-connecting/30 bg-status-connecting/10 p-3 text-xs text-status-connecting"
            >
              <p class="font-medium">Missing runtime dependency</p>
              <p class="mt-1 text-text-secondary">
                This server requires <code class="font-mono">{{ missingDep }}</code>.
                <template v-if="brewHint">
                  Install it with: <code class="rounded bg-surface-2 px-1 py-0.5 font-mono">{{ brewHint }}</code>
                </template>
              </p>
            </div>

            <!-- Required env vars -->
            <div v-if="requiredEnvVars.length" class="mt-4 space-y-3">
              <h3 class="text-xs font-medium text-text-secondary">Environment Variables</h3>
              <div v-for="ev in requiredEnvVars" :key="ev.name" class="space-y-1">
                <label class="block text-xs font-medium text-text-primary">
                  {{ ev.name }}
                  <span class="text-status-error">*</span>
                </label>
                <input
                  v-model="envVarValues[ev.name]"
                  :type="ev.isSecret ? 'password' : 'text'"
                  :placeholder="ev.name"
                  class="w-full rounded-md border border-border bg-surface-1 px-3 py-1.5 font-mono text-xs text-text-primary placeholder-text-muted outline-none transition-colors focus:border-accent"
                />
              </div>
            </div>

            <!-- Install error -->
            <div v-if="installError" class="mt-3 rounded-md bg-status-error/10 p-3 text-xs text-status-error">
              {{ installError }}
            </div>
          </template>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-end gap-2 border-t border-border px-5 py-3">
          <button
            class="rounded-md px-3 py-1.5 text-xs text-text-muted transition-colors hover:text-text-primary"
            @click="emit('close')"
          >
            Cancel
          </button>
          <button
            :disabled="!canInstall || loadingDetail"
            class="rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-40 disabled:hover:bg-accent"
            @click="handleInstall"
          >
            {{ statusText ?? 'Install & Connect' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
