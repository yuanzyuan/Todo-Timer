# Todo Timer Plugin

轻量级桌面 Todo 计时插件，帮助用户高效管理任务并追踪时间消耗。

## 技术栈
- **前端**：Vue 3 + TypeScript + Tailwind CSS
- **桌面框架**：Tauri
- **状态管理**：Vue 3 Composition API + localStorage
- **拖拽功能**：vuedraggable
- **构建工具**：Vite

## 核心功能
1. **看板结构**：三列布局（待办、进行中、已完成），支持卡片拖拽
2. **智能计时**：任务拖入"进行中"自动开始计时
3. **阻塞功能**：支持记录任务阻塞原因和时长
4. **视觉效果**：毛玻璃效果，进行中任务呼吸灯边框，阻塞状态红色背景
5. **持久化**：使用 localStorage 实时保存任务状态

## 如何运行

### 前置条件
- Node.js 16+
- Rust 1.70+
- Tauri CLI

### 安装依赖
```bash
npm install
```

### 开发模式运行
```bash
npm run tauri dev
```

### 构建生产版本
```bash
npm run tauri build
```

## 如何测试

### 功能测试
1. **创建任务**：点击"待办"列下方的"+ 添加任务"按钮
2. **任务拖拽**：将任务卡片在不同列之间拖拽，观察状态变化
3. **计时测试**：
   - 将任务拖入"进行中"列，观察计时开始
   - 将任务拖入"已完成"列，观察计时停止
4. **阻塞测试**：
   - 在"进行中"任务上点击"阻塞"按钮
   - 输入阻塞原因并确认
   - 观察任务状态变为阻塞
   - 点击"恢复"按钮，观察阻塞时长记录
5. **持久化测试**：
   - 创建并编辑任务
   - 关闭应用并重新打开
   - 确认任务状态和计时数据已保存

### 视觉测试
- 观察毛玻璃效果是否正常
- 观察进行中任务的呼吸灯边框动画
- 观察阻塞状态任务的红色背景

## 项目结构
```
todo-timer-plugin/
├── src/
│   ├── components/
│   │   └── TaskCard.vue        # 任务卡片组件
│   ├── App.vue                  # 主应用组件
│   ├── main.ts                  # 应用入口
│   └── style.css                # 全局样式
├── index.html                   # HTML 入口
├── package.json                 # 项目配置
├── tauri.conf.json              # Tauri 配置
├── tsconfig.json                # TypeScript 配置
├── vite.config.ts               # Vite 配置
├── tailwind.config.js           # Tailwind CSS 配置
└── postcss.config.js            # PostCSS 配置
```

## 数据结构

### 任务对象
```typescript
interface Task {
  id: string;                    // 唯一标识符
  title: string;                 // 任务标题
  status: 'todo' | 'in-progress' | 'completed';  // 任务状态
  totalActiveTime: number;       // 总活跃时间（毫秒）
  blockerLogs: BlockerLog[];     // 阻塞记录数组
  startTime?: number;            // 开始时间戳
  blockedStartTime?: number;     // 阻塞开始时间戳
  isBlocked: boolean;            // 是否处于阻塞状态
}

interface BlockerLog {
  reason: string;                // 阻塞原因
  duration: number;              // 阻塞时长（毫秒）
}
```

## 窗口设置
- 无边框（frameless）
- 背景透明（transparent）
- 始终置顶（always on top）
- 顶部隐藏拖拽区域（data-tauri-drag-region）

## 注意事项
- 本插件使用 localStorage 存储数据，数据仅保存在本地
- 建议定期备份 localStorage 数据以防止丢失
- 插件支持多次阻塞记录，每次阻塞都会生成一个新的记录
- 任务完成后，计时会自动停止并计算总活跃时间