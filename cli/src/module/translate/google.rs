use super::Lang;
use translate::google;

fn lang_to_google(lang: &Lang) -> &str {
    match lang {
        Lang::AUTO => "auto",
        Lang::ZH => "zh-CN",
        Lang::EN => "en",
    }
}

pub async fn query(
    query: &str,
    from: &Lang,
    to: &Lang,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(google::query(query, lang_to_google(from), lang_to_google(to)).await?)
}
