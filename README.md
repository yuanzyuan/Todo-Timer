# To do Timer

一个基于 **Tauri + Vue 3 + TypeScript** 的桌面任务看板应用，聚焦于：

- 任务推进（Todo / In Progress / Completed）
- 专注计时（Focus / Block / Pause）
- 阻塞管理与记录沉淀（支持导出 Markdown）

## 核心能力

- 三列并行看板：`Todo` / `In Progress` / `Completed`
- 任务标签体系：优先级（高/中/低）+ 分类（文本）
- 标签编辑权限：仅 `Todo` 列可编辑，其他列只读
- `In Progress` 状态机：
  - `Focus Session`（正常推进）
  - `Block Session`（阻塞阶段，显示阻塞时长）
  - `Pause / Resume`（暂停与恢复，记录时间点）
- 计时统计：
  - Focus 时间
  - 阻塞总时间
  - 任务总时间（Focus + Block）
- 完成流程：
  - 二次确认后进入 `Completed`
  - 记录任务开始时间与结束时间
- 导出能力：
  - 手动导出 Completed 任务为真实 `.md` 文件
  - 可配置自动导出开关与每日导出时间
  - macOS 下支持访达目录选择
- 数据持久化：本地 `localStorage`

## 技术栈

- 前端：Vue 3 + TypeScript
- 桌面容器：Tauri 2
- 构建：Vite
- 后端命令：Rust（Tauri command）
- 样式：自定义 CSS（结合设计变量）

## 快速开始

### 1) 环境要求

- Node.js 18+
- Rust（stable）
- Tauri 开发依赖（macOS）

### 2) 安装依赖

```bash
npm install
```

### 3) 本地开发（桌面运行）

```bash
npm run tauri dev
```

### 4) 构建发布

```bash
npm run tauri build
```

## 导出与数据

### 导出 Markdown

- 在 `Completed` 列点击 `导出MD`
- 选择目录后，应用会将已完成任务写出为多个 `.md` 文件
- 文件名包含时间戳与任务标题，便于检索

### 自动导出

- 可在 `Completed` 工具区设置：
  - 自动导出开关
  - 每日导出时间（`HH:mm`）
- 到达设定时间后自动导出当天已完成任务

### 数据存储说明

- 业务数据：浏览器 `localStorage`
- 导出文件默认目录：`~/Documents/todo-timer-exports`

## 项目结构

```text
todo-timer-plugin/
├── src/
│   ├── App.vue
│   ├── style.css
│   └── components/
│       └── TaskCard.vue
├── src-tauri/
│   ├── src/
│   │   └── lib.rs
│   └── tauri.conf.json
├── package.json
└── README.md
```

## 说明与边界

- 当前版本为本地优先方案，不含账号登录与云同步
- 若清理浏览器存储，历史任务数据会被移除
- 自动导出依赖应用运行状态（应用关闭时不会触发）

## Roadmap（可选）

- 屏幕上下文驱动的阻塞自动识别（规则优先）
- 云同步与多端数据一致性
- 更细粒度的统计报表（按天/周/月）
