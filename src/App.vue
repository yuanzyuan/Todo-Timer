<template>
  <div class="glassmorphism w-full h-full p-4 flex flex-col">
    <!-- 主标题 -->
    <h1 class="text-xl font-bold mb-4 text-center">Todo Timer Plugin</h1>
    
    <!-- 看板容器 -->
    <div class="flex gap-4 flex-1 min-h-0">
      <!-- 待办列 -->
      <div class="flex-1 flex flex-col min-h-0">
        <div class="flex items-center justify-between mb-3 gap-2">
          <h2 class="text-lg font-semibold">待办</h2>
          <button
            @click="addTask('todo')"
            class="px-3 py-1 bg-primary/50 hover:bg-primary/70 rounded-md transition-colors whitespace-nowrap"
          >
            + 创建任务
          </button>
        </div>
        <div class="space-y-2 overflow-y-auto flex-1 min-h-0">
          <div
            v-if="todoTasks.length === 0"
            class="h-full min-h-24 flex flex-col items-center justify-center gap-3 text-sm text-white/80"
          >
            <span>还没有待办任务</span>
            <button
              @click="addTask('todo')"
              class="px-3 py-2 bg-primary/50 hover:bg-primary/70 rounded-md transition-colors"
            >
              + 创建任务
            </button>
          </div>
          <TaskCard 
            v-for="task in todoTasks" 
            :key="task.id"
            :task="task" 
            @start-task="startTask"
            @edit-title="editTaskTitle"
          />
        </div>
      </div>
      
      <!-- 进行中列 -->
      <div class="flex-1 flex flex-col min-h-0">
        <h2 class="text-lg font-semibold mb-3">进行中</h2>
        <div class="space-y-2 overflow-y-auto flex-1 min-h-0">
          <TaskCard 
            v-for="task in inProgressTasks" 
            :key="task.id"
            :task="task" 
            @toggle-block="toggleBlock"
            @complete-task="completeTask"
            @edit-title="editTaskTitle"
          />
        </div>
      </div>
      
      <!-- 已完成列 -->
      <div class="flex-1 flex flex-col min-h-0">
        <h2 class="text-lg font-semibold mb-3">已完成</h2>
        <div class="space-y-2 overflow-y-auto flex-1 min-h-0">
          <TaskCard 
            v-for="task in completedTasks" 
            :key="task.id"
            :task="task" 
            @edit-title="editTaskTitle"
          />
        </div>
      </div>
    </div>
    
    <!-- 阻塞原因输入弹窗 -->
    <Teleport to="body">
      <div v-if="showBlockerDialog" class="fixed inset-0 z-[10000] flex items-center justify-center p-4">
        <div class="absolute inset-0 z-0 bg-black/65" @click="closeBlockerDialog"></div>
        <div class="relative z-10 w-full max-w-sm p-6 rounded-lg border border-white/40 bg-slate-900 shadow-2xl">
          <h3 class="text-lg font-semibold mb-3 text-white">输入阻塞原因</h3>
          <input 
            v-model="blockerReason" 
            type="text" 
            placeholder="请输入阻塞原因" 
            class="w-full p-2 text-white placeholder-white/60 bg-slate-800 border border-white/30 rounded-md mb-4"
          />
          <div class="flex justify-end gap-2">
            <button 
              @click="closeBlockerDialog" 
              class="px-3 py-1 bg-gray-500/70 hover:bg-gray-500 rounded-md transition-colors"
            >
              取消
            </button>
            <button 
              @click="confirmBlockerReason" 
              class="px-3 py-1 bg-primary hover:bg-primary/80 rounded-md transition-colors"
            >
              确认
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TaskCard from './components/TaskCard.vue'

// 类型定义
interface BlockerLog {
  reason: string
  duration: number // in milliseconds
}

interface Task {
  id: string
  title: string
  status: 'todo' | 'in-progress' | 'completed'
  totalActiveTime: number // in milliseconds
  blockerLogs: BlockerLog[]
  startTime?: number // timestamp when task started
  blockedStartTime?: number // timestamp when task was blocked
  isBlocked: boolean
  currentBlockerReason?: string // current blocker reason
}

// 状态管理
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

// 将任务同步到 localStorage
function syncToLocalStorage() {
  const allTasks = [...todoTasks.value, ...inProgressTasks.value, ...completedTasks.value]
  localStorage.setItem('tasks', JSON.stringify(allTasks))
}

