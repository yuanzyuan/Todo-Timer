<template>
  <div class="app-shell">
    <header class="topbar">
      <div class="brand">
        <span class="brand-dot"></span>
        <h1>To do Timer</h1>
      </div>
      <nav class="tabs">
        <a class="tab tab--active" href="#">Board</a>
        <a class="tab" href="#">Timeline</a>
        <a class="tab" href="#">History</a>
      </nav>
      <div class="top-actions">
        <input class="search-input" placeholder="Search tasks..." type="text" />
      </div>
    </header>

    <main class="board">
      <section class="lane lane--compact">
        <div class="lane-head">
          <h2>Todo <span>{{ todoTasks.length }}</span></h2>
        </div>
        <button @click="addTask('todo')" class="add-btn">+ Add Task</button>
        <div class="lane-list">
          <TaskCard
            v-for="task in todoTasks"
            :key="task.id"
            :task="task"
            :can-edit-tags="true"
            @start-task="startTask"
            @delete-task="deleteTask"
            @edit-title="editTaskTitle"
            @edit-priority="editTaskPriority"
            @edit-category="editTaskCategory"
          />
          <div v-if="todoTasks.length === 0" class="lane-empty">No todo task yet</div>
        </div>
      </section>

      <section class="lane lane--progress">
        <div class="lane-head">
          <h2>
            In Progress
            <span class="live-dot-group" v-if="inProgressTasks.length > 0">
              <span
                v-for="(_, index) in inProgressTasks"
                :key="`progress-dot-${index}`"
                class="live-dot live-dot--active"
                :style="{ animationDelay: `${index * 0.18}s` }"
              ></span>
            </span>
            <span class="live-dot" v-else></span>
          </h2>
        </div>
        <div class="lane-list">
          <TaskCard
            v-for="(task, index) in inProgressTasks"
            :key="task.id"
            :task="task"
            :hero="index === 0"
            :can-edit-tags="false"
            @toggle-block="toggleBlock"
            @toggle-pause="togglePause"
            @complete-task="requestCompleteTask"
            @edit-title="editTaskTitle"
            @edit-priority="editTaskPriority"
            @edit-category="editTaskCategory"
          />
          <div v-if="inProgressTasks.length === 0" class="lane-empty">Move a task here to start focus</div>
        </div>
      </section>

      <section class="lane lane--compact">
        <div class="lane-head">
          <h2>Completed <span>✓</span></h2>
        </div>
        <div class="completed-tools">
          <label class="auto-export-toggle">
            <input
              type="checkbox"
              v-model="autoExportEnabled"
              @change="applyAutoExportSettings"
            />
            <span>自动导出</span>
          </label>
          <label class="auto-export-time">
            <span>时间</span>
            <input
              type="time"
              v-model="autoExportTime"
              step="60"
              :disabled="!autoExportEnabled"
              @change="applyAutoExportSettings"
            />
          </label>
          <button
            class="export-btn"
            @click="exportCompletedMarkdownFiles"
            :disabled="isExportingMd"
          >
            {{ isExportingMd ? '导出中...' : '导出MD' }}
          </button>
        </div>
        <div class="lane-list">
          <TaskCard
            v-for="task in completedTasks"
            :key="task.id"
            :task="task"
            :can-edit-tags="false"
            @delete-task="requestDeleteCompletedTask"
            @edit-title="editTaskTitle"
            @edit-priority="editTaskPriority"
            @edit-category="editTaskCategory"
          />
          <div v-if="completedTasks.length === 0" class="lane-empty">No completed task yet</div>
        </div>
      </section>
    </main>

    <Teleport to="body">
      <div v-if="showBlockerDialog" class="modal-wrap">
        <button class="modal-backdrop" @click="closeBlockerDialog" aria-label="关闭阻塞弹窗"></button>
        <div class="modal-panel">
          <h3>输入阻塞原因</h3>
          <input v-model="blockerReason" type="text" placeholder="请输入阻塞原因" />
          <div class="modal-actions">
            <button @click="closeBlockerDialog" class="modal-btn modal-btn--muted">取消</button>
            <button @click="confirmBlockerReason" class="modal-btn modal-btn--primary">确认</button>
          </div>
        </div>
      </div>
    </Teleport>

    <Teleport to="body">
      <div v-if="showConfirmDialog" class="modal-wrap">
        <button class="modal-backdrop" @click="closeConfirmDialog" aria-label="关闭确认弹窗"></button>
        <div class="modal-panel">
          <h3>请确认</h3>
          <p class="confirm-text">{{ confirmMessage }}</p>
          <div class="modal-actions">
            <button @click="closeConfirmDialog" class="modal-btn modal-btn--muted">取消</button>
            <button @click="executeConfirmedAction" class="modal-btn modal-btn--primary">确认</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import TaskCard from './components/TaskCard.vue'

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
  completedMarkdown?: string
  completedAt?: number
  startedAt?: number
  endedAt?: number
  isPaused: boolean
  pausedAt?: number
  pauseLogs: PauseLog[]
  blockerLogs: BlockerLog[]
  startTime?: number
  blockedStartTime?: number
  isBlocked: boolean
  currentBlockerReason?: string
}

