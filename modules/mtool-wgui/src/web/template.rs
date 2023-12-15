use std::{any::type_name, marker::PhantomData};

use anyhow::Context;
use async_trait::async_trait;
use dashmap::DashMap;
use mapp::prelude::*;
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Deserialize};
use yew::prelude::*;

use crate::{WebAppContext, WebStage};

pub type TemplateId = String;
pub type TemplateData = serde_json::Value;

pub trait Template {
    fn render(&self, props: &TemplateData) -> Result<Html, anyhow::Error>;
}

struct ComponentTemplate<T> {
    _marker: PhantomData<SendWrapper<T>>,
}

impl<T> Template for ComponentTemplate<T>
where
    T: BaseComponent,
    T::Properties: DeserializeOwned,
{
    fn render(&self, props: &TemplateData) -> Result<Html, anyhow::Error> {
        let props = serde_json::from_value::<T::Properties>(props.clone())?;

        Ok(html! {
            <T ..props></T>
        })
    }
}

pub struct Templator {
    templates: DashMap<TemplateId, Box<dyn Template + Send + Sync>>,
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

    pub fn render(&self, id: &TemplateId, props: &TemplateData) -> Result<Html, anyhow::Error> {
        self.templates
            .get(id)
            .context(format!("{} is not exist", id))?
            .render(&props)
    }
}

#[derive(Properties, PartialEq, Clone, Deserialize)]
pub struct Props {
    pub template_id: TemplateId,
    pub data: serde_json::Value,
}

#[function_component]
pub fn TemplateView(props: &Props) -> Html {
    let context = use_context::<WebAppContext>().expect("no context found");

    match context.templator.render(&props.template_id, &props.data) {
        Ok(view) => view,
        Err(e) => html! {
            { format!("{:?}", e) }
        },
    }
}

#[function_component]
pub fn EmptyView() -> Html {
    html! {}
}

pub struct Module;

#[async_trait(?Send)]
impl AppLocalModule for Module {
    async fn local_init(&self, ctx: &mut LocalAppContext) -> Result<(), anyhow::Error> {
        ctx.injector().insert(Res::new(Templator::new()));
        ctx.schedule()
            .add_once_task(WebStage::Init, |templator: Res<Templator>| async move {
                templator.add_template::<EmptyView>();
                Ok::<(), anyhow::Error>(())
            });
        Ok(())
    }
}
