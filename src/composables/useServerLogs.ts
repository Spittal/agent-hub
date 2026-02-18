import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useLogsStore, type LogEntry } from '@/stores/logs';

interface ServerLogPayload {
  serverId: string;
  level: LogEntry['level'];
  message: string;
}

export function useServerLogs() {
  let unlisten: UnlistenFn | null = null;

  onMounted(async () => {
    const logsStore = useLogsStore();

    unlisten = await listen<ServerLogPayload>('server-log', (event) => {
      logsStore.addLog({
        timestamp: new Date().toISOString(),
        serverId: event.payload.serverId,
        level: event.payload.level,
        message: event.payload.message,
      });
    });

    // Drain logs that were buffered before we started listening
    // (e.g. HTTP server connections during reconnect_on_startup)
    const buffered = await invoke<ServerLogPayload[]>('drain_log_buffer');
    for (const entry of buffered) {
      logsStore.addLog({
        timestamp: new Date().toISOString(),
        serverId: entry.serverId,
        level: entry.level,
        message: entry.message,
      });
    }
  });

  onUnmounted(() => {
    unlisten?.();
  });
}