const todoTasks = ref<Task[]>([])
const inProgressTasks = ref<Task[]>([])
const completedTasks = ref<Task[]>([])

const showBlockerDialog = ref(false)
const blockerReason = ref('')
const currentBlockingTaskId = ref<string | null>(null)
const showConfirmDialog = ref(false)
const confirmMessage = ref('')
const confirmTaskId = ref<string | null>(null)
const confirmAction = ref<'complete-task' | 'delete-completed-task' | null>(null)
const isExportingMd = ref(false)
let autoExportTimerId: number | null = null

const AUTO_EXPORT_RUN_KEY = 'autoExportLastRunDate'
const AUTO_EXPORT_ENABLED_KEY = 'autoExportEnabled'
const AUTO_EXPORT_TIME_KEY = 'autoExportTime'

const autoExportEnabled = ref(localStorage.getItem(AUTO_EXPORT_ENABLED_KEY) === 'true')
const autoExportTime = ref(localStorage.getItem(AUTO_EXPORT_TIME_KEY) || '00:00')

function closeBlockerDialog() {
  showBlockerDialog.value = false
  currentBlockingTaskId.value = null
  blockerReason.value = ''
}

function closeConfirmDialog() {
  showConfirmDialog.value = false
  confirmMessage.value = ''
  confirmTaskId.value = null
  confirmAction.value = null
}

async function exportCompletedMarkdownFiles() {
  if (isExportingMd.value) return

  const tasksToExport = completedTasks.value
    .filter((task) => typeof task.completedMarkdown === 'string' && task.completedMarkdown.trim().length > 0)
    .map((task) => ({
      title: task.title,
      completedMarkdown: task.completedMarkdown as string,
      completedAt: task.completedAt
    }))

  if (tasksToExport.length === 0) {
    window.alert('没有可导出的已完成任务 Markdown')
    return
  }

  try {
    isExportingMd.value = true
    const selectedFolder = await invoke<string | null>('choose_export_folder')
    if (!selectedFolder) {
      window.alert('已取消导出：未选择目录')
      return
    }

    const savedExportPath = await invoke<string>('export_completed_markdowns', {
      tasks: tasksToExport,
      exportPath: selectedFolder
    })
    window.alert(`已成功导出：${savedExportPath}`)
  } catch (error) {
    console.error(error)
    const message = typeof error === 'string'
      ? error
      : (error && typeof error === 'object' && 'message' in error ? String((error as { message?: unknown }).message) : '导出失败，请稍后重试')
    window.alert(`导出失败：${message}`)
  } finally {
    isExportingMd.value = false
  }
}

