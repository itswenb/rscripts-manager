# RFlow

RFlow is an internal web-based analysis workbench for R-driven bioinformatics and data-analysis workflows.

The platform is designed for teams that run large-scale analysis pipelines on remote Linux servers but still require iterative, human-controlled parameter tuning and result inspection.

---

# Why RFlow Exists

Traditional server-side analysis workflows are painful.

Researchers often need to:

- SSH into servers
- manually upload/download files
- execute partial R scripts
- inspect generated plots locally
- repeatedly tune parameters
- rerun workflows step-by-step

This becomes increasingly inefficient when:

- datasets are too large for local execution
- workflows contain many iterative stages
- every step requires visual inspection

RFlow turns this process into a browser-based workflow.

---

# Core Features

## Project Management

Organize analysis workflows into isolated projects.

---

## File Asset Management

Manage server-side files directly from the browser.

Supports:

- file upload
- directory management
- move/rename/delete
- previews
- downloads

---

## Structured R Workflow Execution

Execute administrator-registered R scripts through structured forms.

Supports:

- multiple input files
- directory inputs
- parameter schemas
- execution history

---

## Result Visualization

Inspect generated outputs directly in the browser.

Supports:

- PNG
- SVG
- PDF
- CSV
- TSV
- HTML
- logs

---

## Execution Tracking

Every run preserves:

- selected inputs
- parameters
- logs
- outputs
- timestamps

All analysis workflows remain reproducible.

---

# Technology Stack

## Backend

- Rust
- Axum
- SQLx
- Tokio

## Frontend

- React
- Farm
- TailwindCSS
- shadcn/ui

## Runtime

- R
- Rscript
- renv

## Infrastructure

- PostgreSQL
- Linux filesystem storage

---

# Project Structure

```text
rflow/
├── crates/
│   ├── api/
│   ├── worker/
│   ├── core/
│   └── rrunner/
├── apps/
│   └── web/
├── scripts/
│   └── r/
└── data/
````

---

# Execution Model

Every analysis step is executed as:

```bash
Rscript script.R \
  --inputs inputs.json \
  --params params.json \
  --output outputs/
```

This ensures:

* reproducibility
* auditability
* structured execution

---

# Current Scope

RFlow currently focuses on:

* internal research teams
* R-based workflows
* step-by-step execution
* human-in-the-loop analysis

The project intentionally avoids:

* arbitrary code execution
* cloud orchestration complexity
* notebook-style execution
* AI-controlled workflows

---

# License

Internal / Private Project

``` 