use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use reqwest::blocking::Response;
use std::error::Error;

fn read_response_body(resp: Response) -> Result<(Response, Vec<u8>), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    let mut body = Vec::new();

    thread::spawn(move || {
        let mut buf = Vec::new();
        let mut response = resp;
        response.read_to_end(&mut buf).unwrap();
        tx.send((response, buf)).unwrap();
    });

    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok((response, buf)) => {
            body = buf;
            Ok((response, body))
        }
        Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Request timed out"))),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use warp::Filter;

    #[test]
    fn test_client_url() {
        let tests = vec![
            ("http://localhost:9090", "/test", None, "http://localhost:9090/test"),
            ("http://localhost", "/test", None, "http://localhost/test"),
            ("http://localhost:9090", "test", None, "http://localhost:9090/test"),
            ("http://localhost:9090/prefix", "/test", None, "http://localhost:9090/prefix/test"),
            ("https://localhost:9090/", "/test/", None, "https://localhost:9090/test"),
            ("http://localhost:9090", "/test/:param", Some(vec![("param", "content")]), "http://localhost:9090/test/content"),
            ("http://localhost:9090", "/test/:param/more/:param", Some(vec![("param", "content")]), "http://localhost:9090/test/content/more/content"),
            ("http://localhost:9090", "/test/:param/more/:foo", Some(vec![("param", "content"), ("foo", "bar")]), "http://localhost:9090/test/content/more/bar"),
            ("http://localhost:9090", "/test/:param", Some(vec![("nonexistent", "content")]), "http://localhost:9090/test/:param"),
        ];

        for (address, endpoint, args, expected) in tests {
            let mut url = address.to_string() + endpoint;
            if let Some(args) = args {
                for (key, value) in args {
                    url = url.replace(&format!(":{}", key), value);
                }
            }
            assert_eq!(url, expected);
        }
    }

    #[tokio::test]
    async fn benchmark_client() {
        let sizes_kb = vec![4, 50, 1000, 2000];
        for size_kb in sizes_kb {
            let size = size_kb * 1024;
            let route = warp::path::end().map(move || {
                let spaces = vec![b' '; size];
                warp::reply::with_header(spaces, "Content-Type", "text/plain")
            });

            let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
            let server = Arc::new(Mutex::new(Some(server)));
            let server_clone = Arc::clone(&server);

            let handle = spawn(move || {
                let server = server_clone.lock().unwrap().take().unwrap();
                server.run();
            });

            let client = Client::new();
            let url = format!("http://{}/prometheus/api/v1/query?query=up", addr);

            for _ in 0..100 {
                let resp = client.get(&url).send().unwrap();
                let (_, body) = read_response_body(resp).unwrap();
                assert_eq!(body.len(), size);
            }

            handle.join().unwrap();
        }
    }
}