use std::{any::type_name, marker::PhantomData};

use anyhow::Context;
use dashmap::DashMap;
use send_wrapper::SendWrapper;
use serde::de::DeserializeOwned;
use yew::prelude::*;

pub trait Template {
    fn render(&self, props: &serde_json::Value) -> Result<Html, anyhow::Error>;
}

struct ComponentTemplate<T> {
    _marker: PhantomData<SendWrapper<T>>,
}

impl<T> Template for ComponentTemplate<T>
where
    T: BaseComponent,
    T::Properties: DeserializeOwned,
{
    fn render(&self, props: &serde_json::Value) -> Result<Html, anyhow::Error> {
        let props = serde_json::from_value::<T::Properties>(props.clone())?;

        Ok(html! {
            <T ..props></T>
        })
    }
}

pub struct Templator {
    templates: DashMap<String, Box<dyn Template + Send + Sync>>,
}

impl PartialEq for Templator {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Templator {
    pub fn new() -> Self {
        Self {
            templates: DashMap::default(),
        }
    }

    pub fn add_template<T>(&self)
    where
        T: BaseComponent + 'static,
        T::Properties: DeserializeOwned,
    {
        self.templates.insert(
            type_name::<T>().to_string(),
            Box::new(ComponentTemplate::<T> {
                _marker: PhantomData,
            }),
        );
    }

    pub fn render(&self, id: &str, props: &serde_json::Value) -> Result<Html, anyhow::Error> {
        self.templates
            .get(id)
            .context(format!("{} is not exist", id))?
            .render(props)
    }
}
