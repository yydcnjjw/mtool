use std::ops::Deref;

use mproxy::protos::geosite;
use mtool_interactive::CompleteItem;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone, Serialize, Deserialize)]
pub struct GeositeItem {
    data: geosite::Domain,
}

impl GeositeItem {
    #[allow(unused)]
    pub fn new(data: geosite::Domain) -> Self {
        Self { data }
    }
}

impl Deref for GeositeItem {
    type Target = geosite::Domain;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl CompleteItem for GeositeItem {
    type WGuiView = GeositeItemView;

    fn complete_hint(&self) -> String {
        self.data.value.clone()
    }
}

#[function_component]
pub fn GeositeItemView(props: &GeositeItem) -> Html {
    let type_ = match props.type_.enum_value() {
        Ok(v) => match v {
            geosite::domain::Type::Plain => "Plain",
            geosite::domain::Type::Regex => "Regex",
            geosite::domain::Type::Domain => "Domain",
            geosite::domain::Type::Full => "Full",
        },
        Err(_) => "Unknown",
    };
    html! {
        <div>
            <span>
              { type_ }
            </span>
            {": "}
            <span>
              { props.value.clone() }
            </span>
        </div>
    }
}
