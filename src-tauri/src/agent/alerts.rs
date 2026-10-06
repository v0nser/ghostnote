//! Periodic overdue / pending-approval alerts. Respects stealth suppression.

use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::runtime::AgentRuntime;
use super::types::ApprovalStatus;
use crate::stealth::notifications::{self, Notification};

pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15 * 60));
        loop {
            interval.tick().await;
            ping(&app);
        }
    });
}

fn ping(app: &AppHandle) {
    if notifications::is_suppressed() {
        return;
    }
    let overdue = crate::meeting::overdue_count(app);
    if overdue > 0 {
        notifications::dispatch(
            app,
            Notification {
                title: "Coda".into(),
                body: format!("{overdue} meeting action item(s) look overdue."),
            },
        );
    }
    let Some(runtime) = app.try_state::<AgentRuntime>() else {
        return;
    };
    runtime.ensure_loaded(app);
    let pending = runtime
        .list_approvals(app)
        .iter()
        .filter(|item| item.status == ApprovalStatus::Pending)
        .count();
    if pending > 0 {
        notifications::dispatch(
            app,
            Notification {
                title: "Coda".into(),
                body: format!("{pending} action(s) waiting for your approval."),
            },
        );
    }
}
