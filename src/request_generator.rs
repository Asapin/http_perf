use crate::models::{
    common::{Header, NextAction},
    request::Request,
};

pub struct RequestGenerator {
    idx: usize,
}

impl RequestGenerator {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }

    pub fn new_request(&mut self, request_number: usize) -> NextAction {
        NextAction::SendRequest(Request::new("http://localhost:8080").get())
    }

    pub fn failed_to_send(&mut self) {
        println!("{}: failed to send", self.idx);
    }

    pub fn failed_request_body(&mut self, status: u16, content_length: u64, headers: Vec<Header>) {
        println!("{}: failed to receive body", self.idx);
    }

    pub fn consume_response(
        &mut self,
        status: u16,
        content_length: u64,
        headers: Vec<Header>,
        body: bytes::Bytes,
    ) {
    }
}
