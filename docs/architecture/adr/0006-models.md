# ADR-0006: Local-first model routing

- Status: Accepted
- Date: 2026-10-28

## Context
Local operation must coexist with optional hosted models without silent data export.

## Decision
Use an Ollama-compatible local adapter by default. OpenAI is optional, disabled until owner configuration, and governed by data classification, budget, retention, and redaction. Models receive no authority token or credential.

## Consequences
Missing models produce unavailable states. Routing and external disclosures are inspectable. Core persistence and policy need no cloud.
