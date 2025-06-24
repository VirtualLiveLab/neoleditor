<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardFooter,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Icon } from '@iconify/vue'
import { useColorMode } from '@vueuse/core'

type MappingValue = [number, number]
type MappingData = Record<string, MappingValue>

const bgColorsDark = ['bg-orange-500', 'bg-yellow-500', 'bg-green-500', 'bg-teal-500', 'bg-sky-500', 'bg-indigo-500', 'bg-purple-500', 'bg-pink-500']
const bgColorsLight = ['bg-orange-400', 'bg-yellow-400', 'bg-green-400', 'bg-teal-400', 'bg-sky-400', 'bg-indigo-400', 'bg-purple-400', 'bg-pink-400']

const colorMode = useColorMode()
const data = defineModel<number[]>({ default: [...Array(2041)] })
const appendStart = ref<number>(1)
const appendEnd = ref<number>(2)
const appendCount = ref<number>(1)
const cursor = ref<number>(0)
const cursorErr = ref<boolean>(false)
const mapping = ref<MappingData>({})
const selection = ref<MappingData>({})
const dragStartCursor = ref<number>(0)
const draging = ref<MappingData>({})
const dragingTpl = computed(() => {
  const tmp: number[] = []
  const offset = cursor.value - Math.min(...Object.keys(draging.value).map(c => Number(c)))
  for (const ci of Object.keys(draging.value).map(c => Number(c))) {
    tmp.push(...[ci + offset, ci + 1 + offset, ci + 2 + offset])
  }
  const unavailable = Object.keys(mapping.value).map(c => [Number(c), Number(c) + 1, Number(c) + 2]).flat()
  const intersection = tmp.filter(x => unavailable.includes(x))
  cursorErr.value = intersection.length > 0
  return tmp
})
const bgColors = computed(() => colorMode.value === 'dark' ? bgColorsDark : bgColorsLight)

let lock = false
watch(data, (newData) => {
  if (lock) return
  lock = true
  mapping.value = {}
  let c = 1
  while (c <= 510) {
    const start = (newData[4 * (c - 1) + 1] << 8) + newData[4 * (c - 1) + 2]
    const end = (newData[4 * (c - 1) + 3] << 8) + newData[4 * (c - 1) + 4]
    if (start * end) {
      mapping.value[c.toString()] = [start, end]
      c += 3
    } else {
      c++
    }
  }
  lock = false
}, { immediate: true })
watch(mapping, (newMapping) => {
  if (lock) return
  lock = true
  for (let c = 1; c <= 510; c++) {
    if (Object.keys(newMapping).includes(c.toString())) {
      data.value[4 * (c - 1) + 1] = (newMapping[c.toString()][0] & 0xff00) >> 8
      data.value[4 * (c - 1) + 2] = newMapping[c.toString()][0] & 0xff
      data.value[4 * (c - 1) + 3] = (newMapping[c.toString()][1] & 0xff00) >> 8
      data.value[4 * (c - 1) + 4] = newMapping[c.toString()][1] & 0xff
    } else {
      data.value[4 * (c - 1) + 1] = 0
      data.value[4 * (c - 1) + 2] = 0
      data.value[4 * (c - 1) + 3] = 0
      data.value[4 * (c - 1) + 4] = 0
    }
  }
  lock = false
}, { deep: true })

onMounted(() => {
  document.addEventListener('click', outClick)
  document.addEventListener('keydown', keydown)
})

function outClick(e: Event) {
  const target = e.target as Element
  if (!target.closest('.selectable')) {
    selection.value = {}
  }
}

function keydown(e: KeyboardEvent) {
  if (e.ctrlKey || e.metaKey) {
    if (e.code === 'KeyA') {
      selection.value = JSON.parse(JSON.stringify(mapping.value))
      e.preventDefault()
    }
  } else {
    if (e.code === 'Delete') {
      for (const c of Object.keys(selection.value)) {
        delete selection.value[c]
        delete mapping.value[c]
      }
    }
    if (e.code === 'Escape') {
      for (const c of Object.keys(selection.value)) {
        delete selection.value[c]
      }
    }
  }
}

