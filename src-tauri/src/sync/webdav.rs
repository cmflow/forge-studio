// 通用 WebDAV 客户端：只做文件级读写，不含任何业务逻辑。
// 设计目标是可以整个文件复制到其它项目复用，所以这里不引用 models / 事件相关类型。
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// WebDAV 连接凭据（存在共享的 credential.json 中）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebdavCredential {
    /// 形如 https://dav.jianguoyun.com/dav/
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub account: String,
    /// 坚果云「第三方应用密码」，非登录密码
    #[serde(default)]
    pub app_password: String,
}

impl WebdavCredential {
    pub fn is_filled(&self) -> bool {
        !self.server.trim().is_empty()
            && !self.account.trim().is_empty()
            && !self.app_password.trim().is_empty()
    }
}

/// 远端文件的元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStat {
    pub exists: bool,
    /// 字节数，未知则为 0
    pub size: u64,
    /// 服务端 Last-Modified 原文，未知则为空
    pub last_modified: String,
    /// ETag，用于判断内容是否变化
    pub etag: String,
}

/// PROPFIND 列出的目录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEntry {
    /// 文件名（不含路径）
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: String,
}

pub struct WebdavClient {
    client: Client,
    base: String,
    account: String,
    password: String,
}

impl WebdavClient {
    pub fn new(cred: &WebdavCredential) -> Result<Self, String> {
        if !cred.is_filled() {
            return Err("WebDAV 凭据不完整".into());
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| e.to_string())?;
        // 统一保证 base 以 / 结尾，后续直接拼相对路径
        let mut base = cred.server.trim().to_string();
        if !base.ends_with('/') {
            base.push('/');
        }
        Ok(Self {
            client,
            base,
            account: cred.account.trim().to_string(),
            password: cred.app_password.trim().to_string(),
        })
    }

    /// Basic Auth 头值
    fn auth(&self) -> String {
        format!(
            "Basic {}",
            BASE64.encode(format!("{}:{}", self.account, self.password))
        )
    }

    /// 把相对路径（如 apps/forge-studio/events.json）拼成完整 URL
    fn url(&self, rel: &str) -> String {
        format!("{}{}", self.base, rel.trim_start_matches('/'))
    }

    /// 探测连通性与鉴权：对根目录发 PROPFIND，Depth: 0
    pub fn check(&self) -> Result<(), String> {
        let resp = self
            .client
            .request(
                reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
                self.url(""),
            )
            .header("Authorization", self.auth())
            .header("Depth", "0")
            .send()
            .map_err(|e| format!("网络请求失败：{}", e))?;

        match resp.status() {
            // 207 Multi-Status 是 WebDAV 正常响应
            StatusCode::MULTI_STATUS | StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => {
                Err("鉴权失败（401）：请确认账号与『应用密码』是否正确".into())
            }
            s => Err(format!("服务器返回异常状态：{}", s)),
        }
    }

