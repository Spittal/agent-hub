<script setup lang="ts">
defineProps<{
  label: string;
  enabled: boolean;
  subtitle?: string;
  subtitleClass?: string;
  toggling?: boolean;
  canEnable?: boolean;
  enableLabel?: string;
  disableLabel?: string;
}>();

defineEmits<{
  toggle: [];
}>();
</script>

<template>
  <div class="rounded border border-border bg-surface-1">
    <div class="flex items-center justify-between px-3 py-2.5">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <span
            class="h-1.5 w-1.5 shrink-0 rounded-full"
            :class="enabled ? 'bg-status-connected' : 'bg-surface-3'"
          />
          <span class="text-xs font-medium text-text-primary">{{ label }}</span>
        </div>
        <div v-if="subtitle" class="mt-0.5 pl-3.5">
          <span class="text-[10px]" :class="subtitleClass ?? 'text-text-muted'">{{ subtitle }}</span>
        </div>
        <div v-if="$slots.subtitle" class="mt-0.5 pl-3.5">
          <slot name="subtitle" />
        </div>
      </div>
      <div class="shrink-0 ml-3">
        <button
          v-if="!enabled"
          class="rounded bg-accent px-3 py-1 text-[11px] font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
          :disabled="toggling || canEnable === false"
          @click="$emit('toggle')"
        >
          {{ enableLabel ?? 'Enable' }}
        </button>
        <button
          v-else
          class="rounded bg-surface-3 px-3 py-1 text-[11px] text-text-secondary transition-colors hover:bg-surface-2 disabled:opacity-50"
          :disabled="toggling"
          @click="$emit('toggle')"
        >
          {{ disableLabel ?? 'Disable' }}
        </button>
      </div>
    </div>
    <slot />
  </div>
</template>
