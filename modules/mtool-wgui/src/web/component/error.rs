use yew::prelude::*;

pub fn error_view(e: &anyhow::Error) -> Html {
    html! {
        <div class={classes!("w-full",
                             "h-full",
                             "overflow-auto",
                             "whitespace-pre",
                             "text-xs")}>
          {
            format!("{:?}", e)
          }
        </div>
    }
}

pub fn render_result_view(view: Result<Html, anyhow::Error>) -> Html {
    match view {
        Ok(view) => view,
        Err(e) => html! {
            <div class={classes!("w-full",
                                 "h-full",
                                 "overflow-auto",
                                 "whitespace-pre",
                                 "text-xs")}>
              {
                format!("{:?}", e)
              }
            </div>
        },
    }
}
