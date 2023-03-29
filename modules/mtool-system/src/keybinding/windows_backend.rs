use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
};

use anyhow::Context;
use async_trait::async_trait;
use mapp::{
    provider::{Injector, Res},
    AppContext, AppModule,
};
use mkeybinding::KeySequence;
use msysev::keydef::KeyModifier;
use tokio::sync::{oneshot, OnceCell, RwLock};
use tracing::{debug, warn, error};
use windows::{
    core::{HRESULT, PCSTR},
    s,
    Win32::{
        Foundation::{GetLastError, HWND},
        System::LibraryLoader::GetModuleHandleA,
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::{
                CreateWindowExA, DestroyWindow, GetMessageA, HMENU, HWND_MESSAGE, MSG, WM_HOTKEY,
                WS_DISABLED, WS_EX_NOACTIVATE,
            },
        },
    },
};

use super::{SetKeybinding, SharedAction};

#[derive(Default)]
pub struct Module {}

#[async_trait]
impl AppModule for Module {
    async fn init(&self, app: &mut AppContext) -> Result<(), anyhow::Error> {
        app.injector().construct_once(Keybinging::construct);
        Ok(())
    }
}

pub struct Keybinging {
    inner: Arc<RwLock<KeybindingInner>>,
    hwnd: OnceCell<HWND>,
}

impl Keybinging {
    async fn construct(injector: Injector) -> Result<Res<super::Keybinding>, anyhow::Error> {
        let this = Res::new(Self::new(injector));

        let hwnd = {
            let (tx, rx) = oneshot::channel();
            let keybinding = this.inner.clone();
            let stop = { keybinding.read().await.stop.clone() };
            tokio::task::spawn_blocking(move || {
                if let Err(e) = Self::run_loop(keybinding, stop, tx) {
                    error!("{}", e);
                }
            });
            rx.await.context("receive HWND failed")?
        };

        this.hwnd.set(hwnd)?;

        Ok(Res::new(super::Keybinding::new(this)))
    }

    fn new(injector: Injector) -> Self {
        Self {
            inner: Arc::new(RwLock::new(KeybindingInner::new(injector))),
            hwnd: OnceCell::default(),
        }
    }

    async fn get_hwnd(&self) -> Result<&HWND, anyhow::Error> {
        self.hwnd.get().context("init failed")
    }

    fn run_loop(
        keybinding: Arc<RwLock<KeybindingInner>>,
        stop: Arc<AtomicBool>,
        tx: oneshot::Sender<HWND>,
    ) -> Result<(), anyhow::Error> {
        let hwnd = create_hidden_window()?;

        tx.send(hwnd.clone()).unwrap();

        register_hotkey(&hwnd, 10, MOD_WIN | MOD_ALT, VK_C)?;

        let mut msg = MSG::default();

        while !stop.load(Ordering::Relaxed) && get_message(&mut msg, &hwnd) {
            if msg.message == WM_HOTKEY {
                let id = msg.wParam.0 as i32;

                debug!("id: {}", id);

                let keybinding = keybinding.clone();
                tokio::spawn(async move {
                    let keybinding = keybinding.read().await;
                    keybinding.do_action(id).await;
                });
            }
        }

        debug!("keybinding loop is exited");

        unsafe {
            DestroyWindow(hwnd);
        }

        Ok(())
    }
}

impl Drop for Keybinging {
    fn drop(&mut self) {
        let mut inner = self.inner.blocking_write();
        if let Some(hwnd) = self.hwnd.get() {
            inner.remove_all(hwnd);
            inner.stop.store(true, Ordering::Relaxed);
        }
    }
}

#[async_trait]
impl SetKeybinding for Keybinging {
    async fn define_global(&self, kbd: &str, action: SharedAction) -> Result<(), anyhow::Error> {
        self.inner
            .write()
            .await
            .define_global(self.get_hwnd().await?, kbd, action)
    }

    async fn remove_global(&self, kbd: &str) -> Result<(), anyhow::Error> {
        self.inner
            .write()
            .await
            .remove_global(self.get_hwnd().await?, kbd)
    }
}

struct KeybindingInner {
    injector: Injector,
    actions: HashMap<i32, SharedAction>,
    kbd_id_map: HashMap<String, i32>,
    idgen: AtomicI32,
    stop: Arc<AtomicBool>,
}

