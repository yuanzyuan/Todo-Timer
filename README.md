# Todo Timer Plugin

一个基于 Tauri + Vue 3 的桌面任务看板应用，聚焦「任务推进 + 专注计时 + 阻塞记录」。

## 功能概览
- 三列看板：`Todo` / `In Progress` / `Completed`
- `Todo` 列支持新建与删除任务
- 任务支持优先级（高/中/低）与分类标签
- 标签仅允许在 `Todo` 列编辑，其他列只读展示
- `In Progress` 提供两类时间：
  - `Session`：当前专注段时长（每次解除阻塞后重新计）
  - `Total`：任务累计总时长（包含阻塞时间）
- 支持阻塞原因记录与阻塞日志
- 完成任务后自动进入 `Completed`，并保留总时长
- 标题右侧进度点会按进行中任务数量动态显示并呼吸闪动
- 所有数据持久化到本地 `localStorage`

## 技术栈
- 前端：Vue 3 + TypeScript
- 桌面框架：Tauri 2
- 构建工具：Vite
- 样式：Tailwind CSS（结合自定义 CSS 设计系统）

## 运行方式

### 前置要求
- Node.js 16+
- Rust 1.70+
- Tauri CLI

### 安装依赖
```bash
npm install
```

### 本地开发（桌面应用）
```bash
npm run tauri dev
```

### 构建发布包
```bash
npm run tauri build
```

## 数据模型

```ts
interface Task {
  id: string
  title: string
  priority: '高' | '中' | '低'
  category: string
  status: 'todo' | 'in-progress' | 'completed'
  totalActiveTime: number       // 纯活跃时长（不含阻塞）
  totalElapsedTime: number      // 总时长（含阻塞）
  blockerLogs: BlockerLog[]
  startTime?: number
  blockedStartTime?: number
  isBlocked: boolean
  currentBlockerReason?: string
}

interface BlockerLog {
  reason: string
  duration: number
}
```

## 关键交互说明
- 新建任务默认进入 `Todo` 顶部
- 完成任务默认进入 `Completed` 顶部
- 点击 `开始`：`Todo -> In Progress`
- 点击 `阻塞`：输入原因后进入阻塞态，活跃计时暂停
- 点击 `恢复`：开始新的 Session 计时段
- 点击 `完成`：任务结束，结算总时长并写入完成列

## 项目结构
```text
todo-timer-plugin/
├── src/
│   ├── App.vue
│   ├── main.ts
│   ├── style.css
│   └── components/
│       └── TaskCard.vue
├── src-tauri/
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── package.json
└── README.md
```

## 注意事项
- 数据仅保存在本机浏览器存储（`localStorage`），卸载或清理数据会丢失记录
- 当前版本未接入远程账户与云同步
- 推荐使用 `npm run tauri dev` 进行桌面端功能验证
