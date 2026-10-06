# Memory

Ghost Note already kept a short live transcript window. The agent memory
layer extends that without dumping the whole store into the prompt.

## Kinds

1. Working — current task scratch
2. Episodic — what happened in a conversation or meeting
3. Semantic — durable facts
4. User preferences
5. Task history summaries

## Object

`id`, `type`, `content`, `source`, `confidence`, `created_at`, `updated_at`,
`importance`, `scope`, `expires_at`, `embedding` (optional).

Scopes: `SESSION`, `CONVERSATION`, `TASK`, `USER`, `AGENT`.

## Operations

`memory_search`, `memory_create`, `memory_update`, `memory_delete`.

Explicit user phrases:

- “Remember this.” → `MEMORY_WRITE` (user source, high importance)
- “Forget this.” → `MEMORY_DELETE` after confirmation

Retrieval is keyword + recency + importance. No full dump.
