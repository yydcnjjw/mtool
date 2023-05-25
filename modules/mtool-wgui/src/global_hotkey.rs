use std::str::FromStr;

use anyhow::Context;
use async_trait::async_trait;
use dashmap::DashMap;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};
use mkeybinding::KeySequence;
use msysev::keydef::{KeyCode, KeyModifier};
use tauri::plugin;
use tauri_plugin_global_shortcut::{self, GlobalShortcutExt, Shortcut};
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

use mtool_system::keybinding::{GlobalHotKeyEvent, Keybinding, SetGlobalHotKey};

use crate::{Builder, GuiStage};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();

        app.injector()
            .construct_once(|| async move { Ok(rx.await?) });

        app.schedule()
            .add_once_task(GuiStage::Setup, |builder, injector| {
                add_wgui_plugin(tx, builder, injector)
            });
        Ok(())
    }
}

async fn add_wgui_plugin(
    tx: oneshot::Sender<Res<Keybinding>>,
    builder: Res<Builder>,
    injector: Injector,
) -> Result<(), anyhow::Error> {
    builder.setup(move |builder| {
        Ok(builder.plugin(
            plugin::Builder::<tauri::Wry>::new("global-shortcut")
                .setup(move |app, _| {
                    let (ktx, krx) = mpsc::unbounded_channel();

                    let hotkey_mgr = Res::new(GlobalHotKeyMgr::new(Res::new(app.clone()), ktx));

                    let app = app.clone();
                    tokio::spawn(async move {
                        {
                            let hotkey_mgr = hotkey_mgr.clone();

                            app.plugin(
                                tauri_plugin_global_shortcut::Builder::with_handler(
                                    move |shortcut| {
                                        if let Some(kv) = hotkey_mgr.shortcut_index.get(shortcut) {
                                            if let Err(e) = hotkey_mgr
                                                .sender
                                                .send(GlobalHotKeyEvent(kv.value().clone()))
                                            {
                                                warn!("send global hotkey event failed: {}", e);
                                            }
                                        }
                                    },
                                )
                                .build(),
                            )
                            .unwrap();
                        }

                        let keybinding = Res::new(Keybinding::new(hotkey_mgr, krx));
                        if let Err(_) = tx.send(keybinding.clone()) {
                            warn!("Failed to send wgui Keybinding");
                        }

                        tokio::spawn(keybinding.clone().handle_event_loop(injector));
                    });

                    Ok(())
                })
                .build(),
        ))
    })
}

pub struct GlobalHotKeyMgr {
    app: Res<tauri::AppHandle>,
    sender: mpsc::UnboundedSender<GlobalHotKeyEvent>,
    shortcut_index: DashMap<Shortcut, KeySequence>,
}

impl GlobalHotKeyMgr {
    fn new(app: Res<tauri::AppHandle>, sender: mpsc::UnboundedSender<GlobalHotKeyEvent>) -> Self {
        Self {
            app,
            sender,
            shortcut_index: DashMap::new(),
        }
    }

    fn define(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        let accelerator = convert_kbd_to_accelerator(ks)?;
        let shortcut = Shortcut::from_str(&accelerator)?;
        self.shortcut_index.insert(shortcut.clone(), ks.clone());

        self.app
            .global_shortcut()
            .register(shortcut)
            .context(format!("tauri register global key: {}", accelerator))
    }

    fn remove(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.app
            .global_shortcut()
            .unregister(convert_kbd_to_accelerator(ks)?.as_str())
            .context("tauri unregister global key")
    }
}

#[async_trait]
impl SetGlobalHotKey for GlobalHotKeyMgr {
    async fn register(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.define(ks)
    }

    async fn unregister(&self, ks: &KeySequence) -> Result<(), anyhow::Error> {
        self.remove(ks)
    }
}

fn convert_kbd_to_accelerator(ks: &KeySequence) -> Result<String, anyhow::Error> {
    if ks.len() > 1 {
        anyhow::bail!("only support single key combine at tauri, {}", ks);
    }

    let kc = ks[0].clone();

    let mut accelerator = String::new();

    if kc.mods.contains(KeyModifier::ALT) {
        accelerator.push_str("alt+");
    }

    if kc.mods.contains(KeyModifier::CONTROL) {
        accelerator.push_str("ctrl+");
    }

    if kc.mods.contains(KeyModifier::SHIFT) {
        accelerator.push_str("shift+");
    }

    if kc.mods.contains(KeyModifier::SUPER) {
        accelerator.push_str("super+");
    }

    accelerator.push_str(match kc.key {
        KeyCode::GraveAccent => "`",
        KeyCode::Num1 => "1",
        KeyCode::Num2 => "2",
        KeyCode::Num3 => "3",
        KeyCode::Num4 => "4",
        KeyCode::Num5 => "5",
        KeyCode::Num6 => "6",
        KeyCode::Num7 => "7",
        KeyCode::Num8 => "8",
        KeyCode::Num9 => "9",
        KeyCode::Num0 => "0",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::BackSpace => "backspace",
        KeyCode::Tab => "tab",
        KeyCode::Q => "q",
        KeyCode::W => "w",
        KeyCode::E => "e",
        KeyCode::R => "r",
        KeyCode::T => "t",
        KeyCode::Y => "y",
        KeyCode::U => "u",
        KeyCode::I => "i",
        KeyCode::O => "o",
        KeyCode::P => "p",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "backslash",
        KeyCode::A => "a",
        KeyCode::S => "s",
        KeyCode::D => "d",
        KeyCode::F => "f",
        KeyCode::G => "g",
        KeyCode::H => "h",
        KeyCode::J => "j",
        KeyCode::K => "k",
        KeyCode::L => "l",
        KeyCode::Semicolon => ";",
        KeyCode::Apostrophe => "'",
        KeyCode::Return => "enter",
        KeyCode::Z => "z",
        KeyCode::X => "x",
        KeyCode::C => "c",
        KeyCode::V => "v",
        KeyCode::B => "b",
        KeyCode::N => "n",
        KeyCode::M => "m",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Spacebar => "space",
        KeyCode::LeftArrow => "arrowleft",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::UpArrow => "up",
        KeyCode::DownArrow => "down",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::RightArrow => "arrowright",
        KeyCode::Keypad7 => "num7",
        KeyCode::Keypad4 => "num4",
        KeyCode::Keypad1 => "num1",
        KeyCode::Divide => "numdivide",
        KeyCode::Keypad8 => "num8",
        KeyCode::Keypad5 => "num5",
        KeyCode::Keypad2 => "num2",
        KeyCode::Keypad0 => "num0",
        KeyCode::Keypad9 => "num9",
        KeyCode::Keypad6 => "num6",
        KeyCode::Keypad3 => "num3",
        KeyCode::Subtract => "numsubstract",
        KeyCode::Add => "numadd",
        KeyCode::KeypadComma => "numcomma",
        KeyCode::KeypadEnter => "numenter",
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