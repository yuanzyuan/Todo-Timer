<template>
  <article
    class="task-card"
    :class="{
      'task-card--todo': task.status === 'todo',
      'task-card--progress': task.status === 'in-progress' && !hero,
      'task-card--hero': task.status === 'in-progress' && hero,
      'task-card--completed': task.status === 'completed',
      'task-card--blocked': task.isBlocked
    }"
  >
    <div class="task-meta-row">
      <template v-if="canEditTags">
        <select
          v-model="editablePriority"
          @change="savePriority"
          class="priority-select"
          :class="priorityClass"
        >
          <option value="高">高</option>
          <option value="中">中</option>
          <option value="低">低</option>
        </select>
        <input
          v-model="editableCategory"
          @blur="saveCategory"
          @keyup.enter="saveCategory"
          type="text"
          placeholder="分类"
          class="category-input"
        />
      </template>
      <template v-else>
        <span class="tag-chip priority-chip" :class="priorityClass">
          {{ task.priority }}
        </span>
        <span v-if="task.category" class="tag-chip category-chip">
          {{ task.category }}
        </span>
      </template>
    </div>

    <input
      v-model="editableTitle"
      @blur="saveTitle"
      @keyup.enter="saveTitle"
      class="title-input"
      :class="{ 'title-input--completed': task.status === 'completed' }"
    />

    <div class="time-panel">
      <template v-if="task.status === 'in-progress' && hero">
        <div class="focus-tag">FOCUS SESSION</div>
        <div class="hero-time">{{ formatHeroTime(getCurrentTotalElapsedTime()) }}</div>
        <div class="hero-session">
          Session {{ formatTime(getCurrentActiveTime()) }}
        </div>
      </template>
      <template v-else-if="task.status === 'in-progress'">
        <div class="time-summary">
          <span>Session {{ formatTime(getCurrentActiveTime()) }}</span>
          <span>Total {{ formatTime(getCurrentTotalElapsedTime()) }}</span>
        </div>
      </template>
      <template v-else-if="task.status === 'completed'">
        总时间: {{ formatTime(task.totalElapsedTime) }}
      </template>
    </div>

    <div
      v-if="task.status === 'in-progress' && task.isBlocked"
      class="blocked-current"
    >
      当前阻塞：{{ task.currentBlockerReason || '未填写原因' }}（{{ formatTime(getCurrentBlockedDuration()) }}）
    </div>

    <div v-if="visibleBlockerLogs.length > 0" class="blocked-log">
      <div v-for="(log, index) in visibleBlockerLogs" :key="index">
        Blocked {{ formatTime(log.duration) }}: {{ log.reason }}
      </div>
    </div>

    <div class="actions">
      <button
        v-if="task.status === 'todo'"
        @click="emit('start-task', task.id)"
        class="btn btn--primary"
      >
        开始
      </button>

      <template v-if="task.status === 'in-progress'">
        <button
          @click="emit('toggle-block', task.id)"
          class="btn btn--soft"
          :class="{ 'btn--warn': !task.isBlocked, 'btn--ok': task.isBlocked }"
        >
          {{ task.isBlocked ? '恢复' : '阻塞' }}
        </button>
        <button
          @click="emit('complete-task', task.id)"
          class="btn btn--success"
        >
          完成
        </button>
      </template>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

interface BlockerLog {
  reason: string
  duration: number
}

interface Task {
  id: string
  title: string
  priority: '高' | '中' | '低'
  category: string
  status: 'todo' | 'in-progress' | 'completed'
  totalActiveTime: number
  totalElapsedTime: number
  blockerLogs: BlockerLog[]
  startTime?: number
  blockedStartTime?: number
  isBlocked: boolean
  currentBlockerReason?: string
}

const props = withDefaults(defineProps<{
  task: Task
  hero?: boolean
  canEditTags?: boolean
}>(), {
  hero: false,
  canEditTags: false
})

const emit = defineEmits<{
  (e: 'toggle-block', taskId: string): void
  (e: 'edit-title', taskId: string, newTitle: string): void
  (e: 'edit-priority', taskId: string, priority: '高' | '中' | '低'): void
  (e: 'edit-category', taskId: string, category: string): void
  (e: 'start-task', taskId: string): void
  (e: 'complete-task', taskId: string): void
}>()

const editableTitle = ref(props.task.title)
const editablePriority = ref(props.task.priority)
const editableCategory = ref(props.task.category)
const visibleBlockerLogs = computed(() => props.task.blockerLogs.filter((log) => log.duration > 0))
const priorityClass = computed(() => {
  if (props.task.priority === '高') return 'priority-high'
  if (props.task.priority === '中') return 'priority-mid'
  return 'priority-low'
})

const now = ref(Date.now())
let timer: number | null = null

watch(() => props.task.title, (newTitle) => {
  editableTitle.value = newTitle
})

watch(() => props.task.priority, (newPriority) => {
  editablePriority.value = newPriority
})