function getDateKey(date: Date): string {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

function parseAutoExportTime() {
  const matched = autoExportTime.value.match(/^([01]\d|2[0-3]):([0-5]\d)$/)
  if (!matched) {
    return { hours: 0, minutes: 0 }
  }
  return { hours: Number(matched[1]), minutes: Number(matched[2]) }
}

function getCompletedTasksForRange(startMs: number, endMs: number) {
  return completedTasks.value
    .filter((task) => typeof task.completedAt === 'number' && task.completedAt >= startMs && task.completedAt < endMs)
    .map((task) => ({
      title: task.title,
      completedMarkdown: typeof task.completedMarkdown === 'string' && task.completedMarkdown.trim().length > 0
        ? task.completedMarkdown
        : buildCompletedMarkdown(task),
      completedAt: task.completedAt
    }))
}

async function runAutoExportForConfiguredDay(now: Date = new Date()) {
  if (!autoExportEnabled.value) return

  const { hours, minutes } = parseAutoExportTime()
  const triggerAt = new Date(now)
  triggerAt.setHours(hours, minutes, 0, 0)
  if (now.getTime() < triggerAt.getTime()) return

  const todayKey = getDateKey(now)
  const lastRunDay = localStorage.getItem(AUTO_EXPORT_RUN_KEY)
  if (lastRunDay === todayKey) return

  const dayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()
  const tasksToExport = getCompletedTasksForRange(dayStart, now.getTime())

  if (tasksToExport.length > 0) {
    try {
      await invoke<string>('export_completed_markdowns', { tasks: tasksToExport })
      console.info(`Auto export success: ${tasksToExport.length} tasks`)
    } catch (error) {
      console.error('Auto export failed:', error)
      return
    }
  }

  localStorage.setItem(AUTO_EXPORT_RUN_KEY, todayKey)
}

function scheduleNextAutoExport() {
  if (autoExportTimerId !== null) {
    window.clearTimeout(autoExportTimerId)
    autoExportTimerId = null
  }

  if (!autoExportEnabled.value) return

  const now = new Date()
  const { hours, minutes } = parseAutoExportTime()
  const nextTrigger = new Date(now)
  nextTrigger.setHours(hours, minutes, 0, 0)
  if (nextTrigger.getTime() <= now.getTime()) {
    nextTrigger.setDate(nextTrigger.getDate() + 1)
  }
  const delay = Math.max(1000, nextTrigger.getTime() - now.getTime())

  autoExportTimerId = window.setTimeout(async () => {
    await runAutoExportForConfiguredDay(new Date())
    scheduleNextAutoExport()
  }, delay)
}

async function applyAutoExportSettings() {
  const valid = /^([01]\d|2[0-3]):([0-5]\d)$/.test(autoExportTime.value)
  if (!valid) {
    autoExportTime.value = '00:00'
  }

  localStorage.setItem(AUTO_EXPORT_ENABLED_KEY, String(autoExportEnabled.value))
  localStorage.setItem(AUTO_EXPORT_TIME_KEY, autoExportTime.value)

  if (autoExportEnabled.value) {
    await runAutoExportForConfiguredDay(new Date())
  }
  scheduleNextAutoExport()
}

function syncToLocalStorage() {
  const allTasks = [...todoTasks.value, ...inProgressTasks.value, ...completedTasks.value]
  localStorage.setItem('tasks', JSON.stringify(allTasks))
}

function loadTasks() {
  const savedTasks = localStorage.getItem('tasks')
  if (!savedTasks) return

  try {
    const tasks = JSON.parse(savedTasks).map((task: Task & { tags?: string[]; priority?: '高' | '中' | '低'; category?: string; completedAt?: number; completedMarkdown?: string }) => {
      const legacyCategory = Array.isArray(task.tags)
        ? task.tags.map((tag) => tag.trim()).find((tag) => tag.length > 0) || ''
        : ''

      const normalizedTask: Task = {
        ...task,
        priority: task.priority && ['高', '中', '低'].includes(task.priority) ? task.priority : '中',
        category: typeof task.category === 'string' ? task.category.trim() : legacyCategory,
        completedAt: typeof task.completedAt === 'number' ? task.completedAt : undefined,
        startedAt: typeof task.startedAt === 'number' ? task.startedAt : undefined,
        endedAt: typeof task.endedAt === 'number' ? task.endedAt : undefined,
        completedMarkdown: typeof task.completedMarkdown === 'string' ? task.completedMarkdown : undefined,
        isPaused: Boolean(task.isPaused),
        pausedAt: typeof task.pausedAt === 'number' ? task.pausedAt : undefined,
        pauseLogs: Array.isArray(task.pauseLogs) ? task.pauseLogs : [],
        totalElapsedTime: typeof task.totalElapsedTime === 'number'
          ? task.totalElapsedTime
          : (task.totalActiveTime || 0) + (task.blockerLogs || []).reduce((acc, log) => acc + (log.duration || 0), 0)
      }

      if (normalizedTask.status === 'completed' && !normalizedTask.completedMarkdown) {
        normalizedTask.completedMarkdown = buildCompletedMarkdown(normalizedTask)
      }

      return normalizedTask
    }) as Task[]

    todoTasks.value = tasks.filter((t: Task) => t.status === 'todo')
    inProgressTasks.value = tasks.filter((t: Task) => t.status === 'in-progress')
    completedTasks.value = tasks.filter((t: Task) => t.status === 'completed')
  } catch (e) {
    console.error('Failed to load tasks:', e)
    todoTasks.value = []
    inProgressTasks.value = []
    completedTasks.value = []
  }
}

function addTask(status: 'todo' | 'in-progress' | 'completed') {
  const newTask: Task = {
    id: Date.now().toString(),
    title: '新任务',
    priority: '中',
    category: '',
    status,
    totalActiveTime: 0,
    totalElapsedTime: 0,
    isPaused: false,
    pauseLogs: [],
    blockerLogs: [],
    isBlocked: false
  }

  if (status === 'todo') {
    todoTasks.value.unshift(newTask)
  } else if (status === 'in-progress') {
    const now = Date.now()
    newTask.startTime = now
    newTask.startedAt = now
    inProgressTasks.value.push(newTask)
  } else {
    const now = Date.now()
    newTask.startedAt = now
    newTask.endedAt = now
    newTask.completedAt = now
    newTask.completedMarkdown = buildCompletedMarkdown(newTask)
    completedTasks.value.unshift(newTask)
  }

  syncToLocalStorage()
}

function startTask(taskId: string) {
  const taskIndex = todoTasks.value.findIndex((t) => t.id === taskId)
  if (taskIndex === -1) return

  const task = todoTasks.value[taskIndex]
  todoTasks.value.splice(taskIndex, 1)
  task.status = 'in-progress'
  const now = Date.now()
  task.startTime = now
  task.startedAt = now
  task.endedAt = undefined
  task.completedAt = undefined
  task.completedMarkdown = undefined
  task.isPaused = false
  task.pausedAt = undefined
  task.pauseLogs = []
  inProgressTasks.value.push(task)
  todoTasks.value = [...todoTasks.value]
  inProgressTasks.value = [...inProgressTasks.value]
  syncToLocalStorage()
}

function deleteTask(taskId: string) {
  const todoTaskIndex = todoTasks.value.findIndex((t) => t.id === taskId)
  if (todoTaskIndex !== -1) {
    todoTasks.value.splice(todoTaskIndex, 1)
    todoTasks.value = [...todoTasks.value]
    syncToLocalStorage()
    return
  }

  const completedTaskIndex = completedTasks.value.findIndex((t) => t.id === taskId)
  if (completedTaskIndex === -1) return
  completedTasks.value.splice(completedTaskIndex, 1)
  completedTasks.value = [...completedTasks.value]
  syncToLocalStorage()
}

function requestCompleteTask(taskId: string) {
  confirmTaskId.value = taskId
  confirmAction.value = 'complete-task'
  confirmMessage.value = '确认将该任务标记为已完成吗？'
  showConfirmDialog.value = true
}

function requestDeleteCompletedTask(taskId: string) {
  confirmTaskId.value = taskId
  confirmAction.value = 'delete-completed-task'
  confirmMessage.value = '确认删除该已完成任务吗？删除后不可恢复。'
  showConfirmDialog.value = true
}

function executeConfirmedAction() {
  if (!confirmTaskId.value || !confirmAction.value) {
    closeConfirmDialog()
    return
  }

  if (confirmAction.value === 'complete-task') {
    completeTask(confirmTaskId.value)
  } else if (confirmAction.value === 'delete-completed-task') {
    const completedTaskIndex = completedTasks.value.findIndex((t) => t.id === confirmTaskId.value)
    if (completedTaskIndex !== -1) {
      completedTasks.value.splice(completedTaskIndex, 1)
      completedTasks.value = [...completedTasks.value]
      syncToLocalStorage()
    }
  }

  closeConfirmDialog()
}

function completeTask(taskId: string) {
  const taskIndex = inProgressTasks.value.findIndex((t) => t.id === taskId)
  if (taskIndex === -1) return

  const task = inProgressTasks.value[taskIndex]
  const now = Date.now()
  inProgressTasks.value.splice(taskIndex, 1)
  task.status = 'completed'

  if (!task.isBlocked && task.startTime) {
    task.totalActiveTime += now - task.startTime
  }

  if (task.isBlocked && task.blockedStartTime) {
    task.blockerLogs.push({
      reason: task.currentBlockerReason || '未指定原因',
      duration: now - task.blockedStartTime
    })
  }

  if (task.isPaused) {
    const lastPause = task.pauseLogs[task.pauseLogs.length - 1]
    if (lastPause && !lastPause.resumedAt) {
      lastPause.resumedAt = now
    }
  }

  task.totalElapsedTime = task.totalActiveTime + task.blockerLogs.reduce((acc, log) => acc + log.duration, 0)
  task.completedAt = now
  task.endedAt = now
  task.completedMarkdown = buildCompletedMarkdown(task)

  task.isPaused = false
  task.pausedAt = undefined
  task.isBlocked = false
  task.startTime = undefined
  task.blockedStartTime = undefined
  task.currentBlockerReason = undefined
  completedTasks.value.unshift(task)
  inProgressTasks.value = [...inProgressTasks.value]
  completedTasks.value = [...completedTasks.value]
  syncToLocalStorage()
}

function editTaskTitle(taskId: string, newTitle: string) {
  let task = todoTasks.value.find((t) => t.id === taskId)
  if (!task) task = inProgressTasks.value.find((t) => t.id === taskId)
  if (!task) task = completedTasks.value.find((t) => t.id === taskId)
  if (!task) return
  task.title = newTitle
  if (task.status === 'completed') {
    task.completedMarkdown = buildCompletedMarkdown(task)
  }
  syncToLocalStorage()
}

function editTaskPriority(taskId: string, priority: '高' | '中' | '低') {
  const task = todoTasks.value.find((t) => t.id === taskId)
  if (!task) return
  task.priority = priority
  syncToLocalStorage()
}

function editTaskCategory(taskId: string, category: string) {
  const task = todoTasks.value.find((t) => t.id === taskId)
  if (!task) return
  task.category = category
  syncToLocalStorage()
}

function toggleBlock(taskId: string) {
  const taskIndex = inProgressTasks.value.findIndex((t) => t.id === taskId)
  if (taskIndex === -1) return

  const task = inProgressTasks.value[taskIndex]
  if (task.isPaused) {
    window.alert('任务已暂停，请先恢复后再设置阻塞')
    return
  }

  if (!task.isBlocked) {
    currentBlockingTaskId.value = taskId
    blockerReason.value = ''
    showBlockerDialog.value = true
    return
  }

  if (task.blockedStartTime) {
    const blockDuration = Date.now() - task.blockedStartTime
    task.blockerLogs.push({
      reason: task.currentBlockerReason || '未指定原因',
      duration: blockDuration
    })
    task.blockedStartTime = undefined
    task.currentBlockerReason = undefined
  }
  task.isBlocked = false
  task.startTime = Date.now()
  inProgressTasks.value = [...inProgressTasks.value]
  syncToLocalStorage()
}

function confirmBlockerReason() {
  if (!currentBlockingTaskId.value) {
    closeBlockerDialog()
    return
  }

  const taskIndex = inProgressTasks.value.findIndex((t) => t.id === currentBlockingTaskId.value)
  if (taskIndex === -1) {
    closeBlockerDialog()
    return
  }

  const task = inProgressTasks.value[taskIndex]
  if (task.startTime) {
    task.totalActiveTime += Date.now() - task.startTime
    task.startTime = undefined
  }

  task.isBlocked = true
  task.blockedStartTime = Date.now()
  task.currentBlockerReason = blockerReason.value
  inProgressTasks.value = [...inProgressTasks.value]
  syncToLocalStorage()
  closeBlockerDialog()
}

function togglePause(taskId: string) {
  const taskIndex = inProgressTasks.value.findIndex((t) => t.id === taskId)
  if (taskIndex === -1) return

  const task = inProgressTasks.value[taskIndex]
  const now = Date.now()

  if (task.isBlocked) {
    window.alert('任务阻塞中，恢复后才可暂停')
    return
  }

  if (!task.isPaused) {
    if (task.startTime) {
      task.totalActiveTime += now - task.startTime
      task.startTime = undefined
    }
    task.isPaused = true
    task.pausedAt = now
    task.pauseLogs.push({ pausedAt: now })
  } else {
    task.isPaused = false
    const lastPause = task.pauseLogs[task.pauseLogs.length - 1]
    if (lastPause && !lastPause.resumedAt) {
      lastPause.resumedAt = now
    }
    task.pausedAt = undefined
    task.startTime = now
  }

  inProgressTasks.value = [...inProgressTasks.value]
  syncToLocalStorage()
}

onMounted(() => {
  loadTasks()
  void applyAutoExportSettings()
})

onBeforeUnmount(() => {
  if (autoExportTimerId !== null) {
    window.clearTimeout(autoExportTimerId)
    autoExportTimerId = null
  }
})

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)

  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}

