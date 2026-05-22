# Technical Design

## Architecture

单体 Rust 应用，服务端渲染，单二进制部署。

```text
浏览器 (SSH 隧道)
  ↓
Axum HTTP Server
  ├── Askama 模板渲染 HTML
  ├── HTMX 处理交互（表单提交、局部刷新、状态轮询）
  ├── SQLite 数据持久化
  └── Slurm Worker（sbatch 提交 + sacct 轮询）
        ↓
      HPC 计算节点执行 R 脚本
        ↓
      共享文件系统读写结果
```

---

## 技术栈

| 层 | 技术 |
|----|------|
| HTTP 框架 | Axum + Tokio |
| 模板引擎 | Askama |
| 前端交互 | HTMX |
| 样式 | Tailwind CSS（构建时编译为静态 CSS） |
| 数据库 | SQLite（通过 sqlx） |
| 计算调度 | Slurm CLI（sbatch, sacct, scancel） |
| R 执行 | Rscript（在计算节点上） |

---

## 数据模型

### Project

分析项目，包含脚本节点实例和运行历史。

字段：id, name, description, created_at

### PipelineNode

可复用的脚本节点定义。每个节点绑定一个 R 脚本。

字段：id, name, script_path, params_schema(json), inputs_schema(json), outputs_schema(json), created_at

通过解析 R 脚本注解自动填充 schema。

### ProjectFlow

项目中的脚本节点实例，由多个节点按顺序串联。

字段：id, project_id, name, created_at

### ProjectFlowStep

脚本节点中的一个步骤，引用一个 PipelineNode。

字段：id, flow_id, node_id, step_order, param_values(json)

### FlowRun

一次脚本节点执行。

字段：id, flow_id, status(pending/running/paused/completed/failed), current_step, created_at, started_at, finished_at

### StepRun

单个步骤的执行记录。

字段：id, flow_run_id, step_order, status, slurm_job_id, stdout, stderr, started_at, finished_at

---

## 文件管理

直接操作用户 home 目录下的文件系统，不使用数据库跟踪文件。

操作：
- 列出目录内容（ls）
- 上传文件（multipart）
- 下载文件
- 创建目录
- 删除文件/目录
- 重命名

根目录限制为 `$HOME` 或配置的 `data-dir`，防止路径穿越。

---

## R 脚本注解规范

```r
#' @title DEG Analysis
#' @description Differential expression using DESeq2
#' @param method character "DESeq2" Analysis method
#' @param pvalue_cutoff number 0.05 P-value threshold
#' @input counts_matrix.csv Raw count matrix
#' @output deg_results.csv Results table
#' @output volcano_plot.png Volcano plot
```

解析后生成节点的 params/inputs/outputs schema。

---

## Slurm 集成

### 提交作业

```bash
sbatch --job-name=ripeline_{run_id}_{step} \
       --output={run_dir}/stdout.log \
       --error={run_dir}/stderr.log \
       --wrap="cd {run_dir} && Rscript {script_path}"
```

### 状态轮询

```bash
sacct -j {job_id} --format=State --noheader --parsable2
```

### 取消作业

```bash
scancel {job_id}
```

---

## 脚本节点执行逻辑

1. 用户点击"执行"→ 创建 FlowRun，状态 running
2. Worker 按 step_order 依次提交作业
3. 每个步骤：写 params.json + inputs.json → sbatch → 轮询状态
4. 步骤完成后，outputs 目录内容作为下一步的 inputs
5. 步骤失败 → FlowRun 状态变为 paused，等待用户操作
6. 用户点击"暂停" → scancel 当前作业，FlowRun 状态变为 paused
7. 用户点击已执行的步骤 → 回退：删除该步骤及之后的 StepRun 记录，重置 current_step
8. 用户点击"继续" → 从 current_step 恢复执行

---

## 页面结构

| 路径 | 页面 | 说明 |
|------|------|------|
| `/login` | 登录 | 管理员密码 |
| `/projects` | 项目列表 | 创建/删除项目 |
| `/projects/{id}` | 项目详情 | 脚本节点配置 + 运行控制 |
| `/pipelines` | 节点管理 | 管理所有可用的 R 脚本节点 |
| `/files` | 文件管理 | 浏览/上传/下载文件 |

---

## HTMX 交互模式

- 表单提交：`hx-post` + `hx-target` 局部替换
- 状态轮询：`hx-trigger="every 2s"` 轮询运行状态
- 节点操作：`hx-post="/flows/{id}/steps/{n}/start"` 触发单步执行
- 文件浏览：`hx-get="/files?path=..."` 目录导航

---

## 认证

单用户模式。启动时通过环境变量或配置文件设置管理员密码。

Cookie-based session，无需 JWT 或 OAuth。

---

## 构建与部署

```bash
# 构建 Tailwind CSS
npx tailwindcss -i static/input.css -o static/style.css --minify

# 构建 Linux 二进制
cargo build --release --target x86_64-unknown-linux-gnu

# 部署
scp target/release/ripeline user@login-node:~/
ssh user@login-node "PORT=9000 ./ripeline run"
```

发布打包统一通过 `xtask` 执行：

```bash
cargo pkg-linux-x86_64
cargo pkg-macos-aarch64
cargo pkg-windows-x86_64
```

对应产物：

- `pkg/ripeline-x86_64-unknown-linux-gnu.tar.gz`
- `pkg/ripeline-aarch64-apple-darwin.tar.gz`
- `pkg/ripeline-x86_64-pc-windows-msvc.zip`

---

## 非目标

- 多用户权限系统
- 前后端分离
- 容器化部署
- 分布式调度
- Notebook 执行
- 实时协作
