<script setup lang="ts">
withDefaults(defineProps<{ title?: string }>(), { title: "" });
const emit = defineEmits<{ (e: "close"): void }>();
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
    @click.self="emit('close')"
  >
    <div class="card w-full max-w-lg p-6 shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold">{{ title }}</h3>
        <button class="text-ink-muted hover:text-ink-primary cursor-pointer" aria-label="關閉" @click="emit('close')">
          ✕
        </button>
      </div>
      <slot />
    </div>
  </div>
</template>
