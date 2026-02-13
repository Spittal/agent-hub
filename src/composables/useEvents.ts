import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import { useServersStore } from '@/stores/servers';
import type { OAuthStatus } from '@/types/oauth';
import { useToolsStore } from '@/stores/tools';
import type { ServerStatus } from '@/types/server';
import type { McpTool } from '@/types/mcp';

interface ServerStatusPayload {
  serverId: string;
  status: ServerStatus;
  error?: string;
  lastConnected?: string;
}

interface ServerErrorPayload {
  serverId: string;
  error: string;
}

interface ToolsUpdatedPayload {
  serverId: string;
  serverName: string;
  tools: McpTool[];
}

interface OAuthRequiredPayload {
  serverId: string;
}

interface OAuthStatusPayload {
  serverId: string;
  status: OAuthStatus;
}

interface NavigateToServerPayload {
  serverId: string;
}

export function useEvents() {
  const unlisteners: UnlistenFn[] = [];
  const router = useRouter();

  onMounted(async () => {
    const serversStore = useServersStore();
    const toolsStore = useToolsStore();

    unlisteners.push(
      await listen<ServerStatusPayload>('server-status-changed', (event) => {
        serversStore.updateServerStatus(event.payload.serverId, event.payload.status);
        if (event.payload.status === 'error' && event.payload.error) {
          serversStore.setError(event.payload.serverId, event.payload.error);
        }
      })
    );

    unlisteners.push(
      await listen<ServerErrorPayload>('server-error', (event) => {
        serversStore.setError(event.payload.serverId, event.payload.error);
      })
    );

    unlisteners.push(
      await listen<ToolsUpdatedPayload>('tools-updated', (event) => {
        toolsStore.setTools(
          event.payload.serverId,
          event.payload.serverName,
          event.payload.tools
        );
      })
    );

    unlisteners.push(
      await listen<OAuthRequiredPayload>('oauth-required', (event) => {
        serversStore.setOAuthStatus(event.payload.serverId, 'idle');
      })
    );

    unlisteners.push(
      await listen<OAuthStatusPayload>('oauth-status-changed', (event) => {
        serversStore.setOAuthStatus(event.payload.serverId, event.payload.status);
        // Clear OAuth status when connected successfully
        if (event.payload.status === 'authorized') {
          // Will be cleared once server-status-changed fires with 'connected'
        }
      })
    );

    unlisteners.push(
      await listen<NavigateToServerPayload>('navigate-to-server', (event) => {
        serversStore.selectServer(event.payload.serverId);
        router.push('/');
      })
    );
  });

  onUnmounted(() => {
    unlisteners.forEach((unlisten) => unlisten());
  });
}
