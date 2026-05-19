# Technical Stack

## Backend

- Rust
- Axum
- Tokio
- SQLx
- PostgreSQL

## Frontend

- React
- TypeScript
- Farm
- TailwindCSS
- shadcn/ui
- TanStack Query
- TanStack Router
- TanStack Table

## Runtime

- R
- Rscript
- renv

## Environment

- mise
- pnpm

## Infrastructure

- PostgreSQL
- Redis (optional future queue backend)
- Linux filesystem storage

---

# High-Level Architecture

```text
Browser
  ↓
React Frontend
  ↓
Axum API
  ↓
PostgreSQL
  ↓
Background Worker
  ↓
Rscript Execution
  ↓
Generated Outputs
````

---

# Monorepo Structure

```text
rflow/
├── .mise.toml
├── Cargo.toml
├── crates/
│   ├── api/
│   ├── worker/
│   ├── core/
│   └── rrunner/
├── apps/
│   └── web/
├── migrations/
├── scripts/
│   └── r/
├── data/
└── docker-compose.yml
```

---

# Core Data Model

## Project

Logical workspace boundary.

Represents:

* experiment
* analysis workspace
* research dataset grouping

Each project owns:

* uploaded files
* generated outputs
* analysis runs

---

## FileAsset

Tracks every file and directory.

Supports:

* upload
* move
* rename
* delete
* download
* preview

Files are never treated as anonymous paths.

All filesystem state must exist in database state.

---

## WorkflowStep

Defines a registered analysis step.

Includes:

* script path
* input schema
* parameter schema
* output expectations

Workflow steps are administrator-defined.

Users cannot execute arbitrary scripts.

---

## ScriptRun

Represents one execution instance.

Contains:

* selected inputs
* parameter values
* execution state
* stdout/stderr
* timing metadata

Every run must be reproducible.

---

## OutputFile

Represents generated outputs.

Includes:

* plots
* CSV
* TSV
* PDF
* HTML
* logs

Outputs belong to ScriptRuns.

---

# File System Layout

```text
/data/rflow/
├── projects/
│   └── project_001/
│       ├── uploads/
│       ├── workspace/
│       ├── runs/
│       └── trash/
├── scripts/
└── shared/
```

---

# Execution Model

Every execution creates:

```text
run_dir/
├── inputs.json
├── params.json
├── outputs/
├── stdout.log
└── stderr.log
```

R scripts are executed using:

```bash
Rscript script.R \
  --inputs inputs.json \
  --params params.json \
  --output outputs/
```

---

# Security Rules

## Forbidden

* arbitrary shell execution
* arbitrary script uploads
* unrestricted filesystem access
* direct path injection
* direct shell interpolation

## Required

* administrator-approved scripts
* typed input schemas
* structured parameter validation
* filesystem sandboxing
* permission-aware file access
* activity logging

---

# Queueing Strategy

Initial implementation:

* PostgreSQL-backed jobs
* Tokio workers

Future optional upgrade:

* Redis queue backend

Do NOT introduce distributed orchestration early.

---

# Frontend Architecture

## State Management

Prefer:

* TanStack Query
* local component state

Avoid:

* oversized global stores
* Redux-style complexity

---

## UI Philosophy

The UI is an internal operations interface.

Priorities:

* clarity
* traceability
* debuggability
* fast navigation

NOT marketing aesthetics.

---

# API Philosophy

Prefer:

* explicit typed contracts
* schema-driven validation
* predictable endpoints

Avoid:

* magic serialization
* hidden implicit behaviors
* auto-generated runtime mutations

---

# File Management Constraints

Files may be:

* very large
* deeply nested
* shared between steps

Design accordingly.

Avoid assumptions of:

* browser-memory-safe processing
* instant uploads
* synchronous transfers

---

# AI Integration Philosophy

AI is NOT the workflow controller.

AI may later assist with:

* result interpretation
* parameter suggestions
* report generation

AI must NOT:

* mutate execution state automatically
* execute arbitrary commands
* bypass permission systems

---

# Non-Goals

This project is NOT:

* a notebook platform
* a generic workflow engine
* a cloud IDE
* a Kubernetes orchestration layer
* a distributed compute scheduler
* a Jupyter replacement