function formatCompletedDateForMarkdown(timestamp?: number): string {
  if (!timestamp) return 'N/A'
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', { hour12: false })
}

function buildCompletedMarkdown(task: Task): string {
  const blockedTotalTime = task.blockerLogs.reduce((acc, log) => acc + log.duration, 0)
  const pausedTotalTime = task.pauseLogs.reduce((acc, log) => acc + ((log.resumedAt || log.pausedAt) - log.pausedAt), 0)
  const blockedLines = task.blockerLogs.length > 0
    ? task.blockerLogs
      .map((log, index) => `${index + 1}. ${formatDuration(log.duration)} - ${log.reason}`)
      .join('\n')
    : '1. 无'
  const pauseLines = task.pauseLogs.length > 0
    ? task.pauseLogs
      .map((log, index) => {
        const pauseAt = formatCompletedDateForMarkdown(log.pausedAt)
        const resumeAt = log.resumedAt ? formatCompletedDateForMarkdown(log.resumedAt) : '未恢复'
        return `${index + 1}. 暂停: ${pauseAt} -> 恢复: ${resumeAt}`
      })
      .join('\n')
    : '1. 无'

  return [
    `# ${task.title}`,
    '',
    `- 状态: 已完成`,
    `- 优先级: ${task.priority}`,
    `- 分类: ${task.category || '未分类'}`,
    `- 开始时间: ${formatCompletedDateForMarkdown(task.startedAt)}`,
    `- 结束时间: ${formatCompletedDateForMarkdown(task.endedAt)}`,
    `- 完成时间: ${formatCompletedDateForMarkdown(task.completedAt)}`,
    `- 总时间: ${formatDuration(task.totalElapsedTime)}`,
    `- Focus 时间: ${formatDuration(task.totalActiveTime)}`,
    `- 阻塞总时间: ${formatDuration(blockedTotalTime)}`,
    `- 暂停总时长: ${formatDuration(pausedTotalTime)}`,
    '',
    '## 阻塞点记录',
    blockedLines,
    '',
    '## 暂停记录',
    pauseLines
  ].join('\n')
}
</script>

