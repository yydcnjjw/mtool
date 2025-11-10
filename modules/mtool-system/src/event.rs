use mapp::{
    dpi::PhysicalPosition,
    keyboard_types::{Code, KeyState, Modifiers},
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum SystemEvent {
    NotificationPosted(Notification),
    Keyboard(Keyboard),
    Mouse(MouseEvent),
}

pub type ButtonId = u32;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum ElementState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum MouseScrollDelta {
    LineDelta(f32, f32),
    PixelDelta(PhysicalPosition<f64>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum MouseEvent {
    Motion {
        position: PhysicalPosition<i64>,
    },
    Wheel {
        position: PhysicalPosition<i64>,
        delta: MouseScrollDelta,
    },
    Button {
        position: PhysicalPosition<i64>,
        state: ElementState,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct Keyboard {
    pub state: KeyState,
    pub code: Code,
    pub modifiers: Modifiers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
#[serde(tag = "type")]
pub enum Notification {
    Generic {
        app: AppInfo,
        content: NotificationContent,
    },
    Im {
        app: AppInfo,
    },
    Media(MediaNotification),
    Agenda {
        app: AppInfo,
        content: NotificationContent,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct AppInfo {
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct NotificationContent {
    pub message: String,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaNotification {
    pub app: AppInfo,
    pub state: PlaybackState,
    pub metadata: MediaMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub enum MediaState {
    None,
    Stopped,
    Paused,
    Playing,
    FastForwarding,
    Rewinding,
    Buffering,
    Error,
    Connecting,
    SkippingToPrevious,
    SkippingToNext,
    SkippingToQueueItem,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct PlaybackState {
    pub state: MediaState,
    pub position: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "mapp::serde")]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
}
