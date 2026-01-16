// Convoy Status - Multi-agent workflow progress
use gpui::{Context, Render, Window, div, prelude::*};

pub struct ConvoyStatus;

impl Render for ConvoyStatus {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Convoy Status - Multi-Agent Workflows (stub)")
    }
}
