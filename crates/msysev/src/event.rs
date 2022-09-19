use crate::keydef::{KeyCode, KeyModifier};

#[derive(Debug, Clone)]
pub enum KeyAction {
    Press,
    Release,
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub scancode: u32,
    pub keycode: KeyCode,
    pub modifiers: KeyModifier,
    pub action: KeyAction,
}

pub type BoxedEventCallback = Box<dyn Fn(Event) -> Result<(), anyhow::Error> + Send + Sync>;

pub fn run_loop<F>(cb: F) -> Result<(), anyhow::Error>
where
    F: Fn(Event) -> Result<(), anyhow::Error> + Send + Sync + 'static,
{
    #[cfg(target_os = "windows")]
    crate::windows::event::run_loop(Box::new(cb))?;

    #[cfg(target_os = "linux")]
    crate::linux::event::run_loop(Box::new(cb))?;

    Ok(())
}

pub fn quit() -> Result<(), anyhow::Error> {
    #[cfg(target_os = "windows")]
    crate::windows::event::quit()?;

    #[cfg(target_os = "linux")]
    crate::linux::event::quit()?;

    Ok(())
}