// 从 localStorage 加载任务
function loadTasks() {
  const savedTasks = localStorage.getItem('tasks')
  if (savedTasks) {
    try {
      const tasks = JSON.parse(savedTasks)
      todoTasks.value = tasks.filter((t: Task) => t.status === 'todo')
      inProgressTasks.value = tasks.filter((t: Task) => t.status === 'in-progress')
      completedTasks.value = tasks.filter((t: Task) => t.status === 'completed')
    } catch (e) {
      console.error('Failed to load tasks:', e)
      // 重置任务列表
      todoTasks.value = []
      inProgressTasks.value = []
      completedTasks.value = []
    }
  }
}

// 添加任务
function addTask(status: 'todo' | 'in-progress' | 'completed') {
  const newTask: Task = {
    id: Date.now().toString(),
    title: `新任务`,
    status,
    totalActiveTime: 0,
    blockerLogs: [],
    isBlocked: false
  }
  
  if (status === 'todo') {
    todoTasks.value.push(newTask)
  } else if (status === 'in-progress') {
    newTask.startTime = Date.now() // 为进行中任务设置开始时间
    inProgressTasks.value.push(newTask)
  } else {
    completedTasks.value.push(newTask)
  }
  
  syncToLocalStorage()
}

// 开始任务
function startTask(taskId: string) {
  const taskIndex = todoTasks.value.findIndex(t => t.id === taskId)
  if (taskIndex !== -1) {
    const task = todoTasks.value[taskIndex]
    
    // 从待办数组中移除
    todoTasks.value.splice(taskIndex, 1)
    
    // 更新任务状态
    task.status = 'in-progress'
    task.startTime = Date.now()
    
    // 添加到进行中数组
    inProgressTasks.value.push(task)
    
    // 触发响应式更新
    todoTasks.value = [...todoTasks.value]
    inProgressTasks.value = [...inProgressTasks.value]
    
    syncToLocalStorage()
  }
}

// 完成任务
function completeTask(taskId: string) {
  const taskIndex = inProgressTasks.value.findIndex(t => t.id === taskId)
  if (taskIndex !== -1) {
    const task = inProgressTasks.value[taskIndex]
    
    // 从进行中数组中移除
    inProgressTasks.value.splice(taskIndex, 1)
    
    // 更新任务状态和时间
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

    task.isBlocked = false
    task.startTime = undefined
    task.blockedStartTime = undefined
    task.currentBlockerReason = undefined
    
    // 添加到已完成数组
    completedTasks.value.push(task)
    
    // 触发响应式更新
    inProgressTasks.value = [...inProgressTasks.value]
    completedTasks.value = [...completedTasks.value]
    
    syncToLocalStorage()
  }
}

// 编辑任务标题
function editTaskTitle(taskId: string, newTitle: string) {
  let task = todoTasks.value.find(t => t.id === taskId)
  if (!task) task = inProgressTasks.value.find(t => t.id === taskId)
  if (!task) task = completedTasks.value.find(t => t.id === taskId)
  
  if (task) {
    task.title = newTitle
    syncToLocalStorage()
  }
}

// 切换阻塞状态
function toggleBlock(taskId: string) {
  const taskIndex = inProgressTasks.value.findIndex(t => t.id === taskId)
  if (taskIndex === -1) return
  
  const task = inProgressTasks.value[taskIndex]
  
  if (!task.isBlocked) {
    // 开始阻塞
    currentBlockingTaskId.value = taskId
    blockerReason.value = ''
    showBlockerDialog.value = true
  } else {
    // 结束阻塞
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
    task.startTime = Date.now() // 重新开始计时
    
    // 触发响应式更新
    inProgressTasks.value = [...inProgressTasks.value]
    
    syncToLocalStorage()
  }
}

// 确认阻塞原因
function confirmBlockerReason() {
  if (currentBlockingTaskId.value) {
    const taskIndex = inProgressTasks.value.findIndex(t => t.id === currentBlockingTaskId.value)
    if (taskIndex === -1) return
    
    const task = inProgressTasks.value[taskIndex]
    
    // 计算已用时间
    if (task.startTime) {
      task.totalActiveTime += Date.now() - task.startTime
      task.startTime = undefined
    }
    
    // 更新任务状态
    task.isBlocked = true
    task.blockedStartTime = Date.now()
    task.currentBlockerReason = blockerReason.value
    
    // 触发响应式更新
    inProgressTasks.value = [...inProgressTasks.value]
    
    syncToLocalStorage()
  }
  
  closeBlockerDialog()
}

// 生命周期
onMounted(() => {
  loadTasks()
})
</script>
