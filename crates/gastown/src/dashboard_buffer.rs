#![allow(dead_code)]

use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use std::sync::Arc;

/// Events emitted by the dashboard when state changes
#[derive(Clone, Debug)]
pub enum DashboardEvent {
    /// Dashboard data was refreshed
    DataRefreshed,
    /// Connection status changed
    ConnectionChanged(ConnectionStatus),
    /// An agent was added
    AgentAdded(String),
    /// An agent was removed
    AgentRemoved(String),
    /// An agent's status changed
    AgentStatusChanged { name: String, status: AgentStatus },
}

/// Dashboard data returned by any data source
#[derive(Clone, Debug, Default)]
pub struct DashboardData {
    pub agents: Vec<AgentInfo>,
    pub convoys: Vec<ConvoyInfo>,
    pub rigs: Vec<RigInfo>,
}

#[derive(Clone, Debug)]
pub struct AgentInfo {
    pub name: String,
    pub status: AgentStatus,
    pub token_usage: Option<TokenUsage>,
    pub context_fill: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentStatus {
    Active,
    Idle,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct ConvoyInfo {
    pub id: String,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub struct RigInfo {
    pub name: String,
    pub path: String,
}

/// Trait for fetching dashboard data - implementations can be direct, mock, or HTTP
pub trait DashboardDataSource: Send + Sync {
    fn fetch(&self) -> Result<DashboardData, DashboardError>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum DashboardError {
    NotAvailable,
    FetchFailed(String),
    ParseError(String),
}

impl std::fmt::Display for DashboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashboardError::NotAvailable => write!(f, "Dashboard not available"),
            DashboardError::FetchFailed(msg) => write!(f, "Fetch failed: {}", msg),
            DashboardError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

/// Formats dashboard data for display
pub struct DashboardFormatter;

impl DashboardFormatter {
    pub fn format(data: &DashboardData) -> String {
        let mut output = String::new();

        output.push_str("═══ Gastown Dashboard ═══\n\n");

        // Agents section
        output.push_str("▸ Agents\n");
        if data.agents.is_empty() {
            output.push_str("  No agents running\n");
        } else {
            for agent in &data.agents {
                let status_icon = match &agent.status {
                    AgentStatus::Active => "●",
                    AgentStatus::Idle => "○",
                    AgentStatus::Error(_) => "✗",
                };
                let mut line = format!("  {} {}", status_icon, agent.name);

                if let Some(fill) = agent.context_fill {
                    line.push_str(&format!(" [ctx: {:.0}%]", fill * 100.0));
                }
                if let Some(tokens) = &agent.token_usage {
                    line.push_str(&format!(
                        " [tokens: {}↓ {}↑]",
                        tokens.input_tokens, tokens.output_tokens
                    ));
                }
                output.push_str(&line);
                output.push('\n');
            }
        }
        output.push('\n');

        // Convoys section
        output.push_str("▸ Convoys\n");
        if data.convoys.is_empty() {
            output.push_str("  No active convoys\n");
        } else {
            for convoy in &data.convoys {
                let progress_bar = Self::progress_bar(convoy.progress);
                output.push_str(&format!(
                    "  {} {} ({:.0}%)\n",
                    convoy.id,
                    progress_bar,
                    convoy.progress * 100.0
                ));
            }
        }
        output.push('\n');

        // Rigs section
        output.push_str("▸ Rigs\n");
        if data.rigs.is_empty() {
            output.push_str("  No rigs configured\n");
        } else {
            for rig in &data.rigs {
                output.push_str(&format!("  {} → {}\n", rig.name, rig.path));
            }
        }

        output
    }

    pub fn format_error(error: &DashboardError) -> String {
        match error {
            DashboardError::NotAvailable => {
                "Dashboard unavailable\n\nRun 'gt up' to start Gastown".into()
            }
            DashboardError::FetchFailed(msg) => {
                format!("Failed to load dashboard\n\n{}", msg)
            }
            DashboardError::ParseError(msg) => {
                format!("Failed to parse dashboard data\n\n{}", msg)
            }
        }
    }

    fn progress_bar(progress: f32) -> String {
        let filled = (progress * 10.0) as usize;
        let empty = 10 - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

/// View for displaying dashboard data.
/// This is a pure GPUI view - not a workspace Item.
pub struct DashboardView {
    focus_handle: FocusHandle,
    content: SharedString,
    data_source: Arc<dyn DashboardDataSource>,
    last_update: Option<std::time::Instant>,
    connection_status: ConnectionStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

impl DashboardView {
    pub fn new(data_source: Arc<dyn DashboardDataSource>, cx: &mut App) -> Self {
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            content: SharedString::from("Loading..."),
            data_source,
            last_update: None,
            connection_status: ConnectionStatus::Unknown,
        };
        view.refresh_sync();
        view
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_read_only(&self) -> bool {
        true
    }

    pub fn connection_status(&self) -> &ConnectionStatus {
        &self.connection_status
    }

    pub fn last_update(&self) -> Option<std::time::Instant> {
        self.last_update
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.refresh_sync();
        cx.emit(DashboardEvent::DataRefreshed);
        cx.notify();
    }

    fn refresh_sync(&mut self) {
        self.connection_status = if self.data_source.is_available() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };

        match self.data_source.fetch() {
            Ok(data) => {
                self.content = DashboardFormatter::format(&data).into();
                self.last_update = Some(std::time::Instant::now());
            }
            Err(err) => {
                self.content = DashboardFormatter::format_error(&err).into();
            }
        }
    }
}

impl Focusable for DashboardView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DashboardEvent> for DashboardView {}

impl Render for DashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.content.clone())
    }
}
