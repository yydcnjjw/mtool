use image::DynamicImage;
use mapp::{
    anyhow::{self, anyhow, Context},
    dashmap::DashMap,
    once_cell::sync::OnceCell,
};
use std::{
    ffi::OsString,
    ops::Deref,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
};
use windows::Win32::{
    Foundation::{CloseHandle, HWND, RECT},
    System::{
        ProcessStatus::K32GetModuleFileNameExW,
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
    UI::WindowsAndMessaging::{
        FlashWindow, GetWindowInfo, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
        SetForegroundWindow, ShowWindow, SW_RESTORE, WINDOWINFO, WINDOW_EX_STYLE,
        WINDOW_STYLE, WS_MINIMIZE,
    },
};
use windows_icons::get_icon_by_path;

use crate::{WindowInfoExt, WindowPosition};

use super::VersionInfo;

#[derive(Clone)]
pub struct WindowInfo {
    hwnd: usize,
    exe_path: PathBuf,
}

static EXECUTABLE_CACHE: OnceCell<ExecutableCache> = OnceCell::new();

#[derive(Clone)]
struct ExecutableMeta {
    #[allow(unused)]
    path: PathBuf,
    version_info: Option<VersionInfo>,
    icon: Option<DynamicImage>,
}

impl ExecutableMeta {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            version_info: None,
            icon: None,
        }
    }
}

struct ExecutableCache {
    inner: DashMap<PathBuf, ExecutableMeta>,
}

impl Deref for ExecutableCache {
    type Target = DashMap<PathBuf, ExecutableMeta>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ExecutableCache {
    fn version_info<Init>(path: &PathBuf, init: Init) -> Result<VersionInfo, anyhow::Error>
    where
        Init: FnOnce(&PathBuf) -> Result<VersionInfo, anyhow::Error>,
    {
        let info = &mut Self::get()
            .entry(path.clone())
            .insert(ExecutableMeta::new(path.clone()))
            .version_info;

        Ok(match info {
            Some(info) => info.clone(),
            None => {
                let value = init(path)?;
                *info = Some(value.clone());
                value
            }
        })
    }

    fn icon<Init>(path: &PathBuf, init: Init) -> Result<DynamicImage, anyhow::Error>
    where
        Init: FnOnce(&PathBuf) -> Result<DynamicImage, anyhow::Error>,
    {
        let icon = &mut Self::get()
            .entry(path.clone())
            .insert(ExecutableMeta::new(path.clone()))
            .icon;

        Ok(match icon {
            Some(icon) => icon.clone(),
            None => {
                let value = init(path)?;
                *icon = Some(value.clone());
                value
            }
        })
    }

    fn get() -> &'static Self {
        EXECUTABLE_CACHE.get_or_init(|| ExecutableCache {
            inner: DashMap::new(),
        })
    }
}

impl WindowInfo {
    pub fn new(hwnd: HWND) -> Result<crate::WindowInfo, anyhow::Error> {
        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd.clone(), Some(&mut pid)) };

        let exe_path = get_process_path(pid)?;

        let version_info = ExecutableCache::version_info(&PathBuf::from(&exe_path), |path| {
            VersionInfo::from_file(path).context(format!("exe_path: {exe_path}"))
        })?;

        Ok(crate::WindowInfo {
            title: window_title(&hwnd),
            position: window_position(&hwnd)?,
            process: crate::ProcessInfo {
                name: Path::new(&exe_path)
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
                    .unwrap(),
                description: version_info.file_description,
                pid: pid as usize,
            },
            inner: Self {
                hwnd: hwnd.0 as usize,
                exe_path: exe_path.into(),
            },
        })
    }
}

pub fn window_style(hwnd: HWND) -> Result<(WINDOW_STYLE, WINDOW_EX_STYLE), anyhow::Error> {
    let mut wi = WINDOWINFO::default();
    unsafe {
        GetWindowInfo(hwnd, &mut wi)?;
    }
    Ok((wi.dwStyle, wi.dwExStyle))
}

impl WindowInfoExt for crate::WindowInfo {
    fn load_icon(&self) -> Result<DynamicImage, anyhow::Error> {
        ExecutableCache::icon(&self.inner.exe_path, |path| {
            get_icon_by_path(path.to_str().context("path to string")?)
                .map(|icon| DynamicImage::from(icon))
                .map_err(|e| anyhow!("{}", e))
        })
    }

    fn active(&self) -> Result<(), anyhow::Error> {
        let hwnd = HWND(self.inner.hwnd as _);

        let (style, _ex_style) = window_style(hwnd)?;

        unsafe {
            _ = if !style.contains(WS_MINIMIZE) {
                SetForegroundWindow(hwnd)
            } else {
                ShowWindow(hwnd, SW_RESTORE)
            };

            _ = FlashWindow(hwnd, true);
        };
        Ok(())
    }
}

fn window_position(hwnd: &HWND) -> Result<WindowPosition, anyhow::Error> {
    unsafe {
        let mut lprect = RECT::default();
        GetWindowRect(*hwnd, &mut lprect)?;
        Ok(WindowPosition {
            height: lprect.bottom - lprect.top,
            width: lprect.right - lprect.left,
            x: lprect.left,
            y: lprect.top,
        })
    }
}

fn window_title(hwnd: &HWND) -> String {
    let mut v: Vec<u16> = vec![0; 255];
    let title_len = unsafe { GetWindowTextW(*hwnd, &mut v) };
    String::from_utf16_lossy(&v[0..(title_len as usize)])
}

pub fn get_process_path(process_id: u32) -> Result<String, anyhow::Error> {
    unsafe {
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        )
        .context(format!("OpenProcess: {}", process_id))?;

        let mut buffer = vec![0u16; 1024];
        let size = K32GetModuleFileNameExW(Some(process_handle), None, &mut buffer);
        CloseHandle(process_handle).context("CloseHandle")?;

        if size == 0 {
            return Err(anyhow::anyhow!(
                "K32GetModuleFileNameExW: {}",
                windows::core::Error::from_thread()
            ));
        }

        buffer.truncate(size as usize);

        OsString::from_wide(&buffer)
            .into_string()
            .map_err(|e| anyhow::anyhow!("{}", e.display()))
    }
}
