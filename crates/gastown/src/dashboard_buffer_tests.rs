use gpui::{AppContext as _, TestAppContext, VisualTestContext};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::dashboard_buffer::{
    AgentInfo, AgentStatus, ConnectionStatus, ConvoyInfo, DashboardData, DashboardDataSource,
    DashboardError, DashboardEvent, DashboardFormatter, DashboardView, RigInfo, TokenUsage,
};

/// Mock data source for testing
pub struct MockDataSource {
    data: Option<DashboardData>,
    available: bool,
}

impl MockDataSource {
    pub fn available_with(data: DashboardData) -> Self {
        Self {
            data: Some(data),
            available: true,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            data: None,
            available: false,
        }
    }
}

/// Reactive mock data source that can be updated during tests
pub struct ReactiveMockDataSource {
    data: Mutex<Option<DashboardData>>,
    available: Mutex<bool>,
}

impl ReactiveMockDataSource {
    pub fn new(data: DashboardData) -> Self {
        Self {
            data: Mutex::new(Some(data)),
            available: Mutex::new(true),
        }
    }

    pub fn update_data(&self, data: DashboardData) {
        *self.data.lock().unwrap() = Some(data);
    }

    pub fn set_unavailable(&self) {
        *self.available.lock().unwrap() = false;
    }

    pub fn set_available(&self) {
        *self.available.lock().unwrap() = true;
    }
}

impl DashboardDataSource for ReactiveMockDataSource {
    fn fetch(&self) -> Result<DashboardData, DashboardError> {
        if !*self.available.lock().unwrap() {
            return Err(DashboardError::NotAvailable);
        }
        self.data
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DashboardError::FetchFailed("No data configured".into()))
    }

    fn is_available(&self) -> bool {
        *self.available.lock().unwrap()
    }
}

impl DashboardDataSource for MockDataSource {
    fn fetch(&self) -> Result<DashboardData, DashboardError> {
        if !self.available {
            return Err(DashboardError::NotAvailable);
        }
        self.data
            .clone()
            .ok_or_else(|| DashboardError::FetchFailed("No data configured".into()))
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

fn sample_dashboard_data() -> DashboardData {
    DashboardData {
        agents: vec![
            AgentInfo {
                name: "agent-1".into(),
                status: AgentStatus::Active,
                token_usage: None,
                context_fill: None,
            },
            AgentInfo {
                name: "agent-2".into(),
                status: AgentStatus::Idle,
                token_usage: None,
                context_fill: None,
            },
        ],
        convoys: vec![ConvoyInfo {
            id: "convoy-1".into(),
            progress: 0.5,
        }],
        rigs: vec![RigInfo {
            name: "main".into(),
            path: "/project".into(),
        }],
    }
}

#[gpui::test]
async fn test_dashboard_displays_content(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(sample_dashboard_data()));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view: &mut DashboardView, _cx| {
        let content = view.content();
        assert!(content.contains("Gastown Dashboard"));
        assert!(content.contains("agent-1"));
        assert!(content.contains("agent-2"));
        assert!(content.contains("convoy-1"));
    });
}

#[gpui::test]
async fn test_dashboard_is_read_only(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view: &mut DashboardView, _cx| {
        assert!(view.is_read_only(), "Dashboard should be read-only");
    });
}

#[gpui::test]
async fn test_dashboard_refresh_updates_timestamp(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view: &mut DashboardView, cx| {
        let first_update = view.last_update();
        assert!(first_update.is_some());

        std::thread::sleep(std::time::Duration::from_millis(10));
        view.refresh(cx);

        let second_update = view.last_update();
        assert!(second_update.is_some());
        assert!(second_update.unwrap() > first_update.unwrap());
    });
}

#[gpui::test]
async fn test_dashboard_shows_error_when_unavailable(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::unavailable());
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view: &mut DashboardView, _cx| {
        assert_eq!(view.connection_status(), &ConnectionStatus::Disconnected);
        assert!(view.content().contains("unavailable"));
        assert!(view.content().contains("gt up"));
    });
}

#[gpui::test]
async fn test_dashboard_shows_connected_status(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view: &mut DashboardView, _cx| {
        assert_eq!(view.connection_status(), &ConnectionStatus::Connected);
    });
}

