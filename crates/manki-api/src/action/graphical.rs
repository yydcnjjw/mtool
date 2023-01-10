use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    api::{self, InvokeResult},
    message::Request,
};

type InvokeResultNoneParams<Result> = InvokeResult<Result, ()>;

pub async fn gui_start_card_timer() -> InvokeResultNoneParams<bool> {
    api::invoke(Request::new("guiStartCardTimer")).await
}

pub async fn gui_show_question() -> InvokeResultNoneParams<bool> {
    api::invoke(Request::new("guiShowQuestion")).await
}

pub async fn gui_show_answer() -> InvokeResultNoneParams<bool> {
    api::invoke(Request::new("guiShowAnswer")).await
}

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum Ease {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

#[derive(Serialize, Debug)]
pub struct GuiAnswerCardParams {
    pub ease: Ease,
}

pub async fn gui_answer_card(ease: Ease) -> InvokeResult<bool, GuiAnswerCardParams> {
    api::invoke(Request::new_with_params(
        "guiShowAnswer",
        GuiAnswerCardParams { ease },
    ))
    .await
}

#[derive(Deserialize, Debug)]
pub struct GuiCurrentCardField {
    pub value: String,
    pub order: isize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GuiCurrentCard {
    pub answer: String,
    pub question: String,
    pub deck_name: String,
    pub model_name: String,
    pub field_order: isize,
    pub fields: HashMap<String, GuiCurrentCardField>,
    pub template: String,
    pub css: String,
    pub card_id: usize,
    pub buttons: Vec<usize>,
    pub next_reviews: Vec<String>,
}

pub async fn gui_current_card() -> InvokeResultNoneParams<Option<GuiCurrentCard>> {
    api::invoke(Request::new("guiCurrentCard")).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gui_current_card() {
        println!("{:?}", gui_current_card().await)
    }
}
