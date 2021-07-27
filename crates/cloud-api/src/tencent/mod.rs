use self::{
    api::{make_client, HttpResponse},
    credential::Credential,
    ocr::{GeneralBasicOCRResponse, OCRImage, OCRLanguageType},
};

mod api;
mod credential;
mod ocr;

pub async fn run() -> Result<(), reqwest::Error> {
    let req = ocr::GeneralBasicOCRRequest::new(
        OCRImage::Base64(base64::encode(include_bytes!("/tmp/test.png"))),
        OCRLanguageType::Auto,
    );
    let cred = Credential::new(
        String::from("AKIDxnU3TlI54iG0b3wfLjw1ylV7bxmVHYyF"),
        String::from("GI4Wu69LHxOSJvMYVH1bK0b4nrrlX4LS"),
    );
    let cli = make_client(req, cred);
    let resp = cli
        .send()
        .await?
        .json::<HttpResponse<GeneralBasicOCRResponse>>()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;

    #[tokio::test]
    async fn it_works() {
        assert!(run().await.is_ok())
    }
}
