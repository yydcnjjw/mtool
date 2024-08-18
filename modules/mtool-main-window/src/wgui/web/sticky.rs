use mapp::prelude::*;
use mtauri_sys::window::{PhysicalPosition, Window};
use mtool_wgui::prelude::*;
use yew::{platform::spawn_local, prelude::*};

#[function_component]
fn StickyView() -> Html {
    let onmousemove = {
        Callback::from(move |e: MouseEvent| {
            if e.ctrl_key() {
                let (move_x, move_y) = (e.movement_x(), e.movement_y());
                spawn_local(async move {
                    let win = Window::current().unwrap();
                    let PhysicalPosition { x, y } = win.outer_position().await.unwrap();
                    win.set_position(PhysicalPosition::new(x + move_x, y + move_y).into())
                        .await
                        .unwrap();
                });
            }
        })
    };

    html! {
        <AutoWindow window={
            WindowProps{
                horizontal: Horizontal::RightAlign,
                vertical: Vertical::Absolute(24),
                resizable: true,
                ..Default::default()
            }
        }>
          <div class={classes!(
              "flex",
              "flex-col",
              "p-1",
              "border-solid",
              "border-2",
              "w-screen",
              "h-screen",
              "bg-gray-600/75",
            )}
            {onmousemove}>
            {{ "sticky window" }}
          </div>
        </AutoWindow>
    }
}

fn render(_: &RouteParams) -> Html {
    html! {
        <StickyView/>
    }
}

pub async fn register(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/sticky", render);
    Ok(())
}
