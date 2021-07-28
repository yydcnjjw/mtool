use self::{
    api::{make_client, HttpResponse},
    credential::Credential,
    ocr::{GeneralBasicOCRResponse, OCRImage, OCRLanguageType},
};

mod api;
mod credential;
mod ocr;

pub async fn run(size: i64, data: *const u8) -> Result<(), reqwest::Error> {
    let base64img: String;

    unsafe {
        base64img = base64::encode(std::slice::from_raw_parts(data, size as usize));
    }

    let req = ocr::GeneralBasicOCRRequest::new(OCRImage::Base64(base64img), OCRLanguageType::Auto);
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

    println!(
        "{:?}",
        resp.response
            .text_detections
            .iter()
            .map(|td| td.detected_text.clone())
            .collect::<Vec<String>>()
    );

    Ok(())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn it_works() {
        // assert!(run().await.is_ok())
    }
}
