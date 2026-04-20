<template>
  <article
    class="task-card"
    :class="{
      'task-card--todo': task.status === 'todo',
      'task-card--progress': task.status === 'in-progress' && !hero,
      'task-card--hero': task.status === 'in-progress' && hero,
      'task-card--completed': task.status === 'completed',
      'task-card--blocked': task.isBlocked,
      'task-card--paused': task.status === 'in-progress' && task.isPaused
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
      :class="{
        'title-input--completed': task.status === 'completed',
        'title-input--center': task.status === 'in-progress'
      }"
    />

    <div class="time-panel">
      <template v-if="task.status === 'in-progress' && hero">
        <div class="time-points">
          <span>开始 {{ formatTimePoint(task.startedAt) }}</span>
        </div>
        <div class="focus-tag">{{ task.isBlocked ? 'BLOCK SESSION' : 'FOCUS SESSION' }}</div>
        <div class="hero-time">
          {{ formatHeroTime(task.isBlocked ? getCurrentBlockedDuration() : getCurrentTotalElapsedTime()) }}
        </div>
        <div class="hero-session">
          {{ task.isBlocked ? 'Block' : 'Session' }} {{ formatTime(task.isBlocked ? getCurrentBlockedDuration() : getCurrentActiveTime()) }}
        </div>
        <div v-if="task.isPaused" class="paused-now">已暂停（{{ formatTimePoint(task.pauseLogs[task.pauseLogs.length - 1]?.pausedAt) }}）</div>
      </template>
      <template v-else-if="task.status === 'in-progress'">
        <div class="time-points">
          <span>开始 {{ formatTimePoint(task.startedAt) }}</span>
        </div>
        <div class="time-summary">
          <span>{{ task.isBlocked ? 'Block' : 'Session' }} {{ formatTime(task.isBlocked ? getCurrentBlockedDuration() : getCurrentActiveTime()) }}</span>
          <span>Total {{ formatTime(getCurrentTotalElapsedTime()) }}</span>
        </div>
        <div v-if="task.isPaused" class="paused-now">已暂停（{{ formatTimePoint(task.pauseLogs[task.pauseLogs.length - 1]?.pausedAt) }}）</div>
      </template>
      <template v-else-if="task.status === 'completed'">
        <div class="time-points">
          <span>开始 {{ formatTimePoint(task.startedAt) }}</span>
          <span>结束 {{ formatTimePoint(task.endedAt) }}</span>
        </div>
        <div class="time-summary time-summary--completed">
          <span>总时间 {{ formatTime(task.totalElapsedTime) }}</span>
          <span>阻塞 {{ formatTime(getBlockedTotalTime()) }}</span>
          <span>Focus {{ formatTime(getFocusTotalTime()) }}</span>
        </div>
        <div class="completed-date">{{ formatCompletedDate(task.completedAt) }}</div>
      </template>
    </div>

    <div
      v-if="task.status === 'in-progress' && task.isBlocked"
      class="blocked-current"
    >
      当前阻塞：{{ task.currentBlockerReason || '未填写原因' }}（{{ formatTime(getCurrentBlockedDuration()) }}）
    </div>

    <div
      v-if="visibleBlockerLogs.length > 0"
      class="blocked-log"
      :class="{ 'blocked-log--highlight': task.status === 'in-progress' }"
    >
      <div
        v-for="(log, index) in visibleBlockerLogs"
        :key="index"
        class="blocked-log-item"
      >
        阻塞点 {{ index + 1 }} · {{ formatTime(log.duration) }}：{{ log.reason }}
      </div>
    </div>

    <div v-if="visiblePauseLogs.length > 0" class="pause-log">
      <div v-for="(log, index) in visiblePauseLogs" :key="`pause-${index}`" class="pause-log-item">
        暂停点 {{ index + 1 }} · {{ formatTimePoint(log.pausedAt) }} → {{ formatTimePoint(log.resumedAt) }}
      </div>
    </div>

    <div class="actions" :class="{ 'actions--center': task.status === 'in-progress' }">
      <button
        v-if="task.status === 'todo'"
        @click="emit('start-task', task.id)"
        class="btn btn--primary"
      >
        开始
      </button>
      <button
        v-if="task.status === 'todo'"
        @click="emit('delete-task', task.id)"
        class="btn btn--danger"
      >
        删除
      </button>

      <template v-if="task.status === 'in-progress'">
        <div class="progress-top-actions">
          <button
            @click="emit('toggle-pause', task.id)"
            class="btn btn--soft btn--mini"
            :class="{ 'btn--pause': !task.isPaused, 'btn--resume': task.isPaused }"
            :disabled="task.isBlocked"
          >
            {{ task.isPaused ? '继续' : '暂停' }}
          </button>
          <button
            @click="emit('complete-task', task.id)"
            class="btn btn--success btn--mini"
          >
            完成
          </button>
        </div>
        <button
          @click="emit('toggle-block', task.id)"
          class="btn btn--soft btn--block-main"
          :class="{ 'btn--warn': !task.isBlocked, 'btn--ok': task.isBlocked }"
          :disabled="task.isPaused"
        >
          {{ task.isBlocked ? '恢复' : '阻塞' }}
        </button>
      </template>

      <button
        v-if="task.status === 'completed'"
        @click="emit('delete-task', task.id)"
        class="btn btn--danger"
      >
        删除
      </button>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

