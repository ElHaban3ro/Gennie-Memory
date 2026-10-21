//! Real HTTP body reads, with scaled durations and agent-owned ephemeral servers.
use super::*;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
fn trickle(header_delay: Duration) -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let worker = thread::spawn(move || {
        let accept_deadline = Instant::now() + Duration::from_secs(1);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if stopped.load(Ordering::SeqCst) || Instant::now() >= accept_deadline {
                        return;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("mock accept failed: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut request = [0u8; 4096];
        let _ = stream.read(&mut request);
        thread::sleep(header_delay);
        if stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n")
            .is_err()
        {
            return;
        }
        for _ in 0..8 {
            if stopped.load(Ordering::SeqCst) || stream.write_all(b" ").is_err() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        let _ = stream.write_all(b"{}");
    });
    (format!("http://{address}/"), stop, worker)
}
fn check_deadline(header_delay: Duration, timeout: Duration) {
    let (url, stop, worker) = trickle(header_delay);
    let classifier = OpenRouterClassifier {
        client: Client::builder()
            .timeout(timeout)
            .no_proxy()
            .build()
            .unwrap(),
        api_key: None,
        evaluation_capture: None,
        evaluation_response_content: None,
    };
    let start = Instant::now();
    let result = classifier.request_measured_at("synthetic", &json!({}), &url, timeout);
    let elapsed = start.elapsed();
    stop.store(true, Ordering::SeqCst);
    drop(classifier);
    worker.join().unwrap();
    match result {
        Err(e) => assert_eq!(e.code, "transport_error"),
        Ok(_) => panic!("trickled valid JSON escaped total deadline: {elapsed:?}"),
    }
    assert!(
        elapsed < timeout + Duration::from_millis(80),
        "total deadline {timeout:?} extended to {elapsed:?}"
    );
}
#[test]
fn product_request_deadline_covers_trickling_body() {
    check_deadline(Duration::ZERO, Duration::from_millis(100));
}

#[test]
fn product_deadline_includes_headers_and_body_in_one_budget() {
    check_deadline(Duration::from_millis(120), Duration::from_millis(200));
}

#[test]
fn evaluation_capture_records_only_hashes_and_transport_metadata() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let request_body = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_body = request_body.clone();
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut request = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let count = stream.read(&mut chunk).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..count]);
            let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
            else {
                continue;
            };
            let header_end = header_end + 4;
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap();
            if request.len() >= header_end + content_length {
                *captured_body.lock().unwrap() =
                    request[header_end..header_end + content_length].to_vec();
                break;
            }
        }
        let body = br#"{"model":"test/model","usage":{"prompt_tokens":11,"completion_tokens":2,"cost":0.001},"choices":[{"message":{"content":"{\"choice\":\"none\"}"}}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(body).unwrap();
    });

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let capture_path = std::env::temp_dir().join(format!("iromem-eval-capture-{nonce}.jsonl"));
    let response_path = std::env::temp_dir().join(format!("iromem-eval-response-{nonce}.jsonl"));
    let classifier = OpenRouterClassifier::new_with_evaluation_capture_and_content(
        None,
        Some(&capture_path),
        Some(&response_path),
    )
    .unwrap();
    let payload = json!({
        "model":"test/model",
        "messages":[
            {"role":"system","content":"synthetic prompt must not be stored"},
            {"role":"user","content":"{\"scope_id\":\"smoke-01\"}"}
        ],
        "response_format":{"type":"json_schema","json_schema":{"schema":{"type":"object","properties":{"choice":{"type":"string"}}}}}
    });
    classifier
        .request_measured_at(
            "synthetic-token-must-not-be-stored",
            &payload,
            &format!("http://{address}/"),
            Duration::from_secs(1),
        )
        .unwrap();
    worker.join().unwrap();

    let recorded_request = request_body.lock().unwrap().clone();
    let record: Value =
        serde_json::from_slice(fs::read_to_string(&capture_path).unwrap().trim().as_bytes())
            .unwrap();
    assert_eq!(
        record["request_payload_sha256"],
        iromem_kernel::digest::sha256_hex(&recorded_request)
    );
    assert_eq!(record["scope_id"], "smoke-01");
    assert_eq!(record["http_status"], 200);
    assert_eq!(record["served_model"], "test/model");
    assert_eq!(record["input_tokens"], 11);
    assert_eq!(record["output_tokens"], 2);
    assert_eq!(record["reported_cost_usd"], 0.001);
    assert_eq!(
        record["prompt_sha256"],
        iromem_kernel::digest::sha256_hex(&serde_json::to_vec(&payload["messages"]).unwrap())
    );
    assert_eq!(
        record["schema_sha256"],
        iromem_kernel::digest::sha256_hex(
            &serde_json::to_vec(&payload["response_format"]["json_schema"]["schema"]).unwrap()
        )
    );
    let response_body =
        br#"{"model":"test/model","usage":{"prompt_tokens":11,"completion_tokens":2,"cost":0.001},"choices":[{"message":{"content":"{\"choice\":\"none\"}"}}]}"#;
    assert_eq!(
        record["response_sha256"],
        iromem_kernel::digest::sha256_hex(response_body)
    );
    assert!(record["response_body_complete"].as_bool().unwrap());
    assert_eq!(
        fs::metadata(&capture_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let contents = fs::read_to_string(&capture_path).unwrap();
    assert!(!contents.contains("synthetic prompt"));
    assert!(!contents.contains("synthetic-token"));
    let response_record: Value = serde_json::from_slice(
        fs::read_to_string(&response_path)
            .unwrap()
            .trim()
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(response_record["response_content"], "{\"choice\":\"none\"}");
    assert_eq!(
        response_record["request_payload_sha256"],
        record["request_payload_sha256"]
    );
    assert_eq!(
        fs::metadata(&response_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    drop(classifier);
    assert!(OpenRouterClassifier::new_with_evaluation_capture(None, Some(&capture_path)).is_err());
    fs::remove_file(capture_path).unwrap();
    fs::remove_file(response_path).unwrap();
}

#[cfg(feature = "http2-transport")]
#[test]
fn product_http2_deadline_covers_trickling_body() {
    use std::sync::atomic::AtomicUsize;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let sent = Arc::new(AtomicUsize::new(0));
    let sent_server = sent.clone();
    let accepted = Arc::new(AtomicBool::new(false));
    let accepted_server = accepted.clone();
    let worker = thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut sender = None;
        rt.block_on(async {
            let tcp = tokio::net::TcpListener::from_std(listener).unwrap();
            tokio::time::timeout(Duration::from_secs(1), async {
                let (socket, _) = tcp.accept().await.unwrap();
                let mut conn = h2::server::handshake(socket).await.unwrap();
                if let Some(Ok((request, mut respond))) = conn.accept().await {
                    // Keep the incoming body alive while accept drives both HTTP/2 directions.
                    let _request = request;
                    let mut stream = respond.send_response(Default::default(), false).unwrap();
                    accepted_server.store(true, Ordering::SeqCst);
                    let stop_driver = stopped.clone();
                    sender = Some(thread::spawn(move || {
                        for _ in 0..8 {
                            if stopped.load(Ordering::SeqCst)
                                || stream.send_data((&b" "[..]).into(), false).is_err()
                            {
                                return;
                            }
                            sent_server.fetch_add(1, Ordering::SeqCst);
                            thread::sleep(Duration::from_millis(50));
                        }
                        let _ = stream.send_data((&b"{}"[..]).into(), true);
                    }));
                    loop {
                        if stop_driver.load(Ordering::SeqCst) {
                            conn.graceful_shutdown();
                            break;
                        }
                        match tokio::time::timeout(Duration::from_millis(10), conn.accept()).await {
                            Ok(None) => break,
                            Ok(Some(_)) | Err(_) => {}
                        }
                    }
                }
            })
            .await
            .expect("mock HTTP/2 connection must terminate");
        });
        if let Some(sender) = sender {
            sender.join().unwrap();
        }
    });
    let timeout = Duration::from_millis(200);
    let classifier = OpenRouterClassifier {
        client: Client::builder()
            .timeout(timeout)
            .http2_prior_knowledge()
            .no_proxy()
            .build()
            .unwrap(),
        api_key: None,
        evaluation_capture: None,
        evaluation_response_content: None,
    };
    let start = Instant::now();
    let result = classifier.request_measured_at(
        "synthetic",
        &json!({}),
        &format!("http://{address}/"),
        timeout,
    );
    let elapsed = start.elapsed();
    stop.store(true, Ordering::SeqCst);
    drop(classifier);
    worker.join().unwrap();
    assert!(
        accepted.load(Ordering::SeqCst),
        "must reach negotiated HTTP/2 response"
    );
    assert!(
        sent.load(Ordering::SeqCst) >= 2,
        "must receive a trickling body, not only headers"
    );
    match result {
        Err(e) => assert_eq!(e.code, "transport_error"),
        Ok(_) => panic!("HTTP/2 trickle escaped total deadline: {elapsed:?}"),
    }
    assert!(
        elapsed < timeout + Duration::from_millis(80),
        "HTTP/2 total deadline extended to {elapsed:?}"
    );
}

#[test]
fn unused_http1_mock_stops_without_a_connection() {
    let (_url, stop, worker) = trickle(Duration::ZERO);
    stop.store(true, Ordering::SeqCst);
    worker.join().unwrap();
}