<style scoped>
.app-shell {
  width: 100%;
  height: 100%;
  padding: 1.15rem 1.2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  background: var(--surface);
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 1rem;
  padding: 0.75rem 1rem;
  border-radius: 1.2rem;
  background: rgba(166, 196, 255, 0.25);
  backdrop-filter: blur(14px);
}

.top-actions {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  min-width: 0;
  margin-left: auto;
}

.completed-tools {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  flex-wrap: wrap;
}

.auto-export-toggle,
.auto-export-time {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  padding: 0.36rem 0.56rem;
  border-radius: 9999px;
  background: rgba(65, 95, 147, 0.1);
  color: #345386;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 0.76rem;
  font-weight: 700;
  white-space: nowrap;
}

.auto-export-toggle input[type='checkbox'] {
  width: 0.86rem;
  height: 0.86rem;
  accent-color: #345386;
}

.auto-export-time input[type='time'] {
  border: 0;
  outline: 0;
  border-radius: 9999px;
  padding: 0.2rem 0.42rem;
  font-size: 0.74rem;
  color: #345386;
  background: rgba(255, 255, 255, 0.78);
  font-family: 'Plus Jakarta Sans', sans-serif;
  min-width: 5.6rem;
}

.auto-export-time input[type='time']:disabled {
  opacity: 0.62;
}

