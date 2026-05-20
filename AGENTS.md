# AGENTS.md

## Project Overview

Ripeline 是部署在 HPC 登录节点上的 R 脚本脚本节点管理平台。

单体 Rust 应用，服务端渲染，通过 Slurm 调度 R 脚本到计算节点执行。

面向生物信息学研究人员，解决传统 SSH + 手动执行 + 本地查看结果的低效工作流。

---

## Core Workflow Philosophy

用户工作流：

1. 在"脚本节点"页面管理 R 脚本节点（每个节点 = 一个 R 脚本 + 注解定义的参数/输入/输出）
2. 在"项目"中将节点串联成分析流程
3. 配置参数，选择输入文件
4. 执行脚本节点，通过 Slurm 提交到计算节点
5. 实时查看运行状态、日志、输出结果
6. 可暂停/回退/重跑任意节点

核心理念：**人控制流程，机器执行计算。**

---

## Architecture

- 单体应用，非前后端分离
- Axum + Askama + HTMX + Tailwind CSS + SQLite
- 单二进制部署，零外部依赖
- 通过 SSH 隧道访问

---

## Design Principles

### 1. 单二进制，零配置部署

不依赖 Node.js、PostgreSQL、Redis、Docker。scp 一个文件即可运行。

### 2. R 脚本是核心

平台围绕 R 脚本构建。脚本通过注解声明接口，平台自动解析。

### 3. Slurm 原生集成

计算任务通过 sbatch 提交，不在登录节点执行任何重计算。

### 4. 人在回路

每个节点可独立控制。失败自动暂停。支持回退重跑。不做黑盒自动化。

### 5. 文件即数据

直接操作文件系统，不做额外抽象层。输入输出都是文件路径。

---

## AI Guidance

生成代码时：

- 使用 Askama 模板 + HTMX 属性实现交互，不引入 JS 框架
- Tailwind CSS 写样式，保持 UI 简洁专业
- SQLite 存储，使用 sqlx 的 compile-time checked queries
- 所有路由在 Axum 中定义，返回 HTML 或 HTMX 片段
- 文件操作直接使用 tokio::fs，限制在配置的根目录内
- Slurm 交互通过 tokio::process::Command 调用 sbatch/sacct/scancel

---

## Forbidden

- 前后端分离架构
- JavaScript 框架（React, Vue, Svelte）
- 外部数据库服务（PostgreSQL, MySQL）
- 容器编排（Docker, K8s）
- 多用户权限系统
- 在登录节点直接执行 R 脚本
- 动态 shell 命令拼接（防注入）
- 自动化决策（AI 自动调参、自动重试）

---

## Required Reading

修改架构前必读：DESIGN.md
