<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useColorMode } from '@vueuse/core'

const greetMsg = ref('')
const name = ref('')
const mode = useColorMode()

async function greet() {
  greetMsg.value = await invoke('greet', { name: name.value })
}
</script>

<template>
  <main class="container mx-auto p-6">
    <h1>Welcome to Tauri + Vue</h1>

    <div class="flex">
      <a href="https://vitejs.dev" target="_blank">
        <img src="/vite.svg" class="h-8 w-8" alt="Vite logo" />
      </a>
      <a href="https://tauri.app" target="_blank">
        <img src="/tauri.svg" class="h-8 w-8" alt="Tauri logo" />
      </a>
      <a href="https://vuejs.org/" target="_blank">
        <img src="./assets/vue.svg" class="h-8 w-8" alt="Vue logo" />
      </a>
    </div>
    <p>Click on the Tauri, Vite, and Vue logos to learn more.</p>

    <form class="row" @submit.prevent="greet">
      <Input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <Button type="submit" class="cursor-pointer">Greet</Button>
    </form>
    <p>{{ greetMsg }}</p>
  </main>
</template>
