use log::Level;
use mtool_service::*;
use wasm_bindgen::prelude::*;

use stylist::{css, yew::Global};
use yew::prelude::*;

enum Msg {
    AddOne,
}

struct Model {
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
          <>
            <Global css=r#"
              html, body {
                font-family: Hack;
                padding: 0;
                margin: 0;
                display: flex;
                min-height: 100vh;
                padding-left: 1vw;
                padding-right: 1vw;
                flex-direction: column;
              }"#
            />
            <div class={css!(r#"
              width: 100vw;
              height: 50px;
              margin-top: 10px;
              display: flex;
              flex-direction: row;
              justify-content: center;"#)}
             >
              <input class={css!(r#"
                width: 100%;
                font-size: 48px;
                outline: none;
                border: none;"#)}
                id="cmder"
                type="text"
               />
            </div>
            <div> { "Hello World!" } </div>
          </>
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let tx = match mrpc::net::websocket::connect("ws://127.0.0.1:8080").await {
        Ok(v) => v,
        Err(e) => {
            anyhow::bail!("{:?}", e);
        }
    };

    let cli = ServerClient::new(tx);

    let v = cli.config().get_value("translate".into()).await?;

    log::debug!("{:?}", v);

    Ok(())
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    main();
}

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();

    mrpc::spawn_local(async move {
        if let Err(e) = run().await {
            log::error!("{:?}", e);
        }
    });
    yew::start_app::<Model>();
}
