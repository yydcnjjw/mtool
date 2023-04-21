use mtauri_sys::window::{PhysicalSize, PhysicalPosition};
use tracing::debug;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlDivElement, ResizeObserver, ResizeObserverEntry};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone)]
pub enum Msg {
    Resize(PhysicalSize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowProps {
    pub vertical_center: bool,
    pub horizontal_center: bool,
    pub center: bool,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Default for WindowProps {
    fn default() -> Self {
        Self {
            vertical_center: false,
            horizontal_center: false,
            center: false,
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub window: WindowProps,

    pub children: Children,
}

pub struct AutoResizeWindow {
    cont: NodeRef,
    window_props: WindowProps,
}
impl AutoResizeWindow {
    fn adjust_window(&self) -> Result<(), JsValue> {
        debug!("{:?}", self.window_props);

        let WindowProps {
            vertical_center,
            horizontal_center,
            center,
            mut x,
            mut y,
            width,
            height,
        } = self.window_props;

        Self::set_window_size(width, height);

        if vertical_center || horizontal_center || center {
            let screen = window().unwrap().screen()?;

            if vertical_center || center {
                let screen_height = screen.height()?;
                x = ((screen_height - height as i32) / 2) as usize;
            }

            if horizontal_center || center {
                let screen_width = screen.width()?;
                y = ((screen_width - width as i32) / 2) as usize;
            }
        }

        Self::set_window_position(y, x);
        Ok(())
    }

    fn set_window_size(width: usize, height: usize) {
        spawn_local(mtauri_sys::window::set_size(PhysicalSize { width, height }));
    }

    fn set_window_position(x: usize, y: usize) {
        spawn_local(mtauri_sys::window::set_position(PhysicalPosition { x, y }));
        // window().unwrap().move_to(x as i32, y as i32).unwrap();
    }
}

impl Component for AutoResizeWindow {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let window_props = ctx.props().window.clone();

        let this = Self {
            cont: NodeRef::default(),
            window_props,
        };
        this.adjust_window().unwrap();
        this
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Resize(size) => {
                self.window_props.width = size.width;
                self.window_props.height = size.height;
                self.adjust_window().unwrap();
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
        if first_render {
            let link = ctx.link().clone();
            let f = Closure::<dyn Fn(Vec<ResizeObserverEntry>)>::new(
                move |entries: Vec<ResizeObserverEntry>| {
                    let elem = entries[0].target();

                    let (width, height) =
                        (elem.client_width() as usize, elem.client_height() as usize);

                    link.send_message(Msg::Resize(PhysicalSize { width, height }));
                },
            );

            let observer = ResizeObserver::new(f.as_ref().unchecked_ref()).unwrap();

            observer.observe(&self.cont.cast::<HtmlDivElement>().unwrap());

            f.forget();
        }
    }
}