function mousemove(e: MouseEvent) {
  const target_rect = (e.currentTarget as Element).getBoundingClientRect()
	const x = e.clientX - target_rect.left
	const y = e.clientY - target_rect.top
  cursor.value = Math.min(Math.floor(x / 35) + 1 + Math.floor(y / 35) * 32, 510)
  if (dragStartCursor.value > 0 && cursor.value !== dragStartCursor.value && Object.keys(selection.value).length) {
    (e.currentTarget as Element).classList.add('cursor-grabbing')
    draging.value = selection.value
    for (const c of Object.keys(draging.value)) {
      delete mapping.value[c]
    }
  }
}

function mouseup(e: MouseEvent) {
  (e.currentTarget as Element).classList.remove('cursor-grabbing')
  if (!cursorErr.value) {
    const offset = cursor.value - Math.min(...Object.keys(draging.value).map(c => Number(c)))
    for (const c of Object.keys(draging.value)) {
      const ci = Number(c)
      mapping.value[(ci + offset).toString()] = draging.value[c]
      selection.value[(ci + offset).toString()] = draging.value[c]
      delete selection.value[c]
    }
  } else {
    for (const c of Object.keys(draging.value)) {
      if (Number(c) > 0) {
        mapping.value[c] = draging.value[c]
      }
    }
  }
  draging.value = {}
  dragStartCursor.value = 0
}

function dragstart(e: MouseEvent, c: string) {
  const isSelected = Object.keys(selection.value).includes(c)
  if (e.ctrlKey || e.metaKey) {
    if (isSelected) {
      delete selection.value[c]
    } else {
      selection.value[c] = mapping.value[c]
    }
  } else {
    if (!isSelected) {
      selection.value = {}
      selection.value[c] = mapping.value[c]
    }
  }
  dragStartCursor.value = cursor.value
}

function append(e: Event) {
  if (
    1 <= appendStart.value && appendStart.value <= 65535 &&
    1 <= appendEnd.value && appendEnd.value <= 65535 &&
    1 <= appendCount.value && appendCount.value <= 170
    ) {
    for (let c = - (appendCount.value * 3); c < 0; c += 3) {
      draging.value[c.toString()] = [appendStart.value, appendEnd.value]
      const step = appendEnd.value - appendStart.value + 1
      appendStart.value += step
      appendEnd.value += step
    }
    appendCount.value = 1
    document.activeElement.blur()
  }
}
</script>

