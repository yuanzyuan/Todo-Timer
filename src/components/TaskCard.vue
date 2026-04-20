<template>
  <div 
    class="task-card" 
    :class="{
      'in-progress': task.status === 'in-progress',
      'blocked': task.isBlocked
    }"
  >
    <input 
      v-model="editableTitle" 
      @blur="saveTitle" 
      @keyup.enter="saveTitle" 
      class="w-full font-medium mb-2 bg-transparent border-b border-white/30 focus:outline-none focus:border-primary"
    />
    
    <!-- 计时信息 -->
    <div class="text-sm mb-3">
      <div v-if="task.status === 'in-progress'">
        <span v-if="task.isBlocked">阻塞中 / </span>
        当前: {{ formatTime(getCurrentActiveTime()) }} / 总: {{ formatTime(task.totalActiveTime + getCurrentActiveTime()) }}
      </div>
      <div v-else>
        总时间: {{ formatTime(task.totalActiveTime) }}
      </div>
    </div>
    
    <!-- 阻塞信息 -->
    <div
      v-if="task.status === 'in-progress' && task.isBlocked"
      class="text-xs text-danger mb-2"
    >
      当前阻塞：{{ task.currentBlockerReason || '未填写原因' }}（{{ formatTime(getCurrentBlockedDuration()) }}）
    </div>

    <div v-if="visibleBlockerLogs.length > 0" class="text-xs text-danger mb-3">
      <div v-for="(log, index) in visibleBlockerLogs" :key="index">
        {{ log.reason }}: {{ formatTime(log.duration) }}
      </div>
    </div>
    
    <!-- 操作按钮区域 -->
    <div class="flex gap-2">
      <!-- 开始按钮 -->
      <button 
        v-if="task.status === 'todo'"
        @click="emit('start-task', task.id)"
        class="flex-1 py-1 rounded-md bg-primary/50 hover:bg-primary/70 transition-colors"
      >
        开始
      </button>
      
      <!-- 阻塞按钮 -->
      <button 
        v-if="task.status === 'in-progress'"
        @click="emit('toggle-block', task.id)"
        class="flex-1 py-1 rounded-md transition-colors"
        :class="task.isBlocked ? 'bg-success/50 hover:bg-success/70' : 'bg-danger/50 hover:bg-danger/70'"
      >
        {{ task.isBlocked ? '恢复' : '阻塞' }}
      </button>
      
      <!-- 完成按钮 -->
      <button 
        v-if="task.status === 'in-progress'"
        @click="emit('complete-task', task.id)"
        class="flex-1 py-1 rounded-md bg-success/50 hover:bg-success/70 transition-colors"
      >
        完成
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

// 类型定义
interface BlockerLog {
  reason: string
  duration: number
}

interface Task {
  id: string
  title: string
  status: 'todo' | 'in-progress' | 'completed'
  totalActiveTime: number
  blockerLogs: BlockerLog[]
  startTime?: number
  blockedStartTime?: number
  isBlocked: boolean
  currentBlockerReason?: string
}

// Props
const props = defineProps<{
  task: Task
}>()

// Emits
const emit = defineEmits<{
  (e: 'toggle-block', taskId: string): void
  (e: 'edit-title', taskId: string, newTitle: string): void
  (e: 'start-task', taskId: string): void
  (e: 'complete-task', taskId: string): void
}>()

// 可编辑标题
const editableTitle = ref(props.task.title)
const visibleBlockerLogs = computed(() => props.task.blockerLogs.filter((log) => log.duration > 0))
const now = ref(Date.now())
let timer: number | null = null

// 监听任务标题变化
watch(() => props.task.title, (newTitle) => {
  editableTitle.value = newTitle
})

onMounted(() => {
  timer = window.setInterval(() => {
    now.value = Date.now()
  }, 1000)
})

onBeforeUnmount(() => {
  if (timer !== null) {
    window.clearInterval(timer)
    timer = null
  }
})

// 保存标题
function saveTitle() {
  if (editableTitle.value !== props.task.title) {
    emit('edit-title', props.task.id, editableTitle.value)
  }
}

// 计算当前活跃时间
function getCurrentActiveTime(): number {
  if (props.task.status !== 'in-progress' || !props.task.startTime) {
    return 0
  }
  
  if (props.task.isBlocked) {
    return 0
  }
  
  return now.value - props.task.startTime
}

function getCurrentBlockedDuration(): number {
  if (props.task.status !== 'in-progress' || !props.task.isBlocked || !props.task.blockedStartTime) {
    return 0
  }

  return now.value - props.task.blockedStartTime
}

// 格式化时间
function formatTime(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  
  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`
  } else {
    return `${seconds}s`
  }
}
</script>

<style scoped>
.task-card {
  cursor: grab;
}

.task-card:active {
  cursor: grabbing;
}
</style>
