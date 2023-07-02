use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Properties, PartialEq, Serialize, Deserialize)]
pub struct QueryResult {
    pub word: String,
    pub phonetic: Option<String>,
    pub definition: Vec<String>,
    pub translation: Vec<String>,
    pub pos: Vec<String>,
    pub collins: Option<i32>,
    pub oxford: Option<i32>,
    pub tag: Vec<String>,
    pub bnc: Option<i32>,
    pub frq: Option<i32>,
    pub exchange: Vec<String>,
    pub detail: Option<String>,
    pub audio: Option<String>,
}

#[function_component]
pub fn DictView(props: &QueryResult) -> Html {
    let QueryResult {
        phonetic,
        translation,
        collins,
        oxford,
        tag,
        bnc,
        frq,
        exchange,
        ..
    } = &props;

    let tag_map = HashMap::from([
        ("zk", "中"),
        ("gk", "高"),
        ("ky", "研"),
        ("cet4", "四"),
        ("cet6", "六"),
        ("toefl", "托"),
        ("ielts", "雅"),
        ("gre", "宝"),
    ]);

    let exchange_map = HashMap::from([
        ("p", "过去式"),
        ("d", "过去分词"),
        ("i", "现在分词"),
        ("3", "第三人称单数"),
        ("s", "名词复数"),
        ("r", "形容词比较级"),
        ("t", "形容词最高级"),
        ("0", "原型"),
        ("1", "原型变换"),
    ]);

    let exchange = exchange
        .iter()
        .filter_map(|item| item.split_once(":"))
        .into_group_map_by(|(exchange, _)| match *exchange {
            "p" | "d" | "i" => "时态",
            "s" | "3" => "单复数",
            "r" | "t" => "比较级",
            _ => "原型",
        });

    html! {
        <div>
          <div>
            if let Some(phonetic) = phonetic {
              <span>{ format!("[{phonetic}]") }</span>
            }
            if collins.is_some() || oxford.is_some() {
              <span>{ "-" }</span>
              if oxford.is_some() {
                <span title={"Oxford 3000 Keywords"}>{ "※" }</span>
              }

              if let Some(collins) = collins.map(|n| "★".repeat(n as usize)) {
                <span title={"Collins Stars"}>{ collins }</span>
              }
            }
          </div>
          <div>
            {
              translation.iter().map(|line| {
                let line = line.trim();
                html! {
                  <div>
                    if line.starts_with("[网络]") {
                      <span>{ "[网络]" }</span>
                      <span>{ &line[4..].trim_start() }</span>
                    } else if line.starts_with(">") {
                      <span>{ line }</span>
                    } else if let Some((pos, text)) = line.split_once('.') {
                      <span>{ format!("{}.", pos) }</span>
                      <span>{ text.trim_start() }</span>
                    } else {
                      <span> { line }</span>
                    }
                  </div>
                }
              }).collect::<Html>()
            }
          </div>
          <div class={classes!("text-sm")}>
            {
              exchange.iter().map(|(key, exchange)| {
                html!{
                  <div>
                    <div>{ format!("[{}]", key) }</div>
                    {
                      exchange.iter().map(|(exchange, value)| html!{
                        <span>{ format!("{}({})/", value, exchange_map[exchange]) }</span>
                      }).collect::<Html>()
                    }
                  </div>
                }
              }).collect::<Html>()
            }
          </div>
          <div class={classes!("text-sm")}>
            {
              tag.iter().map(|tag| {
                html!{
                  <span>
                    { format!("{}/", tag_map[tag.as_str()]) }
                  </span>
                }
              }).collect::<Html>()
            }
            if let Some(frq) = frq {
              <span>{ format!("COCA:{}/", frq) }</span>
            }

            if let Some(bnc) = bnc {
              <span>{ format!("BNC:{}", bnc) }</span>
            }
          </div>
        </div>
    }
}
