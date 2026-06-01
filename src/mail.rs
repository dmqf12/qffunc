use anyhow::{Context, Result};
use lettre::{
    message::{Mailbox, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// SMTP 配置
#[derive(Debug, Clone, Deserialize)]
pub struct MailConfig {
    pub smtp_server: String,
    pub username: String,
    pub auth_code: String,
    pub from_name: String,
}

/// 邮件消息（构建器模式）
#[derive(Debug, Clone)]
pub struct MailMessage {
    to: Option<String>,
    subject: Option<String>,
    body: Option<String>,
    is_html: bool,
    attachments: Vec<(PathBuf, Option<String>)>,
}

impl MailMessage {
    pub fn new() -> Self {
        Self {
            to: None,
            subject: None,
            body: None,
            is_html: false,
            attachments: vec![],
        }
    }

    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn html(mut self, is_html: bool) -> Self {
        self.is_html = is_html;
        self
    }

    /// 添加附件，自动取文件名和 MIME 类型
    pub fn attach(mut self, path: impl Into<PathBuf>) -> Self {
        self.attachments.push((path.into(), None));
        self
    }

    pub async fn send(self, config: &MailConfig) -> Result<()> {
        let from: Mailbox = config.from_name.parse()
            .with_context(|| format!("无效的发件人地址 '{}'", config.from_name))?;
        let to: Mailbox = self.to.ok_or(anyhow::anyhow!("缺少收件人"))?
            .parse()
            .with_context(|| "无效的收件人地址")?;
        let subject = self.subject.ok_or(anyhow::anyhow!("缺少主题"))?;
        let body = self.body.ok_or(anyhow::anyhow!("缺少正文"))?;

        let email = if self.attachments.is_empty() {
            let mut builder = Message::builder()
                .from(from.clone())
                .to(to.clone())
                .subject(subject);
            if self.is_html {
                builder = builder.header(lettre::message::header::ContentType::TEXT_HTML);
            }
            builder.body(body)?
        } else {
            let mut multipart = MultiPart::mixed()
                .singlepart(if self.is_html {
                    SinglePart::html(body)
                } else {
                    SinglePart::plain(body)
                });

            for (path, mime_override) in &self.attachments {
                let data = std::fs::read(path)
                    .with_context(|| format!("读取附件失败 '{}'", path.display()))?;
                let filename = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let mime = mime_override.clone()
                    .unwrap_or_else(|| guess_mime(path));

                let part = SinglePart::builder()
                    .header(lettre::message::header::ContentType::parse(&mime)?)
                    .header(lettre::message::header::ContentDisposition::attachment(&filename))
                    .body(data);
                multipart = multipart.singlepart(part);
            }

            Message::builder()
                .from(from.clone())
                .to(to.clone())
                .subject(subject)
                .multipart(multipart)?
        };

        let creds = lettre::transport::smtp::authentication::Credentials::new(
            config.username.clone(),
            config.auth_code.clone(),
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)
            .with_context(|| format!("无效的 SMTP 服务器 '{}'", config.smtp_server))?
            .credentials(creds)
            .port(465)
            .build();

        mailer.send(email).await?;
        Ok(())
    }
}

fn guess_mime(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("flac") => "audio/flac",
        Some("ogg") => "audio/ogg",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        Some("txt") => "text/plain",
        Some("html") | Some("htm") => "text/html",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }.into()
}

impl Default for MailMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl MailConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }
}
