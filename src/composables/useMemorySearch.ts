import { invoke } from '@tauri-apps/api/core';
import { useMemoriesStore } from '@/stores/memories';
import type { MemorySearchResult } from '@/types/memory';

const PAGE_SIZE = 50;

export function useMemorySearch() {
  const store = useMemoriesStore();

  async function search(append = false) {
    store.loading = true;
    store.error = null;

    try {
      const currentOffset = append ? store.offset : 0;
      const result = await invoke<MemorySearchResult>('search_memories', {
        text: store.query,
        limit: PAGE_SIZE,
        offset: currentOffset,
        memoryType: store.filters.memoryType ?? null,
        topics: store.filters.topics?.length ? store.filters.topics : null,
        entities: store.filters.entities?.length ? store.filters.entities : null,
        namespace: store.filters.namespace ?? null,
        userId: store.filters.userId ?? null,
        sessionId: store.filters.sessionId ?? null,
      });

      if (append) {
        store.items.push(...result.memories);
      } else {
        store.items = result.memories;
      }
      // Sort newest first — the API doesn't guarantee order
      store.items.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
      store.total = result.total;
      store.hasMore = result.memories.length === PAGE_SIZE;
      store.offset = store.items.length;
    } catch (e) {
      store.error = String(e);
    } finally {
      store.loading = false;
    }
  }

  function addTopicFilter(topic: string) {
    const current = store.filters.topics ?? [];
    if (!current.includes(topic)) {
      store.filters = { ...store.filters, topics: [...current, topic] };
      search();
    }
  }

  function addEntityFilter(entity: string) {
    const current = store.filters.entities ?? [];
    if (!current.includes(entity)) {
      store.filters = { ...store.filters, entities: [...current, entity] };
      search();
    }
  }

  function clearFilters() {
    store.filters = {};
    search();
  }

  return { search, addTopicFilter, addEntityFilter, clearFilters };
}