<template>
  <div>
    <div id="appendAction">
      <Button variant="secondary" tabindex="0" class="cursor-pointer">
        <Icon icon="lucide:plus" height="24" />
      </Button>
      <div tabindex="0" class="absolute z-100 mt-2 text-black h-0 overflow-hidden opacity-0 transition">
        <Card class="w-[350px] shadow-md border-2 border-primary/75">
          <CardContent>
            <div class="mb-2">LEDアドレス</div>
            <div class="grid grid-cols-5 w-full mb-4">
              <Input type="number" min="1" max="65535" v-model="appendStart" class="col-span-2 text-right" />
              <div class="text-center">~</div>
              <Input type="number" min="1" max="65535" v-model="appendEnd" class="col-span-2 text-right" />
            </div>
            <div class="flex items-center">
              <div class="flex-none mr-2">複製:</div>
              <Input type="number" min="1" max="170" v-model="appendCount" class="w-16 text-right" />
            </div>
          </CardContent>
          <CardFooter class="flex justify-end px-6">
            <Button @click="append" class="cursor-pointer">追加</Button>
          </CardFooter>
        </Card>
      </div>
    </div>
    <div class="overflow-auto py-2">
      <div @mousemove="mousemove" @mouseup="mouseup" class="relative text-[0.6rem] font-mono" :style="`width: ${35 * 32}px; height: ${35 * 16}px;`">
        <div v-for="i in [...Array(512).keys()]" :class="`absolute p-1 select-none ${dragingTpl.includes(i + 1) && cursorErr ? 'z-100' : ''}`" :style="`width: 35px; height: 35px; top: ${35 * Math.floor(i / 32)}px; left: ${35 * (i % 32)}px;`">
          <div :class="`opacity-75 rounded flex justify-center items-center h-full ${dragingTpl.includes(i + 1) ? 'border' : ''} ${dragingTpl.includes(i + 1) && cursorErr ? 'border-red-500 border-2 bg-red-900' : 'bg-secondary text-secondary-foreground border-primary'}`">
            {{ i + 1 }}
          </div>
        </div>
        <div v-for="c in Object.keys(mapping)" :key="c" @mousedown="(e) => dragstart(e, c)" class="selectable">
          <div v-if="Number(c) % 32 === 0">
            <div class="absolute p-1 select-none" :style="`width: 35px; height: 35px; top: ${35 * Math.floor((Number(c) - 1) / 32)}px; left: ${35 * ((Number(c) - 1) % 32)}px;`">
              <div :class="`${bgColors[(mapping[c][1] - mapping[c][0]) % bgColors.length]} ${Object.keys(selection).includes(c) ? 'border-l border-y border-primary' : ''} w-full h-full rounded-l flex items-center text-black font-bold px-2`">
                {{ c }}
              </div>
            </div>
            <div class="absolute p-1 select-none" :style="`width: ${35 * 2}px; height: 35px; top: ${35 * Math.floor(Number(c) / 32)}px; left: ${35 * (Number(c) % 32)}px;`">
              <div :class="`${bgColors[(mapping[c][1] - mapping[c][0]) % bgColors.length]} ${Object.keys(selection).includes(c) ? 'border-r border-y border-primary' : ''} w-full h-full rounded-r flex justify-end items-center text-black font-bold px-2`">
                <div>
                  <span class="inline-block">{{ mapping[c][0] }}~</span><span class="inline-block">{{ mapping[c][1] }}</span>
                </div>
              </div>
            </div>
          </div>
          <div v-else-if="Number(c) % 32 === 31">
            <div class="absolute p-1 select-none" :style="`width: ${35 * 2}px; height: 35px; top: ${35 * Math.floor((Number(c) - 1) / 32)}px; left: ${35 * ((Number(c) - 1) % 32)}px;`">
              <div :class="`${bgColors[(mapping[c][1] - mapping[c][0]) % bgColors.length]} ${Object.keys(selection).includes(c) ? 'border-l border-y border-primary' : ''} w-full h-full rounded-l flex justify-between items-center text-black font-bold px-2`">
                <div class="flex-none">
                  {{ c }}
                </div>
                <div class="text-center ml-2">
                  <span class="inline-block">{{ mapping[c][0] }}~</span><span class="inline-block">{{ mapping[c][1] }}</span>
                </div>
              </div>
            </div>
            <div class="absolute p-1 select-none" :style="`width: 35px; height: 35px; top: ${35 * Math.floor((Number(c) + 1) / 32)}px; left: ${35 * ((Number(c) + 1) % 32)}px;`">
              <div :class="`${bgColors[(mapping[c][1] - mapping[c][0]) % bgColors.length]} ${Object.keys(selection).includes(c) ? 'border-r border-y border-primary' : ''} w-full h-full rounded-r`"></div>
            </div>
          </div>
          <div v-else class="absolute p-1 select-none" :style="`width: ${35 * 3}px; height: 35px; top: ${35 * Math.floor((Number(c) - 1) / 32)}px; left: ${35 * ((Number(c) - 1) % 32)}px;`">
            <div :class="`${bgColors[(mapping[c][1] - mapping[c][0]) % bgColors.length]} ${Object.keys(selection).includes(c) ? 'border border-primary' : ''} w-full h-full rounded flex justify-between items-center text-black font-bold px-2`">
              <div class="flex-none">
                {{ c }}
              </div>
              <div>
                <span class="inline-block">{{ mapping[c][0] }}~</span><span class="inline-block">{{ mapping[c][1] }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
#appendAction:focus-within div {
  height: auto;
  opacity: 1;
}
input[type=number]::-webkit-inner-spin-button,
input[type=number]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
input[type=number] {
  appearance: textfield;
}
</style>
