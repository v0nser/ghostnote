# Agent API

Logical routes implemented as Tauri commands. Names match the product API
so a future local HTTP gateway can delegate to the same orchestrator.

| Logical route | Command |
| --- | --- |
| POST /agents | `agent_upsert_profile` |
| GET /agents | `agent_list_profiles` |
| POST /tasks | `agent_create_task` |
| GET /tasks | `agent_list_tasks` |
| GET /tasks/{id} | `agent_get_task` |
| POST /tasks/{id}/pause | `agent_pause_task` |
| POST /tasks/{id}/resume | `agent_resume_task` |
| POST /tasks/{id}/cancel | `agent_cancel_task` |
| POST /tasks/{id}/retry | `agent_retry_task` |
| GET /tasks/{id}/events | streamed as `ghostnote://agent-event` |
| GET /approvals | `agent_list_approvals` |
| POST /approvals/{id}/approve | `agent_approve` |
| POST /approvals/{id}/deny | `agent_deny` |
| GET /memories | `agent_list_memories` |
| POST /memories | `agent_remember` |
| DELETE /memories/{id} | `agent_forget` |
| GET /tools | `agent_list_tools` |
| GET /permissions | `agent_get_permissions` |
| PUT /permissions | `agent_set_permission` |
| POST /schedules | `agent_schedule` |
| GET /schedules | `agent_list_schedules` |
| DELETE /schedules/{id} | `agent_delete_schedule` |

Voice: `agent_start_voice`, `agent_stop_voice`.
Setup: `agent_setup_status`, `agent_download_whisper_model`.
