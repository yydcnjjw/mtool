use mtool_wgui::WGuiWindow;
use std::{mem::size_of, sync::Arc};
use tauri::PhysicalSize;
use tokio::sync::oneshot;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2CompositionController, COREWEBVIEW2_MOUSE_EVENT_KIND,
    COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS,
};
use winapi::um::{
    wingdi::MAKEPOINTS,
    winuser::{GET_KEYSTATE_WPARAM, GET_WHEEL_DELTA_WPARAM, GET_XBUTTON_WPARAM},
};
use windows::{
    core::ComInterface,
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{PtInRect, ScreenToClient},
        UI::{
            Controls::*,
            Input::KeyboardAndMouse::{
                GetCapture, ReleaseCapture, SetCapture, TrackMouseEvent, TME_CANCEL, TME_LEAVE,
                TRACKMOUSEEVENT,
            },
            Shell::{DefSubclassProc, SetWindowSubclass},
            WindowsAndMessaging::*,
        },
    },
    UI::Composition::{Compositor, ContainerVisual},
};

#[derive(Clone)]
pub struct WebviewVisual {
    webview_visual: ContainerVisual,
}

struct WebviewVisualData {
    controller: ICoreWebView2CompositionController,
    is_capturing_mouse: bool,
    is_tracking_mouse: bool,
}

impl WebviewVisual {
    pub async fn new(win: Arc<WGuiWindow>, compositor: &Compositor) -> Result<Self, anyhow::Error> {
        let hwnd = win.hwnd()?;

        let webview_visual = {
            let compositor = compositor.clone();

            let (tx, rx) = oneshot::channel();

            let hwnd = hwnd.clone();
            win.with_webview(move |webview| {
                let _ = tx.send(|| -> Result<_, anyhow::Error> {
                    let controller = webview
                        .controller()
                        .cast::<ICoreWebView2CompositionController>()?;

                    let webview_visual = compositor.CreateContainerVisual()?;

                    unsafe {
                        controller.SetRootVisualTarget(&webview_visual)?;
                    }

                    unsafe {
                        SetWindowSubclass(
                            hwnd,
                            Some(Self::subclass_proc),
                            8081,
                            Box::into_raw(Box::new(WebviewVisualData {
                                controller: controller.clone(),
                                is_capturing_mouse: false,
                                is_tracking_mouse: false,
                            })) as _,
                        );
                    }

                    Ok(webview_visual)
                }());
            })?;
            rx.await??
        };

        Ok(Self { webview_visual })
    }

    pub fn handle(&self) -> &ContainerVisual {
        &self.webview_visual
    }

    pub fn set_size(&self, size: PhysicalSize<u32>) -> Result<(), anyhow::Error> {
        self.webview_visual.SetSize(Vector2 {
            X: size.width as f32,
            Y: size.height as f32,
        })?;
        Ok(())
    }

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        uidsubclass: usize,
        dwrefdata: usize,
    ) -> LRESULT {
        let data = dwrefdata as *mut WebviewVisualData;

        match msg {
            WM_MOUSEHWHEEL | WM_MOUSELEAVE | WM_LBUTTONDBLCLK | WM_LBUTTONDOWN | WM_LBUTTONUP
            | WM_MBUTTONDBLCLK | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEMOVE
            | WM_RBUTTONDBLCLK | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MOUSEWHEEL
            | WM_XBUTTONDBLCLK | WM_XBUTTONDOWN | WM_XBUTTONUP => {
                let points = MAKEPOINTS(lparam.0 as u32);
                let mut point = POINT {
                    x: points.x as i32,
                    y: points.y as i32,
                };
                if let WM_MOUSEWHEEL | WM_MOUSEHWHEEL = msg {
                    ScreenToClient(hwnd, &mut point);
                };

                let mut bounds = RECT::default();
                GetClientRect(hwnd, &mut bounds);
                let is_mouse_in_webview = PtInRect(&bounds, point).as_bool();
                if is_mouse_in_webview || msg == WM_MOUSELEAVE || (*data).is_capturing_mouse {
                    let mut mouse_data = 0u32;
                    match msg {
                        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                            mouse_data = GET_WHEEL_DELTA_WPARAM(wparam.0) as u32;
                        }

                        WM_XBUTTONDBLCLK | WM_XBUTTONDOWN | WM_XBUTTONUP => {
                            mouse_data = GET_XBUTTON_WPARAM(wparam.0) as u32;
                        }

                        WM_MOUSEMOVE => {
                            if !(*data).is_tracking_mouse {
                                let mut e = TRACKMOUSEEVENT::default();
                                e.cbSize = size_of::<TRACKMOUSEEVENT>() as u32;
                                e.dwFlags = TME_LEAVE;
                                e.hwndTrack = hwnd;
                                TrackMouseEvent(&mut e);
                                (*data).is_tracking_mouse = true;
                            }
                        }

                        WM_MOUSELEAVE => {
                            (*data).is_tracking_mouse = false;
                        }
                        _ => {}
                    }

                    match msg {
                        WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_XBUTTONDOWN => {
                            if is_mouse_in_webview && GetCapture() != hwnd {
                                (*data).is_capturing_mouse = true;
                                SetCapture(hwnd);
                            }
                        }
                        WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP | WM_XBUTTONUP => {
                            if GetCapture() == hwnd {
                                (*data).is_capturing_mouse = false;
                                ReleaseCapture();
                            }
                        }
                        _ => {}
                    }

                    // if msg != WM_MOUSELEAVE {
                    //     point.x -= webview_bounds.left;
                    //     point.y -= webview_bounds.top;
                    // }

                    (*data)
                        .controller
                        .SendMouseInput(
                            COREWEBVIEW2_MOUSE_EVENT_KIND(msg as i32),
                            COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS(
                                GET_KEYSTATE_WPARAM(wparam.0) as i32
                            ),
                            mouse_data,
                            point,
                        )
                        .unwrap();
                } else if msg == WM_MOUSEMOVE && (*data).is_tracking_mouse {
                    (*data).is_tracking_mouse = false;

                    let mut e = TRACKMOUSEEVENT::default();
                    e.cbSize = size_of::<TRACKMOUSEEVENT>() as u32;
                    e.dwFlags = TME_LEAVE | TME_CANCEL;
                    e.hwndTrack = hwnd;
                    TrackMouseEvent(&mut e);
                    Self::subclass_proc(hwnd, msg, wparam, lparam, uidsubclass, dwrefdata);
                }
            }
            _ => {}
        };

        DefSubclassProc(hwnd, msg, wparam, lparam)
    }
}
