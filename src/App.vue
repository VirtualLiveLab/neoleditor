<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Icon } from '@iconify/vue'
import { useColorMode } from '@vueuse/core'
import { SerialPort } from 'tauri-plugin-serialplugin'
import { invoke } from '@tauri-apps/api/core'

const availabeVersions = [0x20]

useColorMode()
const availablePorts = ref<string[]>([])
const port = ref('')
const loading = ref(0)
const version = ref<number | null>(null)
const versionStr = computed(() => (
  version.value === null ? '' : `v${Math.floor(version.value / 16)}.${version.value % 16}`
))

let intervalId
onMounted(() => {
  intervalId = setInterval(async () => {
    availablePorts.value = Object.entries(await SerialPort.available_ports()).filter(([k, v]) => v.vid === '12346' && v.pid == '4097').map(kv => kv[0]).sort()
    if (availablePorts.value.length) {
      if (!availablePorts.value.includes(port.value)) {
        port.value = availablePorts.value[0]
      }
    } else {
      port.value = ''
    }
  }, 1000)
})

onUnmounted(() => clearInterval(intervalId))

watch(port, async (newPort) => {
  version.value = null
  if (newPort) {
    loading.value = 1
    const serial = new SerialPort({
      path: newPort,
      baudRate: 115200
    })
    try {
      await serial.open()
      const reqPacket = await invoke('encode_packet', { input: [0x00, 0x00] })
      await serial.writeBinary(reqPacket)
      const resData = await serial.readBinary({ timeout: 1000, size: 2100 })
      const resPacket = await invoke('decode_packet', { input: resData })
      if (resPacket[1] === 0x00 && availabeVersions.includes(resPacket[0])) {
        version.value = resPacket[0]
      }
    } catch (error) {
      console.error(error)
    } finally {
      loading.value = 0
      await serial.close()
    }
  }
})
</script>

<template>
  <main class="container mx-auto p-6">
    <h1 class="font-bold text-4xl mb-4">NeoLEDitor</h1>
    <div class="flex items-center">
      ポートを選択:
      <div class="px-4">
        <Select v-model="port">
          <SelectTrigger>
            <SelectValue placeholder="ポートを選択" />
          </SelectTrigger>
          <SelectContent>
            <SelectLabel>ポートを選択</SelectLabel>
            <SelectGroup>
              <SelectItem v-for="p in availablePorts" :key="p" :value="p">
                {{ p }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
      <div v-if="port && !loading && version !== null" class="flex">
        <Icon icon="lucide:circle-check" height="24" class="text-green-500 mr-1" />
        {{ versionStr }}
      </div>
      <div v-if="port && !loading && version === null" class="flex text-red-500 font-bold">
        <Icon icon="lucide:circle-x" height="24" class="mr-1" />
        非対応デバイス
      </div>
    </div>
  </main>
</template>
