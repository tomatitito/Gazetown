// Agent Panel view - Prompting agents (writable text input)
use gpui::{Context, Render, Window, div, prelude::*};

pub struct AgentPanelView;

impl Render for AgentPanelView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Agent Panel - Prompting (stub)")
    }
}
