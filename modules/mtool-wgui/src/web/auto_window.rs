use mtauri_sys::window::{Position, Size, Window};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlDivElement, ResizeObserver, ResizeObserverEntry};
use yew::{platform::spawn_local, prelude::*};

#[derive(Clone)]
pub enum Msg {
    Resize(Size),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vertical {
    TopAlign,
    Center,
    BottomAlign,
    Absolute(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Horizontal {
    LeftAlign,
    Center,
    RightAlign,
    Absolute(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowProps {
    pub vertical: Vertical,
    pub horizontal: Horizontal,
    pub initial_width: usize,
    pub initial_height: usize,
}

impl Default for WindowProps {
    fn default() -> Self {
        Self {
            vertical: Vertical::Absolute(0),
            horizontal: Horizontal::Absolute(0),
            initial_width: 800,
            initial_height: 600,
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
    fn adjust_window(&self, width: usize, height: usize) -> Result<(), JsValue> {
        let WindowProps {
            vertical,
            horizontal,
            ..
        } = &self.window_props;

        self.set_window_size(width, height);

        let screen = window().unwrap().screen()?;
        let x = match horizontal {
            Horizontal::LeftAlign => 0 as usize,
            Horizontal::Center => ((screen.width()? - width as i32) / 2) as usize,
            Horizontal::RightAlign => (screen.width()? - width as i32) as usize,
            Horizontal::Absolute(x) => *x,
        };

        let y = match vertical {
            Vertical::TopAlign => 0 as usize,
            Vertical::Center => ((screen.height()? - height as i32) / 2) as usize,
            Vertical::BottomAlign => (screen.height()? - height as i32) as usize,
            Vertical::Absolute(y) => *y,
        };

        self.set_window_position(x, y);
        Ok(())
    }

    fn set_window_size(&self, width: usize, height: usize) {
        let window = self.window.clone();
        spawn_local(async move { window.set_size(Size::new_physical(width, height)).await.unwrap() });
    }

    fn set_window_position(&self, x: usize, y: usize) {
        let window = self.window.clone();
        spawn_local(async move { window.set_position(Position::new_physical(x, y)).await.unwrap() });
    }
}

impl Component for AutoWindow {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let window_props = ctx.props().window.clone();

        let this = Self {
            cont: NodeRef::default(),
            window_props,
            window: Window::current().unwrap(),
        };
        this.adjust_window(
            this.window_props.initial_width,
            this.window_props.initial_height,
        )
        .unwrap();
        this
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Resize(size) => {
                let (width, height) = size.get();
                self.adjust_window(width, height).unwrap();
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

                    link.send_message(Msg::Resize(Size::new_physical(width, height)));
                },
            );

            let observer = ResizeObserver::new(f.as_ref().unchecked_ref()).unwrap();

            observer.observe(&self.cont.cast::<HtmlDivElement>().unwrap());

            f.forget();
        }
    }
}