#[gpui::test]
async fn test_dashboard_formatter_handles_empty_data(_cx: &mut TestAppContext) {
    let data = DashboardData::default();
    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("Gastown Dashboard"));
    assert!(formatted.contains("No agents running"));
    assert!(formatted.contains("No active convoys"));
    assert!(formatted.contains("No rigs configured"));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_agent_status(_cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![
            AgentInfo {
                name: "active-agent".into(),
                status: AgentStatus::Active,
                token_usage: None,
                context_fill: None,
            },
            AgentInfo {
                name: "idle-agent".into(),
                status: AgentStatus::Idle,
                token_usage: None,
                context_fill: None,
            },
            AgentInfo {
                name: "error-agent".into(),
                status: AgentStatus::Error("connection lost".into()),
                token_usage: None,
                context_fill: None,
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("● active-agent"));
    assert!(formatted.contains("○ idle-agent"));
    assert!(formatted.contains("✗ error-agent"));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_convoy_progress(_cx: &mut TestAppContext) {
    let data = DashboardData {
        convoys: vec![
            ConvoyInfo {
                id: "convoy-half".into(),
                progress: 0.5,
            },
            ConvoyInfo {
                id: "convoy-done".into(),
                progress: 1.0,
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("convoy-half"));
    assert!(formatted.contains("50%"));
    assert!(formatted.contains("convoy-done"));
    assert!(formatted.contains("100%"));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_rigs(_cx: &mut TestAppContext) {
    let data = DashboardData {
        rigs: vec![
            RigInfo {
                name: "frontend".into(),
                path: "/app/frontend".into(),
            },
            RigInfo {
                name: "backend".into(),
                path: "/app/backend".into(),
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("frontend → /app/frontend"));
    assert!(formatted.contains("backend → /app/backend"));
}

#[gpui::test]
async fn test_data_source_trait_with_mock(_cx: &mut TestAppContext) {
    let mock = MockDataSource::available_with(sample_dashboard_data());

    assert!(mock.is_available());

    let data = mock.fetch().expect("should fetch successfully");
    assert_eq!(data.agents.len(), 2);
    assert_eq!(data.convoys.len(), 1);
    assert_eq!(data.rigs.len(), 1);
}

#[gpui::test]
async fn test_data_source_unavailable_returns_error(_cx: &mut TestAppContext) {
    let mock = MockDataSource::unavailable();

    assert!(!mock.is_available());

    let result = mock.fetch();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DashboardError::NotAvailable));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_token_usage(_cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![AgentInfo {
            name: "busy-agent".into(),
            status: AgentStatus::Active,
            token_usage: Some(TokenUsage {
                input_tokens: 1500,
                output_tokens: 500,
            }),
            context_fill: Some(0.75),
        }],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("busy-agent"));
    assert!(formatted.contains("[ctx: 75%]"));
    assert!(formatted.contains("[tokens: 1500↓ 500↑]"));
}

// ============================================================================
// GPUI Rendering Tests
// ============================================================================

#[gpui::test]
async fn test_gpui_render_produces_element(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(sample_dashboard_data()));

    let window = cx.update(|cx| {
        cx.open_window(Default::default(), |_window, cx| {
            cx.new(|cx| DashboardView::new(data_source, cx))
        })
        .unwrap()
    });

    let mut cx = VisualTestContext::from_window(window.into(), cx);

    window
        .root(&mut cx)
        .unwrap()
        .update_in(&mut cx, |view, _window, _cx| {
            let content = view.content();
            assert!(
                !content.is_empty(),
                "GPUI render should produce non-empty content"
            );
            assert!(
                content.contains("Gastown Dashboard"),
                "GPUI render should contain dashboard header"
            );
        });
}

#[gpui::test]
async fn test_gpui_view_is_focusable(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(sample_dashboard_data()));

    let window = cx.update(|cx| {
        cx.open_window(
            Default::default(),
            |_window, cx| -> gpui::Entity<DashboardView> {
                cx.new(|cx| DashboardView::new(data_source, cx))
            },
        )
        .unwrap()
    });

    let mut vcx = VisualTestContext::from_window(window.into(), cx);
    let view = window.root(&mut vcx).unwrap();

    view.read_with(&vcx, |view, cx| {
        use gpui::Focusable;
        let _focus_handle = view.focus_handle(cx);
    });
}

// ============================================================================
// Real-Time Updates / GPUI Reactivity Tests
// ============================================================================

#[gpui::test]
async fn test_refresh_emits_data_refreshed_event(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(sample_dashboard_data()));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    let events: Arc<Mutex<Vec<DashboardEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    cx.update(|cx| {
        cx.subscribe(&view, move |_, event: &DashboardEvent, _cx| {
            events_clone.lock().unwrap().push(event.clone());
        })
        .detach();
    });

    view.update(cx, |view, cx| {
        view.refresh(cx);
    });

    let captured_events = events.lock().unwrap();
    assert_eq!(captured_events.len(), 1, "Should emit exactly one event");
    assert!(
        matches!(captured_events[0], DashboardEvent::DataRefreshed),
        "Should emit DataRefreshed event"
    );
}

#[gpui::test]
async fn test_reactive_data_updates_content(cx: &mut TestAppContext) {
    let initial_data = DashboardData {
        agents: vec![AgentInfo {
            name: "initial-agent".into(),
            status: AgentStatus::Idle,
            token_usage: None,
            context_fill: None,
        }],
        ..Default::default()
    };

    let data_source = Arc::new(ReactiveMockDataSource::new(initial_data));
    let view = cx.new(|cx| DashboardView::new(data_source.clone(), cx));

    view.update(cx, |view, _cx| {
        assert!(view.content().contains("initial-agent"));
        assert!(!view.content().contains("updated-agent"));
    });

    let updated_data = DashboardData {
        agents: vec![AgentInfo {
            name: "updated-agent".into(),
            status: AgentStatus::Active,
            token_usage: None,
            context_fill: None,
        }],
        ..Default::default()
    };
    data_source.update_data(updated_data);

    view.update(cx, |view, cx| {
        view.refresh(cx);
    });

    view.update(cx, |view, _cx| {
        assert!(
            view.content().contains("updated-agent"),
            "Content should reflect updated data after refresh"
        );
        assert!(
            !view.content().contains("initial-agent"),
            "Old data should be replaced"
        );
    });
}

#[gpui::test]
async fn test_connection_status_transitions(cx: &mut TestAppContext) {
    let initial_data = sample_dashboard_data();
    let data_source = Arc::new(ReactiveMockDataSource::new(initial_data));
    let view = cx.new(|cx| DashboardView::new(data_source.clone(), cx));

    view.update(cx, |view, _cx| {
        assert_eq!(
            view.connection_status(),
            &ConnectionStatus::Connected,
            "Should start connected"
        );
    });

    data_source.set_unavailable();
    view.update(cx, |view, cx| {
        view.refresh(cx);
    });

    view.update(cx, |view, _cx| {
        assert_eq!(
            view.connection_status(),
            &ConnectionStatus::Disconnected,
            "Should transition to disconnected"
        );
    });

    data_source.set_available();
    view.update(cx, |view, cx| {
        view.refresh(cx);
    });

    view.update(cx, |view, _cx| {
        assert_eq!(
            view.connection_status(),
            &ConnectionStatus::Connected,
            "Should reconnect when available again"
        );
    });
}

// ============================================================================
// Performance Tests
// ============================================================================

#[gpui::test]
async fn test_gpui_format_performance_small_dataset(_cx: &mut TestAppContext) {
    let data = sample_dashboard_data();

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = DashboardFormatter::format(&data);
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 1000,
        "1000 format operations should complete in under 1 second (took {}ms)",
        elapsed.as_millis()
    );
}

#[gpui::test]
async fn test_gpui_format_performance_large_dataset(_cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: (0..100)
            .map(|i| AgentInfo {
                name: format!("agent-{}", i),
                status: if i % 3 == 0 {
                    AgentStatus::Active
                } else if i % 3 == 1 {
                    AgentStatus::Idle
                } else {
                    AgentStatus::Error("test error".into())
                },
                token_usage: Some(TokenUsage {
                    input_tokens: i as u64 * 100,
                    output_tokens: i as u64 * 50,
                }),
                context_fill: Some((i as f32) / 100.0),
            })
            .collect(),
        convoys: (0..50)
            .map(|i| ConvoyInfo {
                id: format!("convoy-{}", i),
                progress: (i as f32) / 50.0,
            })
            .collect(),
        rigs: (0..20)
            .map(|i| RigInfo {
                name: format!("rig-{}", i),
                path: format!("/path/to/project/{}", i),
            })
            .collect(),
    };

    let start = Instant::now();
    for _ in 0..100 {
        let _ = DashboardFormatter::format(&data);
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 500,
        "100 format operations with large dataset should complete in under 500ms (took {}ms)",
        elapsed.as_millis()
    );
}

