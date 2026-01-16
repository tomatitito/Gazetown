// Dashboard view - Agent status, token usage, context fill
use gpui::{Context, Render, Window, div, prelude::*};

pub struct Dashboard;

impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Dashboard - Agent Status (stub)")
    }
}
