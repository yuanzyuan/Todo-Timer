<template>
  <div class="app-shell">
    <header class="topbar">
      <div class="brand">
        <span class="brand-dot"></span>
        <h1>Todo Timer Plugin</h1>
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
      <section class="lane">
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
            @edit-title="editTaskTitle"
            @edit-priority="editTaskPriority"
            @edit-category="editTaskCategory"
          />
          <div v-if="todoTasks.length === 0" class="lane-empty">No todo task yet</div>
        </div>
      </section>

      <section class="lane lane--progress">
        <div class="lane-head">
          <h2>In Progress <span class="live-dot"></span></h2>
        </div>
        <div class="lane-list">
          <TaskCard
            v-for="(task, index) in inProgressTasks"
            :key="task.id"
            :task="task"
            :hero="index === 0"
            :can-edit-tags="false"
            @toggle-block="toggleBlock"
            @complete-task="completeTask"
            @edit-title="editTaskTitle"
            @edit-priority="editTaskPriority"
            @edit-category="editTaskCategory"
          />
          <div v-if="inProgressTasks.length === 0" class="lane-empty">Move a task here to start focus</div>
        </div>
      </section>

      <section class="lane">
        <div class="lane-head">
          <h2>Completed <span>✓</span></h2>
        </div>
        <div class="lane-list">
          <TaskCard
            v-for="task in completedTasks"
            :key="task.id"
            :task="task"
            :can-edit-tags="false"
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
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import TaskCard from './components/TaskCard.vue'

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

const todoTasks = ref<Task[]>([])
const inProgressTasks = ref<Task[]>([])
const completedTasks = ref<Task[]>([])

const showBlockerDialog = ref(false)
const blockerReason = ref('')
const currentBlockingTaskId = ref<string | null>(null)

function closeBlockerDialog() {
  showBlockerDialog.value = false
  currentBlockingTaskId.value = null
  blockerReason.value = ''
}

function syncToLocalStorage() {
  const allTasks = [...todoTasks.value, ...inProgressTasks.value, ...completedTasks.value]
  localStorage.setItem('tasks', JSON.stringify(allTasks))
}

function loadTasks() {
  const savedTasks = localStorage.getItem('tasks')
  if (!savedTasks) return

  try {
    const tasks = JSON.parse(savedTasks).map((task: Task & { tags?: string[]; priority?: '高' | '中' | '低'; category?: string }) => {
      const legacyCategory = Array.isArray(task.tags)
        ? task.tags.map((tag) => tag.trim()).find((tag) => tag.length > 0) || ''
        : ''

      return {
        ...task,
        priority: task.priority && ['高', '中', '低'].includes(task.priority) ? task.priority : '中',
        category: typeof task.category === 'string' ? task.category.trim() : legacyCategory,
        totalElapsedTime: typeof task.totalElapsedTime === 'number'
          ? task.totalElapsedTime
          : (task.totalActiveTime || 0) + (task.blockerLogs || []).reduce((acc, log) => acc + (log.duration || 0), 0)
      }
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
    blockerLogs: [],
    isBlocked: false
  }

  if (status === 'todo') {
    todoTasks.value.unshift(newTask)
  } else if (status === 'in-progress') {
    newTask.startTime = Date.now()
    inProgressTasks.value.push(newTask)
  } else {
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
  task.startTime = Date.now()
  inProgressTasks.value.push(task)
  todoTasks.value = [...todoTasks.value]
  inProgressTasks.value = [...inProgressTasks.value]
  syncToLocalStorage()
}

function completeTask(taskId: string) {
  const taskIndex = inProgressTasks.value.findIndex((t) => t.id === taskId)
  if (taskIndex === -1) return

  const task = inProgressTasks.value[taskIndex]
  inProgressTasks.value.splice(taskIndex, 1)
  task.status = 'completed'

  if (!task.isBlocked && task.startTime) {
    task.totalActiveTime += Date.now() - task.startTime
  }

  if (task.isBlocked && task.blockedStartTime) {
    task.blockerLogs.push({
      reason: task.currentBlockerReason || '未指定原因',
      duration: Date.now() - task.blockedStartTime
    })
  }

  task.totalElapsedTime = task.totalActiveTime + task.blockerLogs.reduce((acc, log) => acc + log.duration, 0)

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

onMounted(() => {
  loadTasks()
})
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
  min-width: 0;
  margin-left: auto;
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
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1rem;
  overflow-x: hidden;
  align-content: start;
}

.lane {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  background: var(--surface-container);
  border-radius: 1.4rem;
  padding: 1rem;
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
  gap: 0.8rem;
  padding-right: 0.15rem;
  touch-action: pan-y;
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
