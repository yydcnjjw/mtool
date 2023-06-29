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
