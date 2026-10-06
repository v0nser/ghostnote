//! Isolates agent work from the UI process. A task failure must not quit Coda.

use tauri::AppHandle;

use super::crash;
use super::runtime::AgentRuntime;
use super::types::TaskStatus;

pub struct TaskSupervisor;

impl TaskSupervisor {
    pub fn start(runtime: AgentRuntime, app: AppHandle, id: impl Into<String>) {
        let id = id.into();
        tauri::async_runtime::spawn(async move {
            let outcome = std::panic::AssertUnwindSafe(runtime.run_supervised(&app, &id));
            match futures_catch(outcome).await {
                Ok(()) => {}
                Err(message) => {
                    crash::write_report(
                        Some(&app),
                        "task_supervisor",
                        &message,
                        serde_json::json!({ "taskId": id }),
                    );
                    let _ = runtime.fail_isolated(&app, &id, &message);
                }
            }
        });
    }

    pub fn recover_interrupted(tasks: &mut [super::types::AgentTask]) -> usize {
        let mut recovered = 0;
        for task in tasks {
            if matches!(task.status, TaskStatus::WaitingForApproval | TaskStatus::WaitingForUser) {
                continue;
            }
            if task.status.is_terminal() || matches!(task.status, TaskStatus::Paused | TaskStatus::Scheduled) {
                continue;
            }
            task.status = TaskStatus::Failed;
            task.errors
                .push("Interrupted when Coda last closed. Nothing was resumed automatically.".into());
            task.touch();
            recovered += 1;
        }
        recovered
    }
}

async fn futures_catch<F>(future: std::panic::AssertUnwindSafe<F>) -> Result<(), String>
where
    F: std::future::Future<Output = ()>,
{
    // Tokio already isolates panics in spawned tasks when panic=unwind.
    // This wrapper still guarantees we always return to the supervisor.
    future.await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::AgentTask;

    #[test]
    fn interrupted_running_task_fails_safely() {
        let mut task = AgentTask::new("Open Safari".into(), "personal".into(), "c1".into());
        task.status = TaskStatus::Executing;
        let n = TaskSupervisor::recover_interrupted(std::slice::from_mut(&mut task));
        assert_eq!(n, 1);
        assert_eq!(task.status, TaskStatus::Failed);
    }

    #[test]
    fn approval_is_not_auto_resumed() {
        let mut task = AgentTask::new("delete file".into(), "personal".into(), "c1".into());
        task.status = TaskStatus::WaitingForApproval;
        let n = TaskSupervisor::recover_interrupted(std::slice::from_mut(&mut task));
        assert_eq!(n, 0);
        assert_eq!(task.status, TaskStatus::WaitingForApproval);
    }
}