    /// 创建目录（MKCOL）。已存在（405）视为成功。
    fn mkcol(&self, dir: &str) -> Result<(), String> {
        let resp = self
            .client
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), self.url(dir))
            .header("Authorization", self.auth())
            .send()
            .map_err(|e| format!("创建目录失败：{}", e))?;

        match resp.status() {
            StatusCode::CREATED | StatusCode::OK => Ok(()),
            // 405 Method Not Allowed = 目录已存在
            StatusCode::METHOD_NOT_ALLOWED => Ok(()),
            StatusCode::UNAUTHORIZED => Err("鉴权失败（401）".into()),
            StatusCode::CONFLICT => Err(format!("父目录不存在，无法创建 {}", dir)),
            s => Err(format!("创建目录 {} 失败：{}", dir, s)),
        }
    }

    /// 逐级确保目录存在。传入的是「目录」相对路径，如 apps/forge-studio
    pub fn ensure_dir(&self, dir: &str) -> Result<(), String> {
        let parts: Vec<&str> = dir
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let mut cur = String::new();
        for p in parts {
            cur.push_str(p);
            cur.push('/');
            self.mkcol(&cur)?;
        }
        Ok(())
    }

    /// 查询远端文件是否存在、大小、修改时间（HEAD）
    pub fn stat(&self, path: &str) -> Result<RemoteStat, String> {
        let resp = self
            .client
            .head(self.url(path))
            .header("Authorization", self.auth())
            .send()
            .map_err(|e| format!("网络请求失败：{}", e))?;

        match resp.status() {
            StatusCode::OK => {
                let h = |k: reqwest::header::HeaderName| {
                    resp.headers()
                        .get(k)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string()
                };
                let size = h(reqwest::header::CONTENT_LENGTH)
                    .parse::<u64>()
                    .unwrap_or(0);
                Ok(RemoteStat {
                    exists: true,
                    size,
                    last_modified: h(reqwest::header::LAST_MODIFIED),
                    etag: h(reqwest::header::ETAG),
                })
            }
            StatusCode::NOT_FOUND => Ok(RemoteStat {
                exists: false,
                size: 0,
                last_modified: String::new(),
                etag: String::new(),
            }),
            StatusCode::UNAUTHORIZED => Err("鉴权失败（401）".into()),
            s => Err(format!("查询远端文件失败：{}", s)),
        }
    }

    /// 列出目录下的条目（PROPFIND Depth:1）。
    /// 坚果云返回标准 WebDAV XML，这里做轻量解析，不引入 XML 依赖：
    /// 按 <d:response> 切块，再从每块中提取 href / getcontentlength / getlastmodified。
    pub fn list_dir(&self, dir: &str) -> Result<Vec<RemoteEntry>, String> {
        let dir_norm = format!("{}/", dir.trim_matches('/'));
        let resp = self
            .client
            .request(
                reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
                self.url(&dir_norm),
            )
            .header("Authorization", self.auth())
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(
                r#"<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop>
<d:resourcetype/><d:getcontentlength/><d:getlastmodified/></d:prop></d:propfind>"#,
            )
            .send()
            .map_err(|e| format!("列目录失败：{}", e))?;

        match resp.status() {
            StatusCode::MULTI_STATUS | StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Ok(Vec::new()),
            StatusCode::UNAUTHORIZED => return Err("鉴权失败（401）".into()),
            s => return Err(format!("列目录失败：{}", s)),
        }

        let xml = resp.text().map_err(|e| e.to_string())?;
        let mut out = Vec::new();

        for block in xml.split("response>").skip(1) {
            let href = match extract_tag(block, "href") {
                Some(h) => h,
                None => continue,
            };
            // href 是 URL 编码的绝对路径，取最后一段作为名字
            let trimmed = href.trim_end_matches('/');
            let name_raw = trimmed.rsplit('/').next().unwrap_or("");
            if name_raw.is_empty() {
                continue;
            }
            let name = percent_decode(name_raw);
            let is_dir = href.ends_with('/') || block.contains("collection");
            // 目录自身也会出现在结果中，按名字与请求目录相同则跳过
            let req_name = dir.trim_matches('/').rsplit('/').next().unwrap_or("");
            if is_dir && name == req_name {
                continue;
            }
            out.push(RemoteEntry {
                name,
                is_dir,
                size: extract_tag(block, "getcontentlength")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0),
                last_modified: extract_tag(block, "getlastmodified").unwrap_or_default(),
            });
        }
        Ok(out)
    }

    /// 下载文件内容为字符串。文件不存在返回 Ok(None)。
    pub fn get_text(&self, path: &str) -> Result<Option<String>, String> {
        let resp = self
            .client
            .get(self.url(path))
            .header("Authorization", self.auth())
            .send()
            .map_err(|e| format!("下载失败：{}", e))?;

        match resp.status() {
            StatusCode::OK => resp
                .text()
                .map(Some)
                .map_err(|e| format!("读取响应失败：{}", e)),
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => Err("鉴权失败（401）".into()),
            s => Err(format!("下载失败：{}", s)),
        }
    }

    /// 上传文本内容，覆盖同名文件。会自动创建父目录。
    pub fn put_text(&self, path: &str, body: String) -> Result<(), String> {
        if let Some(idx) = path.trim_matches('/').rfind('/') {
            let dir = &path.trim_matches('/')[..idx];
            self.ensure_dir(dir)?;
        }

        let resp = self
            .client
            .put(self.url(path))
            .header("Authorization", self.auth())
            .header("Content-Type", "application/json; charset=utf-8")
            .body(body)
            .send()
            .map_err(|e| format!("上传失败：{}", e))?;

        match resp.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => Err("鉴权失败（401）".into()),
            StatusCode::INSUFFICIENT_STORAGE => {
                Err("空间或流量不足（507）：请检查坚果云本月流量额度".into())
            }
            s => Err(format!("上传失败：{}", s)),
        }
    }

    /// 删除远端文件（供调试/清理用）
    pub fn delete(&self, path: &str) -> Result<(), String> {
        let resp = self
            .client
            .delete(self.url(path))
            .header("Authorization", self.auth())
            .send()
            .map_err(|e| format!("删除失败：{}", e))?;
        match resp.status() {
            StatusCode::OK | StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => Ok(()),
            s => Err(format!("删除失败：{}", s)),
        }
    }
}

/// 从 XML 片段里取某个标签的文本，忽略命名空间前缀（如 d:href / D:href / dav:href）。
/// 注意闭合标签形如 `</d:href>`，不能直接搜 `</href`（会因中间的 `d:` 前缀匹配不上），
/// 所以要逐段找 `</`，再去掉前缀后与标签名比对。
fn extract_tag(block: &str, tag: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(rel) = block[search_from..].find("</") {
        let close_start = search_from + rel;
        let after = &block[close_start + 2..];
        // 标签名由字母数字、冒号、连字符组成，遇到其它字符（通常是 '>'）即为结束
        let name_end = after
            .find(|c: char| !c.is_ascii_alphanumeric() && c != ':' && c != '-')
            .unwrap_or(after.len());
        let tag_name = &after[..name_end];
        if tag_name.ends_with(tag) {
            // 从闭合标签位置往前找最近的 '>'，即开标签的结束位置
            let open_end = block[..close_start].rfind('>')?;
            return Some(block[open_end + 1..close_start].trim().to_string());
        }
        search_from = close_start + 2;
    }
    None
}

/// 极简 percent-decode，只处理 %XX。文件名里主要是中文与常见符号，够用。
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}
