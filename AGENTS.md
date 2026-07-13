# AGENTS.md

## Reference documents

Communication systems for meters:

| Document                                                | Title                     | Applies to                                                          |
| ------------------------------------------------------- | ------------------------- | ------------------------------------------------------------------- |
| [EN 13757-1:2021](spec/EN_13757/EN_13757-1.pdf)         | Data exchange             | Overall architecture, protocol-stack overview, data models and OBIS |
| [EN 13757-2:2018+A1:2023](spec/EN_13757/EN_13757-2.pdf) | Wired M-Bus communication | Physical and data-link layers for wired twisted-pair M-Bus          |
| [EN 13757-3:2025](spec/EN_13757/EN_13757-3.pdf)         | Application protocols     | M-Bus application layer, including data encoding and semantics      |

Consult these reference documents whenever they are relevant to the task.

## GitHub Actions

When authoring GitHub Actions workflows, follow these rules:

- Always use names, not only the `uses` field, to make it easier to understand the workflow.