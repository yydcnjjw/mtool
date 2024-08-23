use tauri::EventLoopMessage;
use tauri_runtime_wry::{
    tao::{
        event::Event,
        event_loop::{ControlFlow, EventLoopProxy, EventLoopWindowTarget},
    },
    EventLoopIterationContext, Message, Plugin, PluginBuilder, WebContextStore,
};
use tracing::debug;

pub struct WryPlugin;

impl Plugin<EventLoopMessage> for WryPlugin {
    fn on_event(
        &mut self,
        event: &Event<Message<EventLoopMessage>>,
        _event_loop: &EventLoopWindowTarget<Message<EventLoopMessage>>,
        _proxy: &EventLoopProxy<Message<EventLoopMessage>>,
        _control_flow: &mut ControlFlow,
        _context: EventLoopIterationContext<'_, EventLoopMessage>,
        _web_context: &WebContextStore,
    ) -> bool {
        match event {
            Event::WindowEvent {
                window_id, event, ..
            } => {
                debug!("{:?}, {:?}", window_id, event);
            }
            _ => {}
        }

        false
    }
}

pub struct WryPluginBuilder {}

impl WryPluginBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl PluginBuilder<EventLoopMessage> for WryPluginBuilder {
    type Plugin = WryPlugin;

    fn build(self, _context: tauri_runtime_wry::Context<EventLoopMessage>) -> Self::Plugin {
        WryPlugin
    }
}
