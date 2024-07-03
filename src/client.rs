use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use prost::Message;
use bytes::BytesMut;
use clap::Parser;

mod message {
    include!(concat!(env!("OUT_DIR"), "/message.rs"));
}

use message::{Request, Response};

const PORT: u32 = 11111;
const BUFF: usize = 1024;
const IP: &str = "::1";

#[derive(Parser, Debug)]
#[command(name = "Client",
          version = "1.0",
          author = "Author Name <author@example.com>",
          about = "Server application")]
struct Args {
    /// Sets the port to listen on
    #[arg(short, long, value_name = "PORT")]
    port: Option<u32>,
    /// Sets the ip to listen on
    #[arg(short, long, value_name = "IP")]
    ip: Option<String>,    
}

struct MyTcpStream {
    stream: TcpStream,
}

macro_rules! exit_decode {
    ( $e:expr ) => {
            println!("Send, Error dec/encoding: {}", $e);
            std::process::exit(0);
    };
}

impl MyTcpStream {
    pub async fn csend<T: Message>(&mut self, r: &T) -> std::io::Result<()> {
        let mut buf = Vec::new();
        match r.encode(&mut buf) {
            Ok(_) => {},
            Err(e) => { exit_decode!(e); },
        }
        self.cwrite(&buf).await
    }
    pub async fn crecv<T: Message>(&mut self) -> Response {
        let r = &mut BytesMut::with_capacity(BUFF);
        match self.cread(r).await {
            Ok(_) => {},
            Err(e) => { exit_decode!(e); },
        }
        match Response::decode(r) {
            Ok(rr) => rr,
            Err(e) => { exit_decode!(e); },
        }
    }
    async fn cread(&mut self, r: &mut BytesMut) -> std::io::Result<()> {
        self.stream.read_buf(r).await.map(|_| ()).map_err(|e| { exit_decode!(e); })
    }
    async fn cwrite(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(buf).await.map(|_| ()).map_err(|e| { exit_decode!(e); })
    }
}
fn request_collect() -> Request {
    Request {
        req: "sent request".to_string(),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let port = args.port.unwrap_or(PORT);
    let ip = args.ip.unwrap_or(IP.to_string());
    let addr = format!("{}:{}", ip, port);

    let mut socket: MyTcpStream;
    match TcpStream::connect(addr.clone()).await {
        Ok(s) => { socket = MyTcpStream { stream: s }; },
        Err(e) => { exit_decode!(e); },
    }

    let request: Request = request_collect();
    match socket.csend(&request).await {
        Ok(_) => println!("Sent request: {:#?}", request),
        Err(e) => { exit_decode!(e); },
    }

    let response: Response = socket.crecv::<Response>().await;
    println!("Received response: {:#?}", response);
    Ok(())
}
