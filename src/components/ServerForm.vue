<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ServerTransport } from '@/types/server';

interface EnvEntry {
  key: string;
  value: string;
}

interface FormValues {
  name: string;
  transport: ServerTransport;
  command: string;
  args: string;
  url: string;
  headers: string;
  env: Record<string, string>;
}

const props = defineProps<{
  initial?: {
    name?: string;
    transport?: ServerTransport;
    command?: string;
    args?: string;
    url?: string;
    headers?: string;
    env?: Record<string, string>;
  };
  submitLabel: string;
}>();

const emit = defineEmits<{
  submit: [values: FormValues];
}>();

const name = ref(props.initial?.name ?? '');
const transport = ref<ServerTransport>(props.initial?.transport ?? 'stdio');
const command = ref(props.initial?.command ?? '');
const args = ref(props.initial?.args ?? '');
const url = ref(props.initial?.url ?? '');
const headers = ref(props.initial?.headers ?? '');

function envToEntries(env?: Record<string, string>): EnvEntry[] {
  if (!env || Object.keys(env).length === 0) return [];
  return Object.entries(env).map(([key, value]) => ({ key, value }));
}

function entriesToEnv(entries: EnvEntry[]): Record<string, string> {
  const env: Record<string, string> = {};
  for (const e of entries) {
    const k = e.key.trim();
    if (k) env[k] = e.value;
  }
  return env;
}

const envEntries = ref<EnvEntry[]>(envToEntries(props.initial?.env));

function addEnvVar() {
  envEntries.value.push({ key: '', value: '' });
}

function removeEnvVar(index: number) {
  envEntries.value.splice(index, 1);
}

// Update refs when initial values change (e.g. after async load)
watch(() => props.initial, (val) => {
  if (!val) return;
  if (val.name !== undefined) name.value = val.name;
  if (val.transport !== undefined) transport.value = val.transport;
  if (val.command !== undefined) command.value = val.command;
  if (val.args !== undefined) args.value = val.args;
  if (val.url !== undefined) url.value = val.url;
  if (val.headers !== undefined) headers.value = val.headers;
  if (val.env !== undefined) envEntries.value = envToEntries(val.env);
});

const urlWarning = computed(() => {
  const u = url.value.trim().toLowerCase();
  if (!u) return null;
  if (u.includes('/docs/') || u.includes('/changelog/') || u.includes('/blog/')) {
    return 'This looks like a documentation page, not an MCP endpoint. The endpoint URL is usually on a different subdomain (e.g. mcp.linear.app) and ends with /sse or /mcp.';
  }
  if (u.endsWith('.html') || u.endsWith('.htm')) {
    return 'This URL points to an HTML page. MCP endpoints typically end with /sse or /mcp.';
  }
  return null;
});

function onSubmit() {
  if (!name.value.trim()) return;
  emit('submit', {
    name: name.value,
    transport: transport.value,
    command: command.value,
    args: args.value,
    url: url.value,
    headers: headers.value,
    env: entriesToEnv(envEntries.value),
  });
}
</script>