watch(() => props.task.category, (newCategory) => {
  editableCategory.value = newCategory
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

function saveTitle() {
  const value = editableTitle.value.trim()
  if (value && value !== props.task.title) {
    emit('edit-title', props.task.id, value)
  } else {
    editableTitle.value = props.task.title
  }
}

function savePriority() {
  if (!props.canEditTags) return
  if (editablePriority.value !== props.task.priority) {
    emit('edit-priority', props.task.id, editablePriority.value)
  }
}

function saveCategory() {
  if (!props.canEditTags) return
  const normalizedCategory = editableCategory.value.trim()
  if (normalizedCategory !== props.task.category) {
    emit('edit-category', props.task.id, normalizedCategory)
  }
}

function getCurrentActiveTime(): number {
  if (props.task.status !== 'in-progress' || !props.task.startTime || props.task.isBlocked) {
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

function getCurrentTotalElapsedTime(): number {
  const blockedLoggedDuration = props.task.blockerLogs.reduce((acc, log) => acc + log.duration, 0)
  return props.task.totalActiveTime + blockedLoggedDuration + getCurrentActiveTime() + getCurrentBlockedDuration()
}

function formatHeroTime(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000))
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  }

  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
}

function formatTime(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)

  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}
</script>

<style scoped>
.task-card {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1.1rem;
  border-radius: 1.4rem;
  background: var(--surface-container-lowest);
  box-shadow: 0 20px 30px rgba(44, 51, 56, 0.06);
  min-width: 0;
}

.task-card--hero {
  background: rgba(166, 196, 255, 0.35);
  backdrop-filter: blur(18px);
  border-radius: 2rem;
  padding: 1.4rem;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.38), 0 25px 40px rgba(52, 83, 134, 0.2);
}

.task-card--completed {
  opacity: 0.82;
}

.task-card--blocked {
  background: #f6ebe4;
}

.task-meta-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  min-width: 0;
}

.priority-select,
.category-input,
.title-input {
  border: 0;
  outline: 0;
  background: var(--surface-container-highest);
  color: var(--on-surface);
}

.priority-select {
  border-radius: 9999px;
  padding: 0.24rem 0.72rem;
  font-size: 0.68rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.priority-high {
  color: #8f2e35;
}

.priority-mid {
  color: #725420;
}

.priority-low {
  color: #35654d;
}

.category-input {
  flex: 1 1 auto;
  min-width: 0;
  border-radius: 9999px;
  padding: 0.28rem 0.72rem;
  font-size: 0.72rem;
  background: #eef3f8;
}

.title-input {
  width: 100%;
  min-width: 0;
  border-radius: 0.8rem;
  padding: 0.46rem 0.72rem;
  font-size: 1.16rem;
  font-weight: 700;
  font-family: 'Plus Jakarta Sans', sans-serif;
}

.title-input--completed {
  text-decoration: line-through;
  opacity: 0.7;
}

.time-panel {
  color: #596065;
  font-size: 0.82rem;
  min-height: 1.2rem;
}

.time-summary {
  display: flex;
  justify-content: space-between;
  gap: 0.6rem;
  font-weight: 600;
}

.focus-tag {
  width: fit-content;
  margin: 0 auto 0.4rem;
  border-radius: 9999px;
  padding: 0.2rem 0.7rem;
  background: rgba(65, 95, 147, 0.16);
  color: #345386;
  font-size: 0.62rem;
  font-weight: 800;
  letter-spacing: 0.08em;
}

.hero-time {
  text-align: center;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: clamp(2rem, 4vw, 3.2rem);
  line-height: 1.1;
  font-weight: 800;
  color: #345386;
}

.hero-session {
  text-align: center;
  margin-top: 0.25rem;
  font-size: 0.78rem;
  font-weight: 600;
  color: #596065;
}

.tag-chip {
  border-radius: 9999px;
  padding: 0.24rem 0.72rem;
  font-size: 0.68rem;
  font-weight: 700;
  line-height: 1;
  letter-spacing: 0.04em;
}

.priority-chip {
  background: #e3e9ee;
}

.category-chip {
  background: rgba(166, 196, 255, 0.24);
  color: #2f4f83;
}

.blocked-current {
  border-radius: 0.7rem;
  background: rgba(254, 190, 161, 0.28);
  color: #6e422c;
  font-size: 0.75rem;
  padding: 0.5rem 0.6rem;
}

.blocked-log {
  display: grid;
  gap: 0.25rem;
  font-size: 0.72rem;
  color: #7e5038;
}

.actions {
  display: flex;
  gap: 0.55rem;
}

.btn {
  border: 0;
  border-radius: 9999px;
  padding: 0.56rem 0.9rem;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 0.88rem;
  font-weight: 700;
  cursor: pointer;
}

.btn--primary {
  background: linear-gradient(90deg, #415f93, #345386);
  color: #f8f8ff;
}

.btn--success {
  background: linear-gradient(90deg, #396851, #2d5c45);
  color: #e6ffee;
}

.btn--soft {
  background: #dce3e9;
  color: #2c3338;
}

.btn--warn {
  color: #83533c;
}

.btn--ok {
  color: #35654d;
}
</style>
