<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface ProxyStatus {
  running: boolean;
  port: number;
}

const proxyStatus = ref<ProxyStatus | null>(null);
const error = ref<string | null>(null);
const copied = ref(false);

async function fetchProxyStatus() {
  try {
    proxyStatus.value = await invoke<ProxyStatus>('get_proxy_status');
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

function claudeDesktopSnippet(): string {
  const port = proxyStatus.value?.port ?? 0;
  return JSON.stringify(
    {
      mcpServers: {
        'mcp-manager': {
          url: `http://localhost:${port}/mcp`,
        },
      },
    },
    null,
    2,
  );
}

async function copySnippet() {
  try {
    await navigator.clipboard.writeText(claudeDesktopSnippet());
    copied.value = true;
    setTimeout(() => (copied.value = false), 2000);
  } catch {
    // Clipboard may not be available in some webview contexts
  }
}

onMounted(() => {
  fetchProxyStatus();
});
</script>

<template>
  <div class="flex h-full flex-col">
    <header class="border-b border-border px-4 py-3">
      <h1 class="text-sm font-medium">Settings</h1>
    </header>
    <div class="flex-1 overflow-y-auto p-4">
      <div class="mx-auto max-w-md space-y-6">
        <section>
          <h2 class="mb-2 font-mono text-xs font-medium tracking-wide text-text-muted uppercase">
            Proxy
          </h2>
          <p class="mb-3 text-xs text-text-secondary">
            MCP Manager exposes all connected servers as a single MCP proxy server.
            Configure Claude Desktop to connect to this app instead of individual servers.
          </p>

          <div v-if="error" class="rounded bg-status-error/10 px-3 py-2 text-xs text-status-error">
            Failed to get proxy status: {{ error }}
          </div>

          <div v-else-if="proxyStatus" class="space-y-3">
            <div class="flex items-center gap-2">
              <span
                class="inline-block h-2 w-2 rounded-full"
                :class="proxyStatus.running ? 'bg-status-connected' : 'bg-status-error'"
              />
              <span class="text-xs text-text-secondary">
                <template v-if="proxyStatus.running">
                  Running on port <span class="font-mono font-medium text-text-primary">{{ proxyStatus.port }}</span>
                </template>
                <template v-else>
                  Not running
                </template>
              </span>
            </div>

            <div v-if="proxyStatus.running">
              <p class="mb-1.5 text-xs text-text-muted">
                Add this to your Claude Desktop config:
              </p>
              <div class="relative">
                <pre class="overflow-x-auto rounded bg-surface-2 p-3 font-mono text-xs text-text-primary">{{ claudeDesktopSnippet() }}</pre>
                <button
                  class="absolute top-1.5 right-1.5 rounded bg-surface-3 px-2 py-0.5 text-[10px] text-text-muted transition hover:text-text-primary"
                  @click="copySnippet"
                >
                  {{ copied ? 'Copied' : 'Copy' }}
                </button>
              </div>
            </div>
          </div>

          <div v-else class="text-xs text-text-muted">Loading proxy status...</div>
        </section>

        <section>
          <h2 class="mb-2 font-mono text-xs font-medium tracking-wide text-text-muted uppercase">
            Runtimes
          </h2>
          <p class="text-xs text-text-muted">Runtime detection coming soon.</p>
        </section>
      </div>
    </div>
  </div>
</template>
