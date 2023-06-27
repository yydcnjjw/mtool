use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Properties, PartialEq, Serialize, Deserialize)]
pub struct QueryResult {
    pub result: Vec<String>,
}

#[function_component]
pub fn DictView(props: &QueryResult) -> Html {
    html! {
        <div>
        {
            props.result.iter().map(|item| {
                html! {
                    <div> { Html::from_html_unchecked(AttrValue::from(item.clone())) } </div>
                }
            }).collect::<Html>()
        }
        </div>
    }
}
