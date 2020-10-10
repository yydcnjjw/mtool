use crate::util::cli_op;
use anki::AnkiNote;
use anki::AnkiNoteOptions;
use futures::future::{BoxFuture, FutureExt};
use hjdict::JPWord;

pub async fn save(word_info: &JPWord) -> Result<(), Box<dyn std::error::Error>> {
    if !cli_op::read_y_or_n("add note[Y/n]") {
        return Ok(());
    }

    let mut note = AnkiNote::new(&word_info, Option::None);
    if !anki::can_add_note(&note).await? {
        println!("{}: duplicate!", word_info.expression);
        return Ok(());
    }
    note.options = Some(AnkiNoteOptions {
        allow_duplicate: false,
        duplicate_scope: "deck".to_string(),
    });
    anki::add_note(&note).await?;
    Ok(())
}

pub fn query<'a>(query_text: &'a str) -> BoxFuture<'a, Result<JPWord, Box<dyn std::error::Error>>> {
    async move {
        match hjdict::get_jp_dict(query_text).await {
            Ok(mut word_infos) => {
                let mut i = 0;

                if word_infos.len() > 1 {
                    i = cli_op::read_choice(
                        "multi words",
                        &word_infos
                            .iter()
                            .map(|v| format!("{}[{}]", v.expression, v.pronounce.pronounce))
                            .collect(),
                    );
                    let word_info = word_infos.get_mut(i).unwrap();
                    if word_info.expression == query_text
                        || word_info.pronounce.pronounce != query_text
                    {
                        word_info.expression = format!(
                            "{}[{}]",
                            word_info.expression, word_info.pronounce.pronounce
                        );
                    }
                }

                Ok(word_infos.remove(i))
            }
            Err(e) => match e {
                hjdict::Error::WordSuggestion(v) => {
                    let i = cli_op::read_choice("word suggestions: ", &v);
                    query(v.get(i).unwrap()).await
                }
                _ => Err(e.into()),
            },
        }
    }
    .boxed()
}