#[gpui::test]
async fn test_gpui_refresh_performance(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(sample_dashboard_data()));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    let start = Instant::now();
    for _ in 0..100 {
        view.update(cx, |view, cx| {
            view.refresh(cx);
        });
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 500,
        "100 refresh operations should complete in under 500ms (took {}ms)",
        elapsed.as_millis()
    );
}

// ============================================================================
// Layout and Styling Tests
// ============================================================================

#[gpui::test]
async fn test_dashboard_layout_sections_order(_cx: &mut TestAppContext) {
    let data = sample_dashboard_data();
    let formatted = DashboardFormatter::format(&data);

    let header_pos = formatted.find("Gastown Dashboard").unwrap();
    let agents_pos = formatted.find("▸ Agents").unwrap();
    let convoys_pos = formatted.find("▸ Convoys").unwrap();
    let rigs_pos = formatted.find("▸ Rigs").unwrap();

    assert!(
        header_pos < agents_pos,
        "Header should come before Agents section"
    );
    assert!(
        agents_pos < convoys_pos,
        "Agents section should come before Convoys"
    );
    assert!(
        convoys_pos < rigs_pos,
        "Convoys section should come before Rigs"
    );
}

#[gpui::test]
async fn test_dashboard_status_icons_styling(_cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![
            AgentInfo {
                name: "active".into(),
                status: AgentStatus::Active,
                token_usage: None,
                context_fill: None,
            },
            AgentInfo {
                name: "idle".into(),
                status: AgentStatus::Idle,
                token_usage: None,
                context_fill: None,
            },
            AgentInfo {
                name: "errored".into(),
                status: AgentStatus::Error("test".into()),
                token_usage: None,
                context_fill: None,
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(
        formatted.contains("● active"),
        "Active status should use filled circle"
    );
    assert!(
        formatted.contains("○ idle"),
        "Idle status should use empty circle"
    );
    assert!(
        formatted.contains("✗ errored"),
        "Error status should use X mark"
    );
}

#[gpui::test]
async fn test_progress_bar_rendering(_cx: &mut TestAppContext) {
    let data = DashboardData {
        convoys: vec![
            ConvoyInfo {
                id: "empty".into(),
                progress: 0.0,
            },
            ConvoyInfo {
                id: "half".into(),
                progress: 0.5,
            },
            ConvoyInfo {
                id: "full".into(),
                progress: 1.0,
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(
        formatted.contains("[░░░░░░░░░░]"),
        "Empty progress bar should be all empty"
    );
    assert!(
        formatted.contains("[█████░░░░░]"),
        "Half progress bar should be half filled"
    );
    assert!(
        formatted.contains("[██████████]"),
        "Full progress bar should be all filled"
    );
}

#[gpui::test]
async fn test_context_fill_percentage_formatting(_cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![
            AgentInfo {
                name: "low".into(),
                status: AgentStatus::Active,
                token_usage: None,
                context_fill: Some(0.15),
            },
            AgentInfo {
                name: "high".into(),
                status: AgentStatus::Active,
                token_usage: None,
                context_fill: Some(0.99),
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(
        formatted.contains("[ctx: 15%]"),
        "Context fill should round to whole number"
    );
    assert!(
        formatted.contains("[ctx: 99%]"),
        "Context fill should show high percentages"
    );
}

// ============================================================================
// Data Binding Tests
// ============================================================================

#[gpui::test]
async fn test_data_binding_agents(cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![
            AgentInfo {
                name: "agent-alpha".into(),
                status: AgentStatus::Active,
                token_usage: Some(TokenUsage {
                    input_tokens: 1000,
                    output_tokens: 500,
                }),
                context_fill: Some(0.5),
            },
            AgentInfo {
                name: "agent-beta".into(),
                status: AgentStatus::Idle,
                token_usage: None,
                context_fill: None,
            },
        ],
        convoys: vec![],
        rigs: vec![],
    };

    let data_source = Arc::new(MockDataSource::available_with(data));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view, _cx| {
        let content = view.content();

        assert!(content.contains("agent-alpha"));
        assert!(content.contains("agent-beta"));
        assert!(content.contains("[tokens: 1000↓ 500↑]"));
        assert!(content.contains("[ctx: 50%]"));
    });
}

#[gpui::test]
async fn test_data_binding_convoys(cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![],
        convoys: vec![
            ConvoyInfo {
                id: "task-001".into(),
                progress: 0.25,
            },
            ConvoyInfo {
                id: "task-002".into(),
                progress: 0.75,
            },
        ],
        rigs: vec![],
    };

    let data_source = Arc::new(MockDataSource::available_with(data));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view, _cx| {
        let content = view.content();

        assert!(content.contains("task-001"));
        assert!(content.contains("25%"));
        assert!(content.contains("task-002"));
        assert!(content.contains("75%"));
    });
}

#[gpui::test]
async fn test_data_binding_rigs(cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![],
        convoys: vec![],
        rigs: vec![
            RigInfo {
                name: "frontend".into(),
                path: "/home/user/apps/frontend".into(),
            },
            RigInfo {
                name: "api".into(),
                path: "/home/user/apps/api".into(),
            },
        ],
    };

    let data_source = Arc::new(MockDataSource::available_with(data));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view, _cx| {
        let content = view.content();

        assert!(content.contains("frontend → /home/user/apps/frontend"));
        assert!(content.contains("api → /home/user/apps/api"));
    });
}

#[gpui::test]
async fn test_data_binding_preserves_special_characters(cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![AgentInfo {
            name: "agent-with-émojis-🚀".into(),
            status: AgentStatus::Active,
            token_usage: None,
            context_fill: None,
        }],
        rigs: vec![RigInfo {
            name: "path with spaces".into(),
            path: "/home/user/my project/src".into(),
        }],
        ..Default::default()
    };

    let data_source = Arc::new(MockDataSource::available_with(data));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view, _cx| {
        let content = view.content();

        assert!(
            content.contains("agent-with-émojis-🚀"),
            "Should preserve unicode and emojis in agent names"
        );
        assert!(
            content.contains("/home/user/my project/src"),
            "Should preserve spaces in paths"
        );
    });
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

