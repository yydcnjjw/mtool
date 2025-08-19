use mapp::{
    anyhow::{self, Context},
    itertools::Itertools,
    tokio::{
        self,
        sync::{mpsc, watch},
    },
    tracing::{info, warn},
};
use notify::Watcher;
use std::{path::PathBuf, time::Duration};
use walkdir::WalkDir;
use windows::{
    core::BOOL,
    Win32::{
        Foundation::{FALSE, HWND, LPARAM, TRUE},
        System::StationsAndDesktops::EnumDesktopWindows,
        UI::WindowsAndMessaging::{
            EnumWindows, IsWindow, IsWindowVisible, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
        },
    },
};

use crate::SearchPath;

use super::{window_style, AppInfo, WindowInfo};

pub fn get_default_search_paths() -> Vec<SearchPath> {
    let mut search_paths = vec![];
    let appdata_path = format!(
        "{}\\Microsoft\\Windows\\Start Menu\\Programs",
        std::env::var("APPDATA").unwrap()
    );
    let default_paths = vec![
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs",
        &appdata_path,
    ];
    for path in default_paths {
        search_paths.push(SearchPath::new(PathBuf::from(path), u8::MAX));
    }
    search_paths
}

fn new_dir_watcher() -> Result<
    (
        notify::RecommendedWatcher,
        mpsc::UnboundedReceiver<notify::Event>,
    ),
    anyhow::Error,
> {
    let (tx, rx) = mpsc::unbounded_channel();

    let wather = notify::recommended_watcher(move |res| match res {
        Ok(ev) => {
            if let Err(e) = tx.send(ev) {
                warn!("{:?}", e)
            }
        }
        Err(e) => warn!("{:?}", e),
    })?;

    Ok((wather, rx))
}

pub type AppsWatcher = watch::Receiver<Vec<crate::AppInfo>>;
pub type WindowsWatcher = watch::Receiver<Vec<crate::WindowInfo>>;

pub struct Searcher {
    #[allow(unused)]
    dir_watcher: notify::RecommendedWatcher,
    apps_watcher: AppsWatcher,
    windows_watcher: WindowsWatcher,
}

impl Searcher {
    pub fn new(extra_search_paths: Vec<SearchPath>) -> Result<Self, anyhow::Error> {
        let (mut dir_watcher, mut dirs_rx) = new_dir_watcher()?;

        let search_paths = get_default_search_paths()
            .into_iter()
            .chain(extra_search_paths)
            .unique()
            .inspect(|path| {
                info!("watch dir {}", path.path.display());
                if let Err(e) = dir_watcher.watch(&path.path, notify::RecursiveMode::Recursive) {
                    warn!("{:?}", e);
                }
            })
            .collect_vec();

        let (apps_tx, apps_watcher) = watch::channel(Self::search_apps(&search_paths));

        tokio::spawn(async move {
            while let Some(_) = dirs_rx.recv().await {
                if let Err(e) = apps_tx.send(Self::search_apps(&search_paths)) {
                    warn!("{:?}", e);
                }
            }
        });

        let windows_watcher = Self::run_search_windows_loop();

        Ok(Self {
            dir_watcher,
            apps_watcher,
            windows_watcher,
        })
    }

    pub fn watch_apps(&self) -> AppsWatcher {
        self.apps_watcher.clone()
    }

    pub fn watch_windows(&self) -> WindowsWatcher {
        self.windows_watcher.clone()
    }

    fn search_apps(search_paths: &Vec<SearchPath>) -> Vec<crate::AppInfo> {
        let mut apps = vec![];
        for search_path in search_paths {
            if !search_path.path.exists() {
                continue;
            }

            for entry in WalkDir::new(&search_path.path)
                .max_depth(search_path.depth as usize)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                if let Some(extension) = path.extension() {
                    if extension == "lnk" {
                        match AppInfo::new(&path) {
                            Ok(app) => {
                                apps.push(app);
                            }
                            Err(e) => warn!("{:?}", e),
                        }
                    }
                }
            }
        }
        apps
    }

    pub fn search_windows() -> Vec<crate::WindowInfo> {
        let mut vec: Vec<crate::WindowInfo> = Vec::new();
        if let Err(e) = enum_windows(
            |hwnd| {
                let (_style, ex_style) = match window_style(hwnd) {
                    Ok(result) => result,
                    Err(e) => {
                        warn!("{:#}", e);
                        return Ok(());
                    }
                };

                if !ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_APPWINDOW) {
                    match WindowInfo::new(hwnd) {
                        Ok(win) => vec.push(win),
                        Err(e) => warn!("{:#}", e),
                    }
                }

                Ok(())
            },
            false,
        ) {
            warn!("{:#}", e);
        }
        vec
    }

    pub fn run_search_windows_loop() -> WindowsWatcher {
        let (tx, rx) = watch::channel(Self::search_windows());

        tokio::spawn(async move {
            loop {
                if let Err(e) = tx.send(Self::search_windows()) {
                    warn!("{:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        rx
    }
}

fn enum_windows<Callback>(callback: Callback, current_desktop: bool) -> Result<(), anyhow::Error>
where
    Callback: FnMut(HWND) -> Result<(), anyhow::Error>,
{
    unsafe {
        let lparam = LPARAM(&callback as *const _ as isize);
        if current_desktop {
            EnumDesktopWindows(None, Some(enum_windows_proc::<Callback>), lparam)
                .context("EnumDesktopWindows")?
        } else {
            EnumWindows(Some(enum_windows_proc::<Callback>), lparam).context("EnumWindows")?
        }
    }
    Ok(())
}

unsafe extern "system" fn enum_windows_proc<Callback>(hwnd: HWND, lparam: LPARAM) -> BOOL
where
    Callback: FnMut(HWND) -> Result<(), anyhow::Error>,
{
    let callback = lparam.0 as *mut Callback;
    unsafe {
        if IsWindow(Some(hwnd)).as_bool() && IsWindowVisible(hwnd).as_bool() {
            if let Err(e) = (*callback)(hwnd) {
                warn!("{:?}", e);
                return FALSE;
            }
        }
        TRUE
    }
}
