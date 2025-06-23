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
import { Button } from '@/components/ui/button'
import { Slider } from '@/components/ui/slider'
import { Icon } from '@iconify/vue'
import Mapping from '@/components/Mapping.vue'
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
const cache = ref<number[]>([])
const data = ref<number[]>([])
const brightness = computed({
  get() {
    return [data.value[0]]
  },
  set(newBrightness) {
    data.value[0] = newBrightness[0]
  },
})

let intervalId: NodeJS.Timeout | null = null
onMounted(() => {
  intervalId = setInterval(async () => {
    const ports = await SerialPort.available_ports() as Record<string, { vid: string; pid: string }>
    availablePorts.value = Object.entries(ports).filter(([_, v]) => v.vid === '12346' && v.pid == '4097').map(kv => kv[0]).sort()
    if (availablePorts.value.length) {
      if (!availablePorts.value.includes(port.value)) {
        port.value = availablePorts.value[0]
      }
    } else {
      port.value = ''
    }
  }, 1000)
})

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId)
  }
})

watch(port, async (newPort) => {
  version.value = null
  cache.value = []
  data.value = []
  if (newPort) {
    loading.value = 1
    const serial = new SerialPort({
      path: newPort,
      baudRate: 115200
    })
    try {
      await serial.open()
      const reqPacket = await invoke('encode_packet', { input: [0x00, 0x00] }) as number[]
      await serial.writeBinary(reqPacket)
      const resData = await serial.readBinary({ timeout: 1000, size: 2100 })
      const resPacket = await invoke('decode_packet', { input: resData }) as number[]
      if (resPacket[1] === 0x00 && availabeVersions.includes(resPacket[0])) {
        version.value = resPacket[0]
        cache.value = resPacket.slice(2)
        data.value = resPacket.slice(2)
      }
    } catch (err) {
      console.error(err)
    } finally {
      loading.value = 0
      await serial.close()
    }
  }
})

async function save() {
  const serial = new SerialPort({
    path: port.value,
    baudRate: 115200
  })
  try {
    await serial.open()
    const reqPacket = await invoke('encode_packet', { input: [version.value, 0x01, ...data.value] }) as number[]
    await serial.writeBinary(reqPacket)
    while (await serial.bytesToWrite() > 0) {
      await new Promise(resolve => setTimeout(resolve, 1));
    }
    const resData = await serial.readBinary({ timeout: 1000, size: 2100 })
    const resPacket = await invoke('decode_packet', { input: resData }) as number[]
    if (resPacket[1] === 0x00 && availabeVersions.includes(resPacket[0])) {
      cache.value = Array.from(data.value)
    }
  } catch (err) {
    console.error(err)
  } finally {
    await serial.close()
  }
}

function reset() {
  data.value = Array.from(cache.value)
}
</script>

<template>
  <main class="max-w-[1328px] mx-auto p-6 select-none">
    <h1 class="font-bold text-4xl mb-4">NeoLEDitor</h1>
    <div class="flex items-center mb-4">
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
    <template v-if="data.length">
      <div class="mb-4 flex">
        <Button @click="save" :disabled="JSON.stringify(cache) === JSON.stringify(data)" class="cursor-pointer mr-2">保存</Button>
        <Button variant="secondary" @click="reset" :disabled="JSON.stringify(cache) === JSON.stringify(data)" class="cursor-pointer">リセット</Button>
      </div>
      <div class="mb-4 flex">
        明るさ:
        <Slider :min="0" :max="255" :step="1" v-model="brightness" class="w-60 mx-4" />
        {{ data[0] }}
      </div>
      <div v-if="false" v-for="(_, i) in data.slice(1)" :key="i">
        <input type="number" v-model="data[i + 1]" />
      </div>
      <Mapping v-model="data" />
    </template>
  </main>
</template>
