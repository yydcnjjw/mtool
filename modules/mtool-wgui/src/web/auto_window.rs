use mtauri_sys::window::{PhysicalPosition, PhysicalSize, Window};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlDivElement, ResizeObserver, ResizeObserverEntry};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone)]
pub enum Msg {
    Resize(PhysicalSize<u32>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vertical {
    TopAlign,
    Center,
    BottomAlign,
    Absolute(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Horizontal {
    LeftAlign,
    Center,
    RightAlign,
    Absolute(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowProps {
    pub vertical: Vertical,
    pub horizontal: Horizontal,
    pub initial_size: PhysicalSize<u32>,
    pub resizable: bool,
}

impl Default for WindowProps {
    fn default() -> Self {
        Self {
            vertical: Vertical::Absolute(0),
            horizontal: Horizontal::Absolute(0),
            initial_size: PhysicalSize::new(800, 600),
            resizable: false,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub window: WindowProps,

    pub children: Children,
}

pub struct AutoWindow {
    cont: NodeRef,
    window_props: WindowProps,
    window: Window,
}
impl AutoWindow {
    fn adjust_window(&self, size: PhysicalSize<u32>) -> Result<(), JsValue> {
        let WindowProps {
            vertical,
            horizontal,
            ..
        } = &self.window_props;

        let PhysicalSize { width, height } = size;

        self.set_window_size(size);

        let screen = window().unwrap().screen()?;
        let x = match horizontal {
            Horizontal::LeftAlign => 0,
            Horizontal::Center => (screen.width()? - width as i32) / 2,
            Horizontal::RightAlign => screen.width()? - width as i32,
            Horizontal::Absolute(x) => *x,
        };

        let y = match vertical {
            Vertical::TopAlign => 0,
            Vertical::Center => (screen.height()? - height as i32) / 2,
            Vertical::BottomAlign => screen.height()? - height as i32,
            Vertical::Absolute(y) => *y,
        };

        self.set_window_position(PhysicalPosition::new(x, y));
        Ok(())
    }

    fn set_window_size(&self, size: PhysicalSize<u32>) {
        let window = self.window.clone();
        spawn_local(async move { window.set_size(size.into()).await.unwrap() });
    }

    fn set_window_position(&self, pos: PhysicalPosition<i32>) {
        let window = self.window.clone();
        spawn_local(async move { window.set_position(pos.into()).await.unwrap() });
    }
}

impl Component for AutoWindow {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let window_props = ctx.props().window.clone();

        let this = Self {
            cont: NodeRef::default(),
            window_props: window_props.clone(),
            window: Window::current().unwrap(),
        };
        this.adjust_window(this.window_props.initial_size).unwrap();

        let window = this.window.clone();
        let WindowProps { resizable, .. } = window_props;
        spawn_local(async move {
            window.set_resizable(resizable).await.unwrap();
        });

        this
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Resize(size) => {
                self.adjust_window(size).unwrap();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("inline-flex")} ref={self.cont.clone()}>
                { for ctx.props().children.iter() }
            </div>
        }
    }
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render && !ctx.props().window.resizable {
            let link = ctx.link().clone();
            let f = Closure::<dyn Fn(Vec<ResizeObserverEntry>)>::new(
                move |entries: Vec<ResizeObserverEntry>| {
                    let elem = entries[0].target();

                    let (width, height) = (elem.client_width() as u32, elem.client_height() as u32);

                    link.send_message(Msg::Resize(PhysicalSize::new(width, height).into()));
                },
            );

            let observer = ResizeObserver::new(f.as_ref().unchecked_ref()).unwrap();

            observer.observe(&self.cont.cast::<HtmlDivElement>().unwrap());

            f.forget();
        }
    }
}
