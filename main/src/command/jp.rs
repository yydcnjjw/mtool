pub fn query<'a>(query: &'a str) -> BoxFuture<'a, Result<WordInfo>> {
    async move {
        match get_dict(query).await {
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
                }

                let mut word_info = word_infos.remove(i);

                if word_info.expression == query || word_info.pronounce.pronounce == query {
                    word_info.expression = format!(
                        "{}[{}]",
                        word_info.expression, word_info.pronounce.pronounce
                    );
                }

                Ok(word_info)
            }
            Err(e) => match e {
                Error::WordSuggestion(v) => {
                    let i = cli_op::read_choice("word suggestions: ", &v);
                    query_dict(v.get(i).unwrap()).await
                }
                _ => Err(e.into()),
            },
        }
    }
    .boxed()
}
