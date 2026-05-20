# 项目节点图编辑器设计

## 概述

为项目详情页实现 ComfyUI 风格的节点图编辑器，用户可以可视化地组装 R 脚本分析脚本节点。

## 技术选型

- 前端：LiteGraph.js（ComfyUI 底层库）
- 交互：Canvas 节点图 + HTMX 弹窗
- 后端：Axum API
- 执行：Slurm sbatch 提交

## 节点类型

### DataSource 节点
- 无 input 端口
- 一个 output 端口 "file"
- 双击弹出文件选择器，从 ~/.ripeline/data/ 选择文件
- 节点属性存储选中的文件路径

### Script 节点（动态注册）
- 从 /pipelines/scripts API 获取所有可用脚本
- 每个脚本注册为一种节点类型
- input 端口：由 @input 注解定义
- output 端口：由 @output 注解定义
- 双击展开参数面板（@param 定义的参数）

## 数据模型

ProjectFlow 表增加 graph_data TEXT 字段，存储 LiteGraph graph.serialize() 的 JSON。

## API

| 方法 | 路径 | 功能 |
|------|------|------|
| GET | /projects/{id} | 项目详情页（LiteGraph 编辑器） |
| GET | /projects/{id}/flow | 获取 graph JSON |
| POST | /projects/{id}/flow | 保存 graph JSON |
| POST | /projects/{id}/run | 整体执行 |
| POST | /projects/{id}/run-node/{index} | 单节点执行 |
| GET | /projects/{id}/run-status | 轮询执行状态 |
| GET | /projects/{id}/output/{node_index} | 查看节点输出文件 |

## 执行逻辑

1. 保存 graph → 解析连线 → 拓扑排序
2. 每个节点创建工作目录：~/.ripeline/projects/{project_name}/{index}_{node_name}/
3. 上游 output 文件软链接到当前节点工作目录作为 input
4. sbatch 提交：cd {work_dir} && Rscript {script_path}
5. 轮询 sacct 状态，完成后更新节点颜色

## 前端交互

- 工具栏：添加节点（数据源/脚本）、保存、运行全部、停止
- 右键菜单：执行此节点 / 查看输出 / 删除
- 节点状态颜色：灰=未执行，蓝=运行中，绿=完成，红=失败
- 自动保存：连线/移动变更后 debounce 2s 保存
- 点击输出端口图标 → 弹出文件浏览器查看结果
