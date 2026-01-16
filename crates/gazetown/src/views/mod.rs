// Views for Gazetown
// - Dashboard: Agent status, token usage, context fill
// - Agent Panel: Prompting agents
// - Diff Viewer: Git diffs with syntax highlighting
// - Convoy Status: Multi-agent workflow progress

pub mod agent_panel;
pub mod convoy_status;
pub mod dashboard;
pub mod diff_viewer;

#[allow(unused_imports)]
pub use agent_panel::AgentPanelView;
#[allow(unused_imports)]
pub use convoy_status::ConvoyStatus;
#[allow(unused_imports)]
pub use dashboard::Dashboard;
#[allow(unused_imports)]
pub use diff_viewer::DiffViewer;