interface BlockerLog {
  reason: string
  duration: number
}

interface PauseLog {
  pausedAt: number
  resumedAt?: number
}

interface Task {
  id: string
  title: string
  priority: '高' | '中' | '低'
  category: string
  status: 'todo' | 'in-progress' | 'completed'
  totalActiveTime: number
  totalElapsedTime: number
  completedAt?: number
  startedAt?: number
  endedAt?: number
  isPaused: boolean
  pauseLogs: PauseLog[]
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
  (e: 'delete-task', taskId: string): void
  (e: 'complete-task', taskId: string): void
  (e: 'toggle-pause', taskId: string): void
}>()

const editableTitle = ref(props.task.title)
const editablePriority = ref(props.task.priority)
const editableCategory = ref(props.task.category)
const visibleBlockerLogs = computed(() => props.task.blockerLogs.filter((log) => log.duration > 0))
const visiblePauseLogs = computed(() => props.task.pauseLogs.filter((log) => !!log.pausedAt))
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
  if (props.task.status !== 'in-progress' || !props.task.startTime || props.task.isBlocked || props.task.isPaused) {
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

function getBlockedTotalTime(): number {
  return props.task.blockerLogs.reduce((acc, log) => acc + log.duration, 0)
}

function getFocusTotalTime(): number {
  return props.task.totalActiveTime
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

function formatCompletedDate(timestamp?: number): string {
  if (!timestamp) return '—'

  const date = new Date(timestamp)
  const now = new Date()

  const isSameDay = date.toDateString() === now.toDateString()
  const yesterday = new Date(now)
  yesterday.setDate(now.getDate() - 1)
  const isYesterday = date.toDateString() === yesterday.toDateString()

  const timeText = date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })
  if (isSameDay) return `Today, ${timeText}`
  if (isYesterday) return `Yesterday, ${timeText}`

  const dateText = date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
  return `${dateText}, ${timeText}`
}

function formatTimePoint(timestamp?: number): string {
  if (!timestamp) return '--:--'
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', hour12: false })
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
  position: relative;
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

.task-card--todo,
.task-card--completed {
  gap: 0.5rem;
  padding: 0.8rem;
  border-radius: 1.1rem;
}

.task-card--blocked {
  background: #f6ebe4;
}

.task-card--paused {
  background: rgba(96, 190, 130, 0.2);
}

.task-card--hero.task-card--paused {
  background: linear-gradient(180deg, rgba(96, 190, 130, 0.28), rgba(166, 196, 255, 0.25));
}

.task-meta-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  min-width: 0;
}

