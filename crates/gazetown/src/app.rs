// Main Gazetown application view
use gpui::{Context, Render, Window, div, prelude::*};

/// Main Gazetown application
pub struct GazetownApp;

impl GazetownApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for GazetownApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Gazetown - Agent Orchestration")
    }
}

impl Default for GazetownApp {
    fn default() -> Self {
        Self::new()
    }
}
