import { ref, computed, watch, onMounted, onUnmounted, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ServerStats } from '@/types/stats';

const CLIENT_LABELS: Record<string, string> = {
  cursor: 'Cursor',
  'claude-code': 'Claude Code',
  'claude-desktop': 'Claude Desktop',
  windsurf: 'Windsurf',
};

export function formatClientName(id: string): string {
  return CLIENT_LABELS[id] ?? id;
}

export function useServerStats(serverId: Ref<string | null>) {
  const stats = ref<ServerStats | null>(null);

  async function fetchStats() {
    if (!serverId.value) return;
    try {
      stats.value = await invoke<ServerStats>('get_server_stats', { serverId: serverId.value });
    } catch {
      stats.value = null;
    }
  }

  async function resetStats() {
    if (!serverId.value) return;
    await invoke('reset_server_stats', { serverId: serverId.value });
    stats.value = null;
  }

  const successRate = computed(() => {
    if (!stats.value || stats.value.totalCalls === 0) return null;
    return ((stats.value.totalCalls - stats.value.errors) / stats.value.totalCalls) * 100;
  });

  const avgLatency = computed(() => {
    if (!stats.value || stats.value.totalCalls === 0) return null;
    return Math.round(stats.value.totalDurationMs / stats.value.totalCalls);
  });

  const topClient = computed(() => {
    if (!stats.value || Object.keys(stats.value.clients).length === 0) return null;
    const entries = Object.entries(stats.value.clients);
    entries.sort((a, b) => b[1] - a[1]);
    return entries[0][0];
  });

  const sortedTools = computed(() => {
    if (!stats.value) return [];
    return Object.entries(stats.value.tools)
      .map(([name, t]) => ({
        name,
        calls: t.totalCalls,
        errors: t.errors,
        avgLatency: t.totalCalls > 0 ? Math.round(t.totalDurationMs / t.totalCalls) : 0,
      }))
      .sort((a, b) => b.calls - a.calls);
  });

  const recentCalls = computed(() => {
    if (!stats.value) return [];
    return [...stats.value.recentCalls].reverse();
  });

  let unlisten: (() => void) | null = null;

  onMounted(async () => {
    fetchStats();
    unlisten = await listen<{ serverId: string }>('tool-call-recorded', (event) => {
      if (event.payload.serverId === serverId.value) {
        fetchStats();
      }
    });
  });

  onUnmounted(() => {
    unlisten?.();
  });

  watch(serverId, () => fetchStats());

  return {
    stats,
    fetchStats,
    resetStats,
    successRate,
    avgLatency,
    topClient,
    sortedTools,
    recentCalls,
  };
}
