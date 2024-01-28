use reqwest::{Client, RequestBuilder};

use super::common::{Header, Verb};

pub struct Request {
    url: String,
    verb: Verb,
    headers: Vec<Header>,
}

impl Request {
    pub fn new<U: ToString>(url: U) -> Self {
        Self {
            url: url.to_string(),
            verb: Verb::Get,
            headers: vec![],
        }
    }

    pub fn get(mut self) -> Self {
        self.verb = Verb::Get;
        self
    }

    pub fn head(mut self) -> Self {
        self.verb = Verb::Head;
        self
    }

    pub fn post(mut self, body: Option<Vec<u8>>) -> Self {
        self.verb = Verb::Post(body);
        self
    }

    pub fn put(mut self, body: Option<Vec<u8>>) -> Self {
        self.verb = Verb::Put(body);
        self
    }

    pub fn delete(mut self) -> Self {
        self.verb = Verb::Delete;
        self
    }

    pub fn patch(mut self, body: Option<Vec<u8>>) -> Self {
        self.verb = Verb::Patch(body);
        self
    }

    pub fn header(mut self, name: String, value: Vec<u8>) -> Self {
        let header = Header::new(name, value);
        self.headers.push(header);
        self
    }

    pub fn build(self, http_client: &Client) -> RequestBuilder {
        let url = self.url;
        let mut request = match self.verb {
            Verb::Get => http_client.get(url),
            Verb::Head => http_client.head(url),
            Verb::Post(body) => {
                let mut req = http_client.post(url);
                if let Some(data) = body {
                    req = req.body(data);
                }
                req
            }
            Verb::Put(body) => {
                let mut req = http_client.put(url);
                if let Some(data) = body {
                    req = req.body(data);
                }
                req
            }
            Verb::Delete => http_client.delete(url),
            Verb::Patch(body) => {
                let mut req = http_client.patch(url);
                if let Some(data) = body {
                    req = req.body(data);
                }
                req
            }
        };

        for header in self.headers {
            request = request.header(header.name, header.value);
        }

        request
    }
}