.export-btn {
  border: 0;
  border-radius: 9999px;
  padding: 0.5rem 0.85rem;
  background: linear-gradient(90deg, rgba(65, 95, 147, 0.2), rgba(52, 83, 134, 0.16));
  color: #345386;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-weight: 700;
  font-size: 0.78rem;
  cursor: pointer;
  white-space: nowrap;
}

.export-btn:disabled {
  opacity: 0.68;
  cursor: progress;
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.brand-dot {
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 9999px;
  background: var(--primary);
  box-shadow: 0 0 0 6px rgba(65, 95, 147, 0.18);
}

.brand h1 {
  margin: 0;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-weight: 800;
  font-size: 1.15rem;
}

.tabs {
  display: flex;
  align-items: center;
  gap: 1.15rem;
}

.tab {
  color: #596065;
  text-decoration: none;
  font-weight: 600;
  font-size: 0.95rem;
}

.tab--active {
  color: var(--primary);
  font-weight: 700;
}

.search-input {
  border: 0;
  outline: 0;
  border-radius: 9999px;
  background: var(--surface-container-highest);
  color: var(--on-surface);
  padding: 0.55rem 0.9rem;
  width: min(18rem, 100%);
  max-width: 100%;
}

.board {
  flex: 1;
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  grid-template-rows: minmax(0, 1fr);
  gap: 1rem;
  overflow-x: hidden;
  overflow-y: hidden;
  align-content: start;
}

.lane {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  background: var(--surface-container);
  border-radius: 1.4rem;
  padding: 1rem;
  overflow: hidden;
}

.lane--progress {
  background: linear-gradient(180deg, rgba(166, 196, 255, 0.14), rgba(234, 238, 243, 0.7));
}

.lane-head h2 {
  margin: 0;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-size: 1.24rem;
  color: var(--on-surface);
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.lane-head span {
  font-size: 0.75rem;
  color: #415f93;
  background: rgba(166, 196, 255, 0.35);
  border-radius: 9999px;
  padding: 0.12rem 0.42rem;
}

.live-dot {
  width: 0.48rem;
  height: 0.48rem;
  padding: 0;
  border-radius: 9999px;
  background: #415f93;
  display: inline-block;
  opacity: 0.28;
}

.live-dot-group {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
}

.live-dot--active {
  animation: breathe-dot 2.6s ease-in-out infinite;
}

@keyframes breathe-dot {
  0% {
    opacity: 0.35;
    box-shadow: 0 0 0 0 rgba(65, 95, 147, 0.26);
    transform: scale(1);
  }
  50% {
    opacity: 0.9;
    box-shadow: 0 0 0 8px rgba(65, 95, 147, 0.1);
    transform: scale(1.05);
  }
  100% {
    opacity: 0.35;
    box-shadow: 0 0 0 0 rgba(65, 95, 147, 0.2);
    transform: scale(1);
  }
}

.add-btn {
  border: 0;
  border-radius: 9999px;
  padding: 0.62rem 0.95rem;
  font-family: 'Plus Jakarta Sans', sans-serif;
  font-weight: 700;
  color: #415f93;
  background: linear-gradient(90deg, rgba(65, 95, 147, 0.18), rgba(52, 83, 134, 0.13));
  cursor: pointer;
}

.lane-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  display: grid;
  grid-auto-rows: max-content;
  align-content: start;
  align-items: start;
  gap: 0.8rem;
  padding-right: 0.15rem;
  touch-action: pan-y;
}

.lane--compact .lane-list {
  gap: 0.55rem;
}

.lane-list > * {
  min-width: 0;
}

.lane-empty {
  border-radius: 1rem;
  padding: 1rem;
  color: #596065;
  background: rgba(220, 227, 233, 0.62);
  text-align: center;
  font-size: 0.9rem;
}

.modal-wrap {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.modal-backdrop {
  position: absolute;
  inset: 0;
  border: 0;
  background: rgba(16, 24, 40, 0.45);
}

.modal-panel {
  position: relative;
  z-index: 1;
  width: min(26rem, 100%);
  border-radius: 1.4rem;
  padding: 1.2rem;
  background: #f7f9fc;
  box-shadow: 0 40px 40px rgba(44, 51, 56, 0.16);
}

.modal-panel h3 {
  margin: 0 0 0.7rem;
  font-family: 'Plus Jakarta Sans', sans-serif;
}

.confirm-text {
  margin: 0;
  color: #596065;
  line-height: 1.5;
}

.modal-panel input {
  width: 100%;
  border: 0;
  outline: 0;
  border-radius: 0.8rem;
  background: #e3e9ee;
  padding: 0.55rem 0.7rem;
  color: #2c3338;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 0.85rem;
}

.modal-btn {
  border: 0;
  border-radius: 9999px;
  padding: 0.46rem 0.95rem;
  font-weight: 700;
  cursor: pointer;
}

.modal-btn--muted {
  background: #dce3e9;
  color: #2c3338;
}

.modal-btn--primary {
  background: linear-gradient(90deg, #415f93, #345386);
  color: #f8f8ff;
}

@media (max-width: 1100px) {
  .tabs {
    display: none;
  }

  .search-input {
    width: 10rem;
  }
}

</style>
