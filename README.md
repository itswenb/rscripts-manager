# Ripeline

Ripeline 是一个面向生物信息学的 R 脚本脚本节点管理平台，部署在 HPC 登录节点上，通过 Slurm 调度计算任务到集群执行。

单二进制部署，无外部依赖（除 SQLite）。

---

## 核心功能

### 1. 项目管理

以项目为单位组织分析工作。每个项目包含独立的脚本节点配置和运行历史。

### 2. 脚本节点（Pipeline）

核心功能。用户在"脚本节点"标签页中管理所有可用的 R 脚本节点：

- 每个节点绑定一个 R 脚本
- 通过脚本注解（`#' @param`、`#' @input`、`#' @output`）自动解析参数和输入输出
- 用户可查看每个节点的输入/输出定义

在项目中，用户将节点串联成完整的分析流程，支持：

- Steps 组件展示每个节点的输入/输出
- 控制台实时显示运行日志
- 整体执行/暂停
- 单节点开始/停止
- 点击已执行节点时回退到该节点重新执行
- 节点失败后自动暂停

### 3. 文件管理

管理当前用户 home 目录下的文件：

- 浏览目录结构
- 上传/下载文件
- 创建/删除/重命名目录和文件

### 4. Slurm 集成

R 脚本通过 `sbatch` 提交到 HPC 计算节点执行，登录节点仅负责 Web 服务和作业调度。

---

## 技术栈

- **Rust** — Axum + Tokio
- **模板** — Askama（服务端渲染）
- **交互** — HTMX（局部刷新，无需 JS 框架）
- **样式** — Tailwind CSS
- **数据库** — SQLite（单文件，零配置）
- **计算调度** — Slurm（sbatch/sacct）

---

## 部署

单二进制，scp 到 HPC 登录节点即可运行：

```bash
./ripeline --port 9000 --data-dir ~/ripeline-data
```

通过 SSH 隧道从本地浏览器访问：

```bash
ssh -L 8080:localhost:9000 -J user@bastion user@login-node
# 浏览器打开 http://localhost:8080
```

---

## 项目结构

```text
ripeline/
├── src/
│   ├── main.rs
│   ├── routes/
│   ├── models/
│   ├── slurm/
│   └── rparser/
├── templates/          # Askama HTML 模板
├── static/             # Tailwind CSS 产物
├── migrations/         # SQLite 迁移
├── scripts/            # R 脚本模板和 ripeline.R 辅助库
└── Cargo.toml
```

---

## 执行模型

每个节点执行时：

1. Worker 生成 Slurm 作业脚本
2. 写入 `params.json`（参数）和 `inputs.json`（输入文件路径）
3. `sbatch` 提交到计算节点
4. 计算节点执行 `Rscript script.R`，脚本通过 `source("ripeline.R")` 读取参数和输入
5. 输出写到 `outputs/` 目录
6. Worker 轮询作业状态，完成后收集结果

---

## 单用户模式

仅需管理员密码登录，无多用户权限系统。

---

## License

Internal / Private Project
