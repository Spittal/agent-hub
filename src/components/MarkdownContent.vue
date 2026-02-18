<script setup lang="ts">
import { computed } from 'vue';
import MarkdownIt from 'markdown-it';
import markdownItGitHubAlerts from 'markdown-it-github-alerts';

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
});

md.use(markdownItGitHubAlerts);

// Open links in external browser (target="_blank")
const defaultRender = md.renderer.rules.link_open || function (tokens, idx, options, _env, self) {
  return self.renderToken(tokens, idx, options);
};
md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
  tokens[idx].attrSet('target', '_blank');
  tokens[idx].attrSet('rel', 'noopener noreferrer');
  return defaultRender(tokens, idx, options, env, self);
};

const props = withDefaults(defineProps<{
  content: string;
  inline?: boolean;
}>(), {
  inline: false,
});

const html = computed(() => {
  if (props.inline) {
    return md.renderInline(props.content);
  }
  return md.render(props.content);
});
</script>

<template>
  <span v-if="inline" class="markdown-inline" v-html="html" />
  <div v-else class="markdown-body prose prose-invert prose-sm max-w-none prose-headings:text-text-primary prose-p:text-text-secondary prose-a:text-accent hover:prose-a:text-accent-hover prose-strong:text-text-primary prose-code:text-text-primary prose-code:bg-surface-3 prose-code:rounded prose-code:px-1 prose-code:py-0.5 prose-code:before:content-none prose-code:after:content-none prose-pre:bg-surface-2 prose-pre:border prose-pre:border-border prose-li:text-text-secondary prose-hr:border-border prose-img:rounded prose-img:inline" v-html="html" />
</template>

<style scoped>
.markdown-inline :deep(a) {
  color: var(--color-accent);
}
.markdown-inline :deep(a:hover) {
  color: var(--color-accent-hover);
}
.markdown-inline :deep(code) {
  background: var(--color-surface-3);
  border-radius: 0.25rem;
  padding: 0.125rem 0.25rem;
  font-size: 0.85em;
}

/* GitHub alert boxes */
.markdown-body :deep(.markdown-alert) {
  padding: 0.75rem 1rem;
  margin: 1rem 0;
  border-radius: 0.5rem;
  border-left: 3px solid;
}
.markdown-body :deep(.markdown-alert-title) {
  font-weight: 600;
  font-size: 0.8125rem;
  margin-bottom: 0.25rem;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}
.markdown-body :deep(.markdown-alert-title svg) {
  width: 1rem;
  height: 1rem;
}
.markdown-body :deep(.markdown-alert > p:last-child) {
  margin-bottom: 0;
}
.markdown-body :deep(.markdown-alert-note) {
  border-color: var(--color-accent);
  background: color-mix(in srgb, var(--color-accent) 8%, transparent);
}
.markdown-body :deep(.markdown-alert-note .markdown-alert-title) {
  color: var(--color-accent);
}
.markdown-body :deep(.markdown-alert-tip) {
  border-color: var(--color-status-connected);
  background: color-mix(in srgb, var(--color-status-connected) 8%, transparent);
}
.markdown-body :deep(.markdown-alert-tip .markdown-alert-title) {
  color: var(--color-status-connected);
}
.markdown-body :deep(.markdown-alert-important) {
  border-color: #a78bfa;
  background: color-mix(in srgb, #a78bfa 8%, transparent);
}
.markdown-body :deep(.markdown-alert-important .markdown-alert-title) {
  color: #a78bfa;
}
.markdown-body :deep(.markdown-alert-warning) {
  border-color: var(--color-status-connecting);
  background: color-mix(in srgb, var(--color-status-connecting) 8%, transparent);
}
.markdown-body :deep(.markdown-alert-warning .markdown-alert-title) {
  color: var(--color-status-connecting);
}
.markdown-body :deep(.markdown-alert-caution) {
  border-color: var(--color-status-error);
  background: color-mix(in srgb, var(--color-status-error) 8%, transparent);
}
.markdown-body :deep(.markdown-alert-caution .markdown-alert-title) {
  color: var(--color-status-error);
}
</style>
