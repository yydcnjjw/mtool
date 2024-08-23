use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Debug, Clone, Properties, PartialEq, Serialize, Deserialize)]
pub struct ProgressBarProps {
    pub progress: usize,
}

#[function_component(ProgressBar)]
pub fn progress_bar(props: &ProgressBarProps) -> Html {
    html! {
    <div class={classes!(
         "flex",
         "w-full",
         "h-1.5",
         "bg-gray-200",
         "rounded-full",
         "overflow-hidden",
         "dark:bg-neutral-700"
         )}
         role="progressbar"
         aria-valuenow={format!("{}", props.progress)}
         aria-valuemin="0"
         aria-valuemax="100">
      <div class={classes!(
           "flex",
           "flex-col",
           "justify-center",
           "rounded-full",
           "overflow-hidden",
           "bg-blue-600",
           "text-xs",
           "text-white",
           "text-center",
           "whitespace-nowrap",
           "transition",
           "duration-500",
           "dark:bg-blue-500"
           )}
           style={format!("width: {}%", props.progress)}/>
    </div>
    }
}
