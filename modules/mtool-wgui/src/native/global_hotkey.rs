use std::str::FromStr;

use anyhow::Context;
use dashmap::DashMap;
use mapp::prelude::*;
use mkeybinding::KeySequence;
use msysev::*;
use tauri::{plugin, AppHandle};
use tauri_plugin_global_shortcut::{self, GlobalShortcutExt, Shortcut};
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

use mtool_system::keybinding::{GlobalHotKeyEvent, Keybinding, SetupGlobalHotKey};

use crate::{Builder, WGuiStage};

pub struct Module;

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        if cfg!(windows) {
            app.schedule()
                .add_once_task(WGuiStage::Setup, register_wgui_plugin::<tauri::Wry>);
        }
        Ok(())
    }
}

async fn register_wgui_plugin<R>(
    builder: Res<Builder<R>>,
    injector: Injector,
) -> Result<(), anyhow::Error>
where
    R: tauri::Runtime,
{
    let tx = injector.construct_oneshot();
    builder.setup(move |builder| {
        let (ktx, krx) = mpsc::unbounded_channel();

        let hotkey_mgr = Res::new(GlobalHotKeyMgr::new(injector.clone(), ktx));

        let global_shortcut_plugin = {
            let hotkey_mgr = hotkey_mgr.clone();
            tauri_plugin_global_shortcut::Builder::<R>::new()
                .with_handler(move |_, shortcut, _| {
                    if let Some(kv) = hotkey_mgr.shortcut_index.get(shortcut) {
                        if let Err(e) = hotkey_mgr
                            .sender
                            .send(GlobalHotKeyEvent(kv.value().clone()))
                        {
                            warn!("send global hotkey event failed: {}", e);
                        }
                    }
                })
                .build()
        };
        Ok(builder.plugin(global_shortcut_plugin).plugin(
            plugin::Builder::<R>::new("mtool-global-shortcut")
                .setup(move |_app, _| {
                    let keybinding = Res::new(Keybinding::new(hotkey_mgr, krx));
                    if let Err(_) = tx.send(keybinding.clone()) {
                        warn!("Failed to send wgui Keybinding");
                    }

                    tokio::spawn(keybinding.clone().handle_event_loop(injector));
                    Ok(())
                })
                .build(),
        ))
    })
}

pub struct GlobalHotKeyMgr {
    injector: Injector,
    sender: mpsc::UnboundedSender<GlobalHotKeyEvent>,
    shortcut_index: DashMap<Shortcut, KeySequence>,
}

impl GlobalHotKeyMgr {
    fn new(injector: Injector, sender: mpsc::UnboundedSender<GlobalHotKeyEvent>) -> Self {
        Self {
            injector,
            sender,
            shortcut_index: DashMap::new(),
        }
    }

    async fn app_handle(&self) -> Res<AppHandle> {
        self.injector.get::<Res<AppHandle>>().await.unwrap()
    }

    async fn run_on_main_thread<F, O>(&self, f: F) -> Result<O, anyhow::Error>
    where
        F: FnOnce() -> O + Send + 'static,
        O: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let app = self.app_handle().await;
        app.run_on_main_thread(move || {
            let _ = tx.send(f());
        })
        .context("run on main thread")?;
        rx.await
            .map_err(|_| anyhow::anyhow!("wait for result on main thread failed"))
    }

    async fn define(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        let accelerator = convert_kbd_to_accelerator(ks)?;
        let shortcut = Shortcut::from_str(&accelerator)?;
        self.shortcut_index.insert(shortcut.clone(), ks.clone());
        let app = self.app_handle().await;
        self.run_on_main_thread(move || {
            app.global_shortcut()
                .register(shortcut)
                .context("tauri register global key")
        })
        .await?
    }

    async fn remove(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        let app = self.app_handle().await;
        let ks = ks.clone();
        self.run_on_main_thread(move || {
            app.global_shortcut()
                .unregister(convert_kbd_to_accelerator(&ks)?.as_str())
                .context("tauri unregister global key")
        })
        .await?
    }
}

#[async_trait]
impl SetupGlobalHotKey for GlobalHotKeyMgr {
    async fn register(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.define(ks).await
    }

    async fn unregister(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.remove(ks).await
    }
}

fn convert_kbd_to_accelerator(ks: &KeySequence) -> Result<String, anyhow::Error> {
    if ks.len() > 1 {
        anyhow::bail!("only support single key combine at tauri, {}", ks);
    }

    let kc = ks[0].clone();

    let mut accelerator = String::new();

    if kc.mods.contains(ModifierState::ALT) {
        accelerator.push_str("alt+");
    }

    if kc.mods.contains(ModifierState::CONTROL) {
        accelerator.push_str("ctrl+");
    }

    if kc.mods.contains(ModifierState::SHIFT) {
        accelerator.push_str("shift+");
    }

    if kc.mods.contains(ModifierState::SUPER) {
        accelerator.push_str("super+");
    }

    accelerator.push_str(match kc.key {
        KeyCode::Backquote => "`",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::Digit0 => "0",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::Backspace => "backspace",
        KeyCode::Tab => "tab",
        KeyCode::KeyQ => "q",
        KeyCode::KeyW => "w",
        KeyCode::KeyE => "e",
        KeyCode::KeyR => "r",
        KeyCode::KeyT => "t",
        KeyCode::KeyY => "y",
        KeyCode::KeyU => "u",
        KeyCode::KeyI => "i",
        KeyCode::KeyO => "o",
        KeyCode::KeyP => "p",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "backslash",
        KeyCode::KeyA => "a",
        KeyCode::KeyS => "s",
        KeyCode::KeyD => "d",
        KeyCode::KeyF => "f",
        KeyCode::KeyG => "g",
        KeyCode::KeyH => "h",
        KeyCode::KeyJ => "j",
        KeyCode::KeyK => "k",
        KeyCode::KeyL => "l",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::Enter => "enter",
        KeyCode::KeyZ => "z",
        KeyCode::KeyX => "x",
        KeyCode::KeyC => "c",
        KeyCode::KeyV => "v",
        KeyCode::KeyB => "b",
        KeyCode::KeyN => "n",
        KeyCode::KeyM => "m",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Space => "space",
        KeyCode::ArrowLeft => "arrowleft",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::ArrowUp => "up",
        KeyCode::ArrowDown => "down",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::ArrowRight => "arrowright",
        KeyCode::Numpad7 => "num7",
        KeyCode::Numpad4 => "num4",
        KeyCode::Numpad1 => "num1",
        KeyCode::NumpadDivide => "numdivide",
        KeyCode::Numpad8 => "num8",
        KeyCode::Numpad5 => "num5",
        KeyCode::Numpad2 => "num2",
        KeyCode::Numpad0 => "num0",
        KeyCode::Numpad9 => "num9",
        KeyCode::Numpad6 => "num6",
        KeyCode::Numpad3 => "num3",
        KeyCode::NumpadSubtract => "numsubstract",
        KeyCode::NumpadAdd => "numadd",
        KeyCode::NumpadComma => "numcomma",
        KeyCode::NumpadEnter => "numenter",
        KeyCode::Escape => "esc",
        KeyCode::F1 => "f1",
        KeyCode::F2 => "f2",
        KeyCode::F3 => "f3",
        KeyCode::F4 => "f4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "f6",
        KeyCode::F7 => "f7",
        KeyCode::F8 => "f8",
        KeyCode::F9 => "f9",
        KeyCode::F10 => "f10",
        KeyCode::F11 => "f11",
        KeyCode::F12 => "f12",
        _ => unimplemented!("unknown key code: {:?}", kc.key),
    });

    Ok(accelerator)
}
