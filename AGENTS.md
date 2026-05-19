# AGENTS.md

## Project Overview

RFlow is an internal web-based analysis workbench for bioinformatics and data-analysis teams.

The system is designed to replace fragmented workflows based on:

- SSH
- FinalShell
- manual file upload/download
- shell scripts
- local result inspection
- repeated parameter tuning

with a structured browser-based workflow.

RFlow is NOT an online IDE.

RFlow is NOT a notebook system.

RFlow is NOT intended to replace RStudio.

The purpose of the system is:

> Turn server-side R analysis workflows into a manageable, traceable, interactive web platform.

---

# Core Workflow Philosophy

Traditional workflow:

1. Upload data to server
2. SSH into server
3. Run partial R scripts
4. Download generated plots
5. Inspect locally
6. Modify parameters
7. Re-run next step
8. Repeat many times

RFlow transforms this into:

1. Upload/manage files in browser
2. Select analysis step
3. Configure parameters
4. Execute R scripts
5. View logs and generated plots directly in browser
6. Adjust parameters
7. Continue workflow

The system is specifically designed for:

- iterative parameter tuning
- step-by-step exploratory workflows
- large datasets that cannot run locally
- human-in-the-loop analysis

This is NOT a fixed pipeline runner.

---

# Design Principles

## 1. Human-Controlled Workflow

The platform must preserve researcher control.

Do NOT automate away the workflow into a black-box pipeline.

Each step should remain inspectable and configurable.

---

## 2. R Scripts Are First-Class Citizens

The platform wraps existing R workflows.

The system does NOT attempt to replace:

- R
- RStudio
- Bioconductor ecosystems

R scripts remain the execution source of truth.

---

## 3. File-Centric Architecture

Files are critical assets.

Every uploaded/generated file must be:

- tracked
- version-aware
- auditable
- permission-controlled

No anonymous filesystem mutations.

---

## 4. Secure Execution

Users must NEVER execute arbitrary shell commands.

R scripts must be pre-registered by administrators.

Execution must occur through structured JSON inputs.

---

## 5. Reproducibility

Every run must preserve:

- input files
- parameters
- logs
- outputs
- execution timestamps

Runs must be reproducible.

---

## 6. Large File Awareness

The platform is designed for large bioinformatics datasets.

Avoid designs assuming:

- small uploads
- browser-only processing
- local-first architecture

The server is the compute source of truth.

---

# AI Guidance

When generating code for this project:

- prioritize maintainability over abstraction
- avoid overengineering
- avoid premature microservice decomposition
- avoid Kubernetes-specific assumptions
- avoid hidden magic
- prefer explicit schemas and typed contracts
- prefer filesystem transparency
- preserve debuggability

---

# Forbidden Architectural Decisions

Do NOT introduce:

- dynamic shell execution
- user-defined arbitrary scripts
- hidden runtime mutation
- browser-side execution of analysis
- opaque workflow engines
- automatic AI-driven parameter mutation
- tightly coupled monolith state managers

---

# Required Reading

Before modifying architecture or major workflows, read:

- DESIGN.md

DESIGN.md is the authoritative technical specification.