impl KeybindingInner {
    fn new(injector: Injector) -> Self {
        Self {
            injector,
            actions: HashMap::new(),
            kbd_id_map: HashMap::new(),
            idgen: AtomicI32::new(0),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl KeybindingInner {
    fn get_action(&self, id: i32) -> Option<SharedAction> {
        self.actions.get(&id).cloned()
    }

    async fn do_action(&self, id: i32) {
        let action = self.get_action(id);

        if let Some(action) = action {
            if let Err(e) = action.do_action(&self.injector).await {
                warn!("do action error: {}", e);
            }
        }
    }

    fn define_global(
        &mut self,
        hwnd: &HWND,
        kbd: &str,
        action: SharedAction,
    ) -> Result<(), anyhow::Error> {
        let ks = KeySequence::parse(kbd)?;
        let (mods, key) = convert_hotkey(&ks)?;

        let id = self.idgen.fetch_add(1, Ordering::Relaxed);

        register_hotkey(hwnd, id, mods, key)?;

        self.actions.insert(id, action);
        self.kbd_id_map.insert(ks.to_string(), id);

        Ok(())
    }

    fn remove_global(&mut self, hwnd: &HWND, kbd: &str) -> Result<(), anyhow::Error> {
        let id = self
            .kbd_id_map
            .get(&KeySequence::parse(kbd)?.to_string())
            .context(format!("{} isn't exist", kbd))?;

        unregister_hotkey(hwnd, *id)?;

        self.actions.remove(&id);
        self.kbd_id_map.remove(kbd);

        Ok(())
    }

    fn remove_all(&mut self, hwnd: &HWND) {
        for (id, _) in &self.actions {
            if let Err(e) = unregister_hotkey(hwnd, *id) {
                warn!("{}", e);
            }
        }
    }
}

fn get_last_error() -> HRESULT {
    unsafe { GetLastError() }.to_hresult()
}

fn create_hidden_window() -> Result<HWND, anyhow::Error> {
    let hwnd = unsafe {
        let hinstance = GetModuleHandleA(PCSTR::null())?;
        CreateWindowExA(
            WS_EX_NOACTIVATE,
            s!("Static"),
            PCSTR::null(),
            WS_DISABLED,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            HMENU::default(),
            hinstance,
            None,
        )
    };

    if hwnd.0 == 0 {
        let e = get_last_error();
        anyhow::bail!(
            "hidden window create failed: {}, {}",
            e.to_string(),
            e.message()
        );
    } else {
        Ok(hwnd)
    }
}

fn get_message(msg: &mut MSG, hwnd: &HWND) -> bool {
    unsafe { GetMessageA(msg, *hwnd, 0, 0) }.as_bool()
}

fn register_hotkey(
    hwnd: &HWND,
    id: i32,
    mods: HOT_KEY_MODIFIERS,
    vk: VIRTUAL_KEY,
) -> Result<(), anyhow::Error> {
    if unsafe { RegisterHotKey(*hwnd, id, mods, vk.0 as u32) }.as_bool() {
        let e = get_last_error();
        anyhow::bail!("register hotkey failed: {}, {}", e.to_string(), e.message());
    }
    Ok(())
}

fn unregister_hotkey(hwnd: &HWND, id: i32) -> Result<(), anyhow::Error> {
    if unsafe { UnregisterHotKey(*hwnd, id) }.as_bool() {
        let e = get_last_error();
        anyhow::bail!(
            "unregister hotkey failed: {}, {}",
            e.to_string(),
            e.message()
        );
    }
    Ok(())
}

fn convert_hotkey(ks: &KeySequence) -> Result<(HOT_KEY_MODIFIERS, VIRTUAL_KEY), anyhow::Error> {
    if ks.len() > 1 {
        anyhow::bail!("only support single key combine at windows, {}", ks);
    }

    let kc = ks[0].clone();

    let mut mods = HOT_KEY_MODIFIERS::default();
    if kc.mods.contains(KeyModifier::ALT) {
        mods |= MOD_ALT;
    }

    if kc.mods.contains(KeyModifier::CONTROL) {
        mods |= MOD_CONTROL;
    }

    if kc.mods.contains(KeyModifier::SHIFT) {
        mods |= MOD_SHIFT;
    }

    if kc.mods.contains(KeyModifier::SUPER) {
        mods |= MOD_WIN;
    }

    Ok((mods, VIRTUAL_KEY::from(kc.key)))
}
