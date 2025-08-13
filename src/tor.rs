use rustls::{ClientConnection, Stream};
use std::sync::Arc;
use std::io::Write;
use std::io::Read;

pub struct TorStream {
    pub inner: std::net::TcpStream,
}

impl TorStream {
    pub fn new(stream: std::net::TcpStream) -> Self {
        Self {inner: stream}
    }
}

impl Read for TorStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let result = self.inner.read(buf);
        let plain_text = String::from_utf8_lossy(&buf[..]);
        if let Ok(n) = result {
            // println!("READ: {} bytes: {:02x?}", n, &buf[..n]);
            println!("READ: {} bytes:\n{}", n, plain_text);
        }
        result
    }
}

impl Write for TorStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = self.inner.write(buf);
        if let Ok(n) = result {
            // println!("WRITE: {} bytes: {:02x?}", n, &buf[..n]);
            println!("WRITE: [{}] bytes:\n{}", n, String::from_utf8_lossy(&buf[..n]));
        }
        result
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub fn bootstrap() {
    println!("Connecting to Tor directory server... ");
    let tcp_dir = std::net::TcpStream::connect("198.251.84.163:22")
        .expect("Failed to connect to the Tor directory server");

    let mut tor_stream = TorStream::new(tcp_dir);
    println!("Connected to Tor directory server");

    let request_version = b"version 1.0\r\n";
    tor_stream.write(request_version)
        .expect("Failed to write to Tor directory stream");

    let mut buf = Vec::new();
    tor_stream.read(&mut buf)
        .expect("Failed to read from Tor directory stream");

    let request = b"GET /tor/status-vote/current/consensus-microdesc.z HTTP/1.0";
    tor_stream.write(request)
        .expect("Failed to write to Tor directory stream");

    println!("Request sent, reading response...");
    tor_stream.read_to_end(&mut buf)
        .expect("Failed to read from Tor directory stream");

    println!("Response: {}", String::from_utf8_lossy(&buf));
}

pub struct Tor {
    pub inner: ClientConnection,
    pub stream: TorStream,
}

impl Tor {
    pub fn connect(uri: &str, port: &str) -> Self {
        let tcp = std::net::TcpStream::connect(format!("{}:{}", uri, port))
            .expect("Failed to connect to the Tor server");

        let mut tor_stream = TorStream::new(tcp);

        let root_store = rustls::RootCertStore::from_iter(
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .cloned(),
        );

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = uri
            .to_owned()
            .try_into()
            .expect("Invalid server name");

        let rc_config = Arc::new(config);

        let mut conn = ClientConnection::new(rc_config, server_name)
            .expect("Failed to create TLS connection");

        conn.complete_io(&mut tor_stream)
            .expect("Failed to complete TLS handshake");

        Self { inner: conn, stream: tor_stream }
    }

    pub fn stream(&mut self) -> Stream<ClientConnection, TorStream> {
        Stream::new(&mut self.inner, &mut self.stream)
    }

}

