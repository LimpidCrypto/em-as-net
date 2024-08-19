use alloc::{
    dbg, format,
    string::{String, ToString},
};
use anyhow::Result;
use thiserror_no_std::Error;
use url::Host;

#[cfg(feature = "dns")]
pub mod dns;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "tls")]
pub mod tls;

#[derive(Debug, Clone)]
pub struct Url(url::Url);

impl Url {
    pub fn parse(url: &str) -> Result<Self> {
        let mut url =
            Url(url::Url::parse(url).map_err(|e| UrlException::InvalidUrl(e.to_string()).into())?);
        url.assure_port().unwrap();
        dbg!(url.clone());
        Ok(url)
    }

    pub fn for_tcp(&self) -> Result<String> {
        let mut url = self.clone();
        url.assure_port().unwrap();
        url.set_port(Some(443)).unwrap();
        let host = match self.host_str() {
            Some(host) => host,
            None => return Err(anyhow::anyhow!("Host not found")),
        };
        let port = match self.port() {
            Some(port) => port,
            None => return Err(anyhow::anyhow!("Port not found")),
        };
        let path = self.path();
        let url = host.to_string() + ":" + &port.to_string() + path;

        Ok(url)
    }

    pub fn scheme(&self) -> &str {
        self.0.scheme()
    }

    pub fn host(&self) -> Option<Host<&str>> {
        self.0.host()
    }

    pub fn host_str(&self) -> Option<&str> {
        self.0.host_str()
    }

    pub fn port(&self) -> Option<u16> {
        self.0.port()
    }

    pub fn path(&self) -> &str {
        self.0.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.0.query()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.0.fragment()
    }

    pub fn set_scheme(&mut self, scheme: &str) -> Result<(), ()> {
        self.0.set_scheme(scheme)
    }

    pub fn set_port(&mut self, port: Option<u16>) -> Result<(), ()> {
        self.0.set_port(port)
    }

    pub fn set_path(&mut self, path: &str) -> () {
        self.0.set_path(path)
    }

    pub fn set_query(&mut self, query: &str) -> () {
        self.0.set_query(Some(query))
    }

    pub fn get_with_port(&self, default: Option<u16>) -> String {
        let scheme = self.0.scheme();
        let host = match self.0.host_str() {
            Some(host) => host,
            None => return "".to_string(),
        };
        let port = match self.0.port() {
            Some(port) => port,
            None => match scheme {
                "wss" | "https" => 443,
                "ws" | "http" => 80,
                _ => default.unwrap_or(80),
            },
        };
        let path = self.path();
        // let query = self.query().map_or("".into(), |q| format!("?{}", q));
        // let fragment = self.fragment().map_or("".into(), |f| format!("#{}", f));

        format!(
            "{}://{}:{}{}",
            scheme,
            host,
            port,
            path,
            // query,
            // fragment
        )
    }

    fn assure_port(&mut self) -> Result<(), ()> {
        if self.0.port().is_none() {
            let port = match self.0.scheme() {
                "wss" | "https" => 443,
                "ws" | "http" => 80,
                _ => 80,
            };
            self.0.set_port(Some(port))?;
        }

        Ok(())
    }
}

impl TryFrom<url::Url> for Url {
    type Error = ();

    fn try_from(url: url::Url) -> Result<Self, ()> {
        let mut url = Url(url);
        url.assure_port()?;
        Ok(url)
    }
}

impl From<Url> for url::Url {
    fn from(url: Url) -> url::Url {
        url.0
    }
}

impl TryFrom<&str> for Url {
    type Error = anyhow::Error;

    fn try_from(url: &str) -> Result<Self> {
        let url =
            Url(url::Url::parse(url).map_err(|e| UrlException::InvalidUrl(e.to_string()).into())?);

        Ok(url)
    }
}

impl ToString for Url {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<Url> for String {
    fn from(url: Url) -> Self {
        url.to_string()
    }
}

#[derive(Debug, Error)]
pub enum UrlException {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Invalid scheme: {0}")]
    InvalidScheme(String),
}

impl From<url::ParseError> for UrlException {
    fn from(e: url::ParseError) -> Self {
        UrlException::InvalidUrl(e.to_string())
    }
}

impl Into<anyhow::Error> for UrlException {
    fn into(self) -> anyhow::Error {
        anyhow::anyhow!(self)
    }
}
