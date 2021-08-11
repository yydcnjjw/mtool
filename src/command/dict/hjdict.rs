// use anki::AnkiNote;
// use anki::AnkiNoteOptions;
// use futures::future::{BoxFuture, FutureExt};
use super::{Dict, DictCap, DictQuery, Lang};
use async_trait::async_trait;

// pub async fn save(word_info: &JPWord) -> Result<(), Box<dyn std::error::Error>> {
//     if !cli_op::read_y_or_n("add note[Y/n]") {
//         return Ok(());
//     }

//     let mut note = AnkiNote::new(&word_info, Option::None);
//     if !anki::can_add_note(&note).await? {
//         println!("{}: duplicate!", word_info.expression);
//         return Ok(());
//     }
//     note.options = Some(AnkiNoteOptions {
//         allow_duplicate: false,
//         duplicate_scope: "deck".to_string(),
//     });
//     anki::add_note(&note).await?;
//     Ok(())
// }

pub struct HJDict {}

impl Dict for HJDict {}

#[async_trait]
impl DictQuery for HJDict {
    async fn query(&self, text: &String) -> Vec<String> {
        hjdict::api::query_jp_dict(text)
            .await
            .map(|result| result.iter().map(|word| word.to_cli_str()).collect())
            .unwrap_or_default()
    }
}

impl DictCap for HJDict {
    fn support_languages(&self) -> Vec<super::Lang> {
        vec![Lang::JP]
    }
}

// pub fn query<'a>(query_text: &'a str) -> BoxFuture<'a, Result<JPWord, Box<dyn std::error::Error>>> {
//     async move {
//         match hjdict::api::query_jp_dict(query_text.into()).await {
//             Ok(mut word_infos) => {
//                 let mut i = 0;

//                 if word_infos.len() > 1 {
//                     i = cli_op::read_choice(
//                         "multi words",
//                         &word_infos
//                             .iter()
//                             .map(|v| format!("{}[{}]", v.expression, v.pronounce.pronounce))
//                             .collect(),
//                     );
//                     let word_info = word_infos.get_mut(i).unwrap();
//                     if word_info.expression == query_text
//                         || word_info.pronounce.pronounce != query_text
//                     {
//                         word_info.expression = format!(
//                             "{}[{}]",
//                             word_info.expression, word_info.pronounce.pronounce
//                         );
//                     }
//                 }

//                 Ok(word_infos.remove(i))
//             }
//             Err(e) => match e {
//                 hjdict::Error::WordSuggestion(v) => {
//                     let i = cli_op::read_choice("word suggestions: ", &v);
//                     query(v.get(i).unwrap()).await
//                 }
//                 _ => Err(e.into()),
//             },
//         }
//     }
//     .boxed()
// }