<template>
  <form class="flex-1 overflow-y-auto p-4" @submit.prevent="onSubmit">
    <div class="mx-auto max-w-md space-y-4">
      <!-- Name -->
      <div>
        <label class="mb-1 block font-mono text-xs text-text-muted uppercase">Name</label>
        <input
          v-model="name"
          type="text"
          placeholder="My MCP Server"
          class="w-full rounded border border-border bg-surface-1 px-3 py-2 text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
        />
      </div>

      <!-- Transport -->
      <div>
        <label class="mb-1 block font-mono text-xs text-text-muted uppercase">Transport</label>
        <div class="flex gap-2">
          <button
            type="button"
            class="rounded border px-3 py-1.5 text-xs transition-colors"
            :class="transport === 'stdio'
              ? 'border-accent bg-accent/10 text-accent'
              : 'border-border text-text-secondary hover:border-border-active'"
            @click="transport = 'stdio'"
          >
            stdio
          </button>
          <button
            type="button"
            class="rounded border px-3 py-1.5 text-xs transition-colors"
            :class="transport === 'http'
              ? 'border-accent bg-accent/10 text-accent'
              : 'border-border text-text-secondary hover:border-border-active'"
            @click="transport = 'http'"
          >
            HTTP
          </button>
        </div>
      </div>

      <!-- stdio fields -->
      <template v-if="transport === 'stdio'">
        <div>
          <label class="mb-1 block font-mono text-xs text-text-muted uppercase">Command</label>
          <input
            v-model="command"
            type="text"
            placeholder="npx"
            class="w-full rounded border border-border bg-surface-1 px-3 py-2 font-mono text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
          />
        </div>
        <div>
          <label class="mb-1 block font-mono text-xs text-text-muted uppercase">Arguments</label>
          <input
            v-model="args"
            type="text"
            placeholder="-y @modelcontextprotocol/server-filesystem /tmp"
            class="w-full rounded border border-border bg-surface-1 px-3 py-2 font-mono text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
          />
        </div>
        <div>
          <label class="mb-1 block font-mono text-xs text-text-muted uppercase">Environment Variables</label>
          <div v-if="envEntries.length > 0" class="mb-2 space-y-2">
            <div v-for="(entry, i) in envEntries" :key="i" class="flex items-center gap-2">
              <input
                v-model="entry.key"
                type="text"
                placeholder="KEY"
                class="w-2/5 rounded border border-border bg-surface-1 px-2 py-1.5 font-mono text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
              />
              <span class="text-xs text-text-muted">=</span>
              <input
                v-model="entry.value"
                type="password"
                placeholder="value"
                class="min-w-0 flex-1 rounded border border-border bg-surface-1 px-2 py-1.5 font-mono text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
              />
              <button
                type="button"
                class="shrink-0 rounded p-1 text-text-muted transition-colors hover:bg-status-error/10 hover:text-status-error"
                title="Remove variable"
                @click="removeEnvVar(i)"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
          <button
            type="button"
            class="rounded border border-dashed border-border px-3 py-1.5 text-xs text-text-muted transition-colors hover:border-border-active hover:text-text-secondary"
            @click="addEnvVar"
          >
            + Add variable
          </button>
          <p class="mt-1 text-[11px] text-text-muted">API keys and secrets needed by the server process.</p>
        </div>
      </template>

      <!-- HTTP fields -->
      <template v-if="transport === 'http'">
        <div>
          <label class="mb-1 block font-mono text-xs text-text-muted uppercase">URL</label>
          <input
            v-model="url"
            type="text"
            placeholder="https://mcp.linear.app/sse"
            class="w-full rounded border border-border bg-surface-1 px-3 py-2 font-mono text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
          />
          <p v-if="urlWarning" class="mt-1.5 rounded bg-status-error/10 px-2 py-1 text-[11px] text-status-error">{{ urlWarning }}</p>
          <p v-else class="mt-1 text-[11px] text-text-muted">The MCP server endpoint URL, not the docs page. Often ends with /sse or /mcp.</p>
        </div>
        <div>
          <label class="mb-1 block font-mono text-xs text-text-muted uppercase">Headers</label>
          <textarea
            v-model="headers"
            placeholder="Authorization: Bearer your-token-here"
            rows="3"
            class="w-full rounded border border-border bg-surface-1 px-3 py-2 font-mono text-xs text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent"
          />
          <p class="mt-1 text-[11px] text-text-muted">One header per line, format: Key: Value</p>
        </div>
      </template>

      <div class="flex gap-2 pt-2">
        <button
          type="submit"
          class="rounded bg-accent px-4 py-2 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
        >
          {{ submitLabel }}
        </button>
        <slot name="actions" />
      </div>

      <slot name="footer" />
    </div>
  </form>
</template>
