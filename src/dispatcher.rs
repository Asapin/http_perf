use std::{sync::Arc, time::Duration};

use reqwest::Client;
use tokio::time::Instant;

use crate::{
    models::common::{BenchResult, Header, NextAction},
    request_generator::RequestGenerator,
    Config,
};

pub struct Dispatcher {
    number_of_requests: usize,
    generators: Vec<RequestGenerator>,
    http_client: Client,
}

impl Dispatcher {
    pub fn new<F>(config: &Config, generator_provider: F) -> Self
    where
        F: Fn(usize) -> RequestGenerator,
    {
        let mut generators = Vec::with_capacity(config.total_number_of_connections);
        for i in 0..config.total_number_of_connections {
            let generator = generator_provider(i);
            generators.push(generator);
        }
        let http_client = reqwest::Client::builder()
            .tcp_keepalive(Duration::from_secs(120))
            .build()
            .expect("Couldn't construct reqwest client");
        Self {
            number_of_requests: config.number_of_requests,
            generators,
            http_client,
        }
    }

    pub async fn run(self) {
        if self.generators.len() == 0 {
            return;
        }
        let requests_per_client = self.number_of_requests / self.generators.len();
        let mut remainder = self.number_of_requests % self.generators.len();
        let mut futures = Vec::with_capacity(self.generators.len());
        let http_client = Arc::new(self.http_client);
        for generator in self.generators {
            let mut number_of_requests = requests_per_client;
            if remainder > 0 {
                remainder -= 1;
                number_of_requests += 1;
            }
            let http_client_clone = http_client.clone();
            let future = tokio::spawn(async move {
                Dispatcher::run_generator(http_client_clone, number_of_requests, generator).await
            });
            futures.push(future);
        }

        let mut bench_result = BenchResult::new(self.number_of_requests);
        for future in futures {
            let future_result = future.await;
            match future_result {
                Ok(r) => {
                    bench_result = bench_result.merge(&r);
                    // TODO: notify monitoring system that the generator had finished its work
                }
                Err(e) => {
                    println!("Error occured while waiting for future to finish: {}", e);
                    // TODO: notify monitoring system that the future had failed
                }
            }
        }
        bench_result.print_stats();
    }

    async fn run_generator(
        http_client: Arc<Client>,
        number_of_requests: usize,
        mut generator: RequestGenerator,
    ) -> BenchResult {
        let mut bench_results = BenchResult::new(number_of_requests);
        for i in 0..number_of_requests {
            let start_time = Instant::now();
            let next_action = generator.new_request(i);
            let new_request = match next_action {
                NextAction::SendRequest(request) => request,
                NextAction::Stop => {
                    // TODO: notify monitoring system that generator has stopped sending requests
                    break;
                }
            };
            let builder = new_request.build(&http_client);
            let response_result = builder.send().await;
            let response = match response_result {
                Ok(response) => response,
                Err(e) => {
                    // TODO: send failed response error to monitoring system
                    bench_results.count_errors();
                    generator.failed_to_send();
                    continue;
                }
            };
            let status = response.status().as_u16();
            bench_results.count_response_status(status);
            let content_length = response.content_length().unwrap_or(0);
            bench_results.count_data(content_length);
            let mut headers = Vec::new();
            for (header_name, header_value) in response.headers() {
                let name = header_name.to_string();
                let value = header_value.as_bytes().to_vec();
                let header = Header::new(name, value);
                headers.push(header);
            }
            let body = match response.bytes().await {
                Ok(bytes) => {
                    bench_results.register_time(start_time);
                    bytes
                }
                Err(e) => {
                    // TODO: send body error to monitoring system
                    bench_results.register_time(start_time);
                    bench_results.count_errors();
                    generator.failed_request_body(status, content_length, headers);
                    continue;
                }
            };
            // TODO: track request duration, status and length
            generator.consume_response(status, content_length, headers, body);
        }

        bench_results
    }
}