.task-card--progress .task-meta-row,
.task-card--hero .task-meta-row {
  padding-right: 5.8rem;
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
  color: #a33b46;
}

.priority-mid {
  color: #7c5c24;
}

.priority-low {
  color: #376c52;
}

.priority-select.priority-high {
  background: rgba(250, 116, 111, 0.22);
}

.priority-select.priority-mid {
  background: rgba(247, 199, 99, 0.26);
}

.priority-select.priority-low {
  background: rgba(96, 190, 130, 0.24);
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

.title-input--center {
  text-align: center;
}

.task-card--todo .title-input,
.task-card--completed .title-input {
  font-size: 0.95rem;
  padding: 0.36rem 0.58rem;
}

.time-panel {
  color: #596065;
  font-size: 0.82rem;
  min-height: 1.2rem;
}

.time-points {
  display: flex;
  justify-content: space-between;
  gap: 0.45rem;
  font-size: 0.68rem;
  color: #6a7176;
  margin-bottom: 0.2rem;
}

.time-summary {
  display: flex;
  justify-content: space-between;
  gap: 0.6rem;
  font-weight: 600;
}

.time-summary--completed {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.35rem;
  font-size: 0.68rem;
}

.completed-date {
  margin-top: 0.15rem;
  text-align: right;
  font-size: 0.68rem;
  color: #7a8186;
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

.paused-now {
  margin-top: 0.25rem;
  text-align: center;
  font-size: 0.72rem;
  color: #83533c;
  font-weight: 700;
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

.priority-chip.priority-high {
  background: rgba(250, 116, 111, 0.2);
  border: 1px solid rgba(163, 59, 70, 0.28);
}

.priority-chip.priority-mid {
  background: rgba(247, 199, 99, 0.24);
  border: 1px solid rgba(124, 92, 36, 0.24);
}

.priority-chip.priority-low {
  background: rgba(96, 190, 130, 0.22);
  border: 1px solid rgba(55, 108, 82, 0.24);
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

.blocked-log-item {
  border-radius: 0.7rem;
  padding: 0.38rem 0.52rem;
  background: rgba(254, 190, 161, 0.14);
}

.blocked-log--highlight .blocked-log-item {
  background: rgba(254, 190, 161, 0.24);
  color: #6d3f2b;
  border-left: 3px solid rgba(163, 59, 70, 0.42);
  font-weight: 600;
}

.pause-log {
  display: grid;
  gap: 0.25rem;
  font-size: 0.7rem;
  color: #4b5a70;
}

.pause-log-item {
  border-radius: 0.7rem;
  padding: 0.35rem 0.5rem;
  background: rgba(166, 196, 255, 0.2);
}

.actions {
  display: flex;
  gap: 0.55rem;
}

.actions--center {
  justify-content: center;
}

.progress-top-actions {
  position: absolute;
  top: 0.85rem;
  right: 0.85rem;
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
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

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn--mini {
  padding: 0.38rem 0.66rem;
  font-size: 0.72rem;
}

.btn--block-main {
  padding: 0.6rem 1.08rem;
  font-size: 0.9rem;
}

.task-card--todo .btn,
.task-card--completed .btn {
  padding: 0.42rem 0.72rem;
  font-size: 0.78rem;
}

.btn--primary {
  background: linear-gradient(90deg, #415f93, #345386);
  color: #f8f8ff;
}

.btn--success {
  background: linear-gradient(90deg, #396851, #2d5c45);
  color: #e6ffee;
}

.btn--danger {
  background: rgba(131, 83, 60, 0.14);
  color: #7e5038;
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

.btn--pause {
  color: #345386;
}

.btn--resume {
  color: #35654d;
}
</style>