#[gpui::test]
async fn test_empty_sections_display_placeholder(cx: &mut TestAppContext) {
    let data = DashboardData::default();
    let data_source = Arc::new(MockDataSource::available_with(data));
    let view = cx.new(|cx| DashboardView::new(data_source, cx));

    view.update(cx, |view, _cx| {
        let content = view.content();

        assert!(content.contains("No agents running"));
        assert!(content.contains("No active convoys"));
        assert!(content.contains("No rigs configured"));
    });
}

#[gpui::test]
async fn test_error_display_fetch_failed(_cx: &mut TestAppContext) {
    let error = DashboardError::FetchFailed("Connection timeout".into());
    let formatted = DashboardFormatter::format_error(&error);

    assert!(formatted.contains("Failed to load dashboard"));
    assert!(formatted.contains("Connection timeout"));
}

#[gpui::test]
async fn test_error_display_parse_error(_cx: &mut TestAppContext) {
    let error = DashboardError::ParseError("Invalid JSON".into());
    let formatted = DashboardFormatter::format_error(&error);

    assert!(formatted.contains("Failed to parse dashboard data"));
    assert!(formatted.contains("Invalid JSON"));
}

#[gpui::test]
async fn test_multiple_rapid_refreshes(cx: &mut TestAppContext) {
    let data_source = Arc::new(ReactiveMockDataSource::new(sample_dashboard_data()));
    let view = cx.new(|cx| DashboardView::new(data_source.clone(), cx));

    for i in 0..50 {
        let new_data = DashboardData {
            agents: vec![AgentInfo {
                name: format!("agent-{}", i),
                status: AgentStatus::Active,
                token_usage: None,
                context_fill: None,
            }],
            ..Default::default()
        };
        data_source.update_data(new_data);

        view.update(cx, |view, cx| {
            view.refresh(cx);
        });
    }

    view.update(cx, |view, _cx| {
        assert!(
            view.content().contains("agent-49"),
            "After rapid refreshes, content should reflect latest data"
        );
    });
}
