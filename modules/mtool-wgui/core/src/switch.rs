use mtauri_sys::window::Window;
use tracing::{debug, warn};
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::{prelude::*, switch::SwitchProps};

use std::marker::PhantomData;

pub struct ListenSwitch<R>
where
    R: Routable,
{
    route_unlisten: Option<RouteListener>,
    _phantom: PhantomData<R>,
}

pub enum Msg {
    RegisterRouteListener(RouteListener),
}

pub struct RouteListener {
    pub unlisten: Option<Box<dyn Fn() -> Result<(), JsValue>>>,
}

impl Drop for RouteListener {
    fn drop(&mut self) {
        if let Some(unlisten) = &self.unlisten {
            (*unlisten)().unwrap();
        }
    }
}

impl<R> ListenSwitch<R>
where
    R: Routable + 'static,
{
    fn listen_route(ctx: &Context<Self>) {
        let link = ctx.link().clone();
        ctx.link().send_future(async move {
            let unlisten = match Window::current()
                .unwrap()
                .listen("route", move |e: mtauri_sys::event::Event<String>| {
                    debug!("try route to {}", &e.payload);
                    if let Some(nav) = link.navigator() {
                        if let Some(r) = R::recognize(&e.payload) {
                            nav.push(&r);
                        } else {
                            warn!("route failed: {}", e.payload);
                        }
                    }
                    Ok(())
                })
                .await
            {
                Ok(v) => Some(Box::new(v) as Box<dyn Fn() -> Result<(), JsValue>>),
                Err(e) => {
                    warn!("listen route event failed: {:?}", e);
                    None
                }
            };

            Msg::RegisterRouteListener(RouteListener { unlisten })
        });
    }
}

impl<R> Component for ListenSwitch<R>
where
    R: Routable + 'static,
{
    type Message = Msg;

    type Properties = SwitchProps<R>;

    fn create(ctx: &Context<Self>) -> Self {
        Self::listen_route(ctx);

        Self {
            route_unlisten: None,
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RegisterRouteListener(unlisten) => {
                self.route_unlisten = Some(unlisten);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <Switch<R> render={ ctx.props().render.clone() } />
        }
    }
}
