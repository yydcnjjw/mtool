use mapp::{
    anyhow, tokio,
    tracing::{debug, warn},
};
use mtool_cmdpal::CommandResult;
use std::process::Command;

pub struct OrgCapture {
    template: String,
    url: Option<String>,
    title: Option<String>,
    body: Option<String>,
}

impl OrgCapture {
    fn new<T>(template: T) -> Self
    where
        T: ToString,
    {
        Self {
            template: template.to_string(),
            url: None,
            title: None,
            body: None,
        }
    }

    pub fn open(self) -> Result<(), anyhow::Error> {
        let Self {
            template,
            url,
            title,
            body,
        } = self;

        let mut open_url = format!("org-protocol://capture?template={template}");

        if let Some(url) = url {
            open_url.push_str(&format!("&url={url}"));
        }

        if let Some(title) = title {
            open_url.push_str(&format!("&title={title}"));
        }

        if let Some(body) = body {
            open_url.push_str(&format!("&body={body}"));
        }

        tokio::spawn(async move {
            let output = if cfg!(windows) {
                Command::new("C:/Program Files/WSL/wslg.exe")
                    .args([
                        "-d",
                        "Arch",
                        "--cd",
                        "~",
                        "--",
                        "sh",
                        "-c",
                        &format!("emacsclient -c \"{open_url}\""),
                    ])
                    .current_dir("c:/windows/system32")
                    .output()
            } else {
                unimplemented!()
            };

            match output {
                Err(e) => {
                    warn!("{:?}", e);
                }
                Ok(output) => {
                    debug!("{:?}", output);
                }
            }
        });

        Ok(())
    }

    pub fn inbox() -> Self {
        Self::new("i")
    }

    pub fn project() -> Self {
        Self::new("p")
    }

    pub fn with_url<T>(mut self, url: T) -> Self
    where
        T: ToString,
    {
        self.url = Some(url.to_string());
        self
    }

    pub fn with_title<T>(mut self, title: T) -> Self
    where
        T: ToString,
    {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_body<T>(mut self, body: T) -> Self
    where
        T: ToString,
    {
        self.body = Some(body.to_string());
        self
    }
}

// fn get_clipboard_copy() -> Result<String, Box<(dyn Error + Send + Sync + 'static)>> {
//     let mut ctx = ClipboardContext::new()?;
//     ctx.get_contents()
// }

pub async fn capture_inbox() -> Result<CommandResult, anyhow::Error> {
    OrgCapture::inbox().with_title("").open()?;
    Ok(CommandResult::Dismiss)
}

pub async fn capture_project() -> Result<CommandResult, anyhow::Error> {
    OrgCapture::project().with_title("").open()?;
    Ok(CommandResult::Dismiss)
}
