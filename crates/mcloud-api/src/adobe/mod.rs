mod pdf_structure;

pub use pdf_structure::*;
use tracing::debug;

use std::{collections::HashMap, fmt::Debug, time::Duration};

use anyhow::Context;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Body,
};
use serde::{Deserialize, Serialize};

type AssetId = String;

pub struct Client {
    inner: reqwest::Client,
    basic_url: String,
    access_token: String,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("basic_url", &self.basic_url)
            .field("access_token", &self.access_token)
            .finish()
    }
}

impl Client {
    pub async fn new(url: &str, client_id: &str, key: &str) -> Result<Self, anyhow::Error> {
        let access_token = Self::get_access_token(url, client_id, key).await?;

        let mut headers = HeaderMap::default();
        headers.insert("x-api-key", HeaderValue::from_str(client_id)?);
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {access_token}"))?,
        );

        Ok(Self {
            inner: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()?,
            basic_url: url.into(),
            access_token: access_token.into(),
        })
    }

    fn from_access_token(
        url: &str,
        client_id: &str,
        access_token: &str,
    ) -> Result<Self, anyhow::Error> {
        let mut headers = HeaderMap::default();
        headers.insert("x-api-key", HeaderValue::from_str(client_id)?);
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {access_token}"))?,
        );

        Ok(Self {
            inner: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()?,
            basic_url: url.into(),
            access_token: access_token.into(),
        })
    }

    pub async fn get_access_token(
        url: &str,
        client_id: &str,
        key: &str,
    ) -> Result<String, anyhow::Error> {
        #[derive(Debug, Deserialize)]
        #[allow(unused)]
        struct Response {
            access_token: String,
            token_type: String,
            expires_in: isize,
        }

        let mut params = HashMap::new();
        params.insert("client_id", client_id);
        params.insert("client_secret", key);
        Ok(reqwest::Client::new()
            .post(format!("{url}/token"))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?
            .access_token)
    }

    pub async fn upload_asset<T>(
        &self,
        media_type: &str,
        asset: T,
    ) -> Result<AssetId, anyhow::Error>
    where
        T: Into<Body>,
    {
        #[derive(Debug, Serialize)]
        struct Request {
            #[serde(rename = "mediaType")]
            media_type: String,
        }

        #[derive(Debug, Deserialize)]
        pub struct Response {
            #[serde(rename = "assetID")]
            assets_id: String,
            #[serde(rename = "uploadUri")]
            upload_uri: String,
        }

        let resp = self
            .inner
            .post(format!("{}/assets", self.basic_url))
            .json(&Request {
                media_type: media_type.into(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;

        reqwest::Client::builder()
            .build()?
            .put(&resp.upload_uri)
            .header(CONTENT_TYPE, HeaderValue::from_str(media_type)?)
            .body(asset)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.assets_id)
    }

    pub async fn get_download_asset(&self, asset_id: &AssetId) -> Result<String, anyhow::Error> {
        #[derive(Debug, Deserialize)]
        pub struct Response {
            #[serde(rename = "downloadUri")]
            download_uri: String,
            size: usize,
            r#type: String,
        }

        Ok(self
            .inner
            .get(format!("{}/assets/{asset_id}", self.basic_url))
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?
            .download_uri)
    }

    pub async fn extract_pdf(&self, asset_id: AssetId) -> Result<PdfStructure, anyhow::Error> {
        #[derive(Debug, Serialize)]
        struct Request {
            #[serde(rename = "assetID")]
            asset_id: String,
            #[serde(rename = "getCharBounds")]
            get_char_bounds: bool,
            #[serde(rename = "includeStyling")]
            include_styling: bool,
            #[serde(rename = "elementsToExtract")]
            elements_to_extract: Vec<&'static str>,
            #[serde(rename = "tableOutputFormat")]
            table_output_format: String,
            #[serde(rename = "renditionsToExtract")]
            renditions_to_extract: Vec<&'static str>,
        }

        let resp = self
            .inner
            .post(format!("{}/operation/extractpdf", self.basic_url))
            .json(&Request {
                asset_id: asset_id.clone(),
                get_char_bounds: true,
                include_styling: false,
                elements_to_extract: vec!["text", "tables"],
                table_output_format: "csv".into(),
                renditions_to_extract: vec![],
            })
            .send()
            .await?
            .error_for_status()?;

        let location = resp
            .headers()
            .get("location")
            .context("location header is not exist")?
            .to_str()?;

        debug!("{location}");

        let content = loop {
            #[derive(Debug, Deserialize)]
            struct AssetMetadata {
                size: usize,
                r#type: String,
            }

            #[derive(Debug, Deserialize)]
            struct Asset {
                metadata: AssetMetadata,
                #[serde(rename = "assetID")]
                asset_id: String,
                #[serde(rename = "downloadUri")]
                download_uri: String,
            }

            #[derive(Debug, Deserialize)]
            struct Response {
                status: String,
                content: Option<Asset>,
                resource: Option<Asset>,
            }

            let resp: Response = self
                .inner
                .get(location)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            match resp.status.as_str() {
                "done" => break resp.content.context("bad response")?,
                "failed" => anyhow::bail!("extract pdf failed"),
                _ => {
                    debug!("in progress");
                }
            };

            tokio::time::sleep(Duration::from_secs(1)).await;
        };

        let download_uri = self.get_download_asset(&content.asset_id).await?;

        debug!("{download_uri}");

        Ok(reqwest::get(download_uri)
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;
    use tokio::fs::{self, File};

    use super::*;

    #[test(tokio::test)]
    async fn test_pdf_extract() -> Result<(), anyhow::Error> {
        let cli = Client::new(
            "https://pdf-services.adobe.io",
            "541915cb51f14e2eb486939cbd538f99",
            "p8e-cCRzoAa675zQyapr2xdAYFyRbvwnR5Nq",
        )
        .await?;

        let pdf = fs::read("/mnt/d/book/art.pdf").await?;
        let asset_id = cli.upload_asset("application/pdf", pdf).await?;

        debug!("{asset_id}");

        debug!("{:?}", cli.extract_pdf(asset_id).await?);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::fs::File;

    use super::*;

    #[tokio::test]
    async fn test_extract_pdf() -> Result<(), anyhow::Error> {
        // let cli = Client::new(
        //     "https://pdf-services.adobe.io",
        //     "541915cb51f14e2eb486939cbd538f99",
        //     "p8e-cCRzoAa675zQyapr2xdAYFyRbvwnR5Nq",
        // )
        // .await?;
        let cli = Client::from_access_token("https://pdf-services.adobe.io", "541915cb51f14e2eb486939cbd538f99", "eyJhbGciOiJSUzI1NiIsIng1dSI6Imltc19uYTEta2V5LWF0LTEuY2VyIiwia2lkIjoiaW1zX25hMS1rZXktYXQtMSIsIml0dCI6ImF0In0.eyJpZCI6IjE2OTQ3NDAyMzU0NjVfOWU2Nzk5ZmQtNjZhMS00ZDViLTkzZTEtYjNmMWE1NWVmMzllX3VlMSIsIm9yZyI6IkVEMTcxREUxNjRDMjNEMEEwQTQ5NUNGNEBBZG9iZU9yZyIsInR5cGUiOiJhY2Nlc3NfdG9rZW4iLCJjbGllbnRfaWQiOiI1NDE5MTVjYjUxZjE0ZTJlYjQ4NjkzOWNiZDUzOGY5OSIsInVzZXJfaWQiOiJGOUM4MUU2RDY0QzIzRDU2MEE0OTVFNDZAdGVjaGFjY3QuYWRvYmUuY29tIiwiYXMiOiJpbXMtbmExIiwiYWFfaWQiOiJGOUM4MUU2RDY0QzIzRDU2MEE0OTVFNDZAdGVjaGFjY3QuYWRvYmUuY29tIiwiY3RwIjozLCJtb2kiOiI1N2JmM2VkIiwiZXhwaXJlc19pbiI6Ijg2NDAwMDAwIiwic2NvcGUiOiJEQ0FQSSxvcGVuaWQsQWRvYmVJRCIsImNyZWF0ZWRfYXQiOiIxNjk0NzQwMjM1NDY1In0.GETyS75S7D8q4FrKZzJvNowGMDIrOXx213AiJ9cB2tryP9C-htEuXBSfFKW5S3AqIKIb5yiuCjUDoAcLLZx8NCJ8HofPQXQL7yyFhVnyCe0Ypg67NTDAiN3El_fszFvAZA2WI2Sdq6em6_57Ba00EXtulfuarrFqzrTireDB2kchBWz5YzQ61AOK_41Fnlvt2F4PdrgMBQff8QnQLDfaNehBYb6or4PY57C9zCs6gi0GOjJ9Ie79dlVwXDZR2LI1YT74UpiWMDwiOahJ6GKS0phH8j6NClwGX7pj04JL9cOwkV5-a3pOF-5FxGABZ94Jx6KnFsV2eWY3cH56jr6k8Q")?;

        println!("{:?}", cli);

        // let asset_id = cli
        //     .upload_asset(
        //         "application/pdf",
        //         File::open("/home/yydcnjjw/resource/book/art.pdf").await?,
        //     )
        //     .await?;

        let asset_id = "urn:aaid:AS:UE1:b7fba28a-7463-4f1a-807b-b370aed0a143".to_string();

        println!("{:?}", asset_id);

        let json = cli.extract_pdf(asset_id).await?;

        println!("{:?}", json);

        Ok(())
    }
}
