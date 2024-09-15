mod view;

use view::View;

use mapp::{anyhow, prelude::*};
use mtool_wgui::prelude::*;
use yew::prelude::*;

fn render(_: &RouteParams) -> Html {
    html! {
        <View/>
    }
}

pub async fn init(router: Res<Router>) -> Result<(), anyhow::Error> {
    router.add("/sticky", render);
    Ok(())
}
