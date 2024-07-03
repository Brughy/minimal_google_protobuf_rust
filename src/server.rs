use tokio::net::{TcpListener, TcpStream}; 
use tokio::io::{AsyncReadExt, AsyncWriteExt}; 
use prost::{EncodeError, Message}; 
use bytes::BytesMut; 
use clap::Parser; 
use std::error::Error; 

mod message {   
    include!(concat!(env!("OUT_DIR"), "/message.rs"));
}

use message::{Request, Response};

const PORT: u32 = 11111;
const BUFF: usize = 1024;
const IP: &str = "::1";

#[derive(Parser, Debug)] #[command(name = "Client",
    version = "1.0",
    author = "Author Name author@example.com",
    about = "Server application")]

struct Args {
    /// Sets the port to listen on
     #[arg(short, long, value_name = "PORT")]
    port: Option<u32>,
}

struct MyTcpListener {
    stream: TcpListener,
}

macro_rules! exit_decode {
    ( $e:expr ) => {
        println!("Error: {}", $e);
        //std::process::exit(0);
    };
}

struct MyTcpStream {
    stream: TcpStream,
}

impl MyTcpStream {
    pub async fn ssend<T: Message>(&mut self, r: &T) -> Result<() , Box<dyn Error + Send + Sync>> {
        let mut buf = Vec::new();
        let _r = match r.encode(&mut buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::<EncodeError>::new(e.into())),
        }?;
        match self.swrite(&buf).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    pub async fn srecv<T: Message>(&mut self) -> Result<Option<Request>, Box<dyn Error + Send + Sync>> {
        let mut r = BytesMut::with_capacity(BUFF);
        match self.sread(&mut r).await {
            Ok(_) => None,
            Err(e) => Some(Err::<T, Box<dyn std::error::Error + Send + Sync>>(e)),

        };
        match Request::decode(r) {
            Ok(rr) => Ok(Some(rr)),
            Err(e) => Err(Box::new(e)),
        }
    }
    async fn sread(&mut self, r: &mut BytesMut) -> Result<() , Box<dyn Error + Send + Sync>> {
        match self.stream.read_buf(r).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
    async fn swrite(&mut self, buf: &[u8]) -> Result<() , Box<dyn Error + Send + Sync>> {
        match self.stream.write_all(buf).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

fn response_collect() -> Response {
    Response {
        rsp: "received request".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();
    let port = args.port.unwrap_or(PORT);
    let addr = format!("{}:{}", IP, port);

    let listener = MyTcpListener {
        stream: tokio::net::TcpListener::bind(addr.clone()).await?
    };
    println!("Starting server at {:#?}", addr.clone());

    loop {
        let (s, _) = listener.stream.accept().await.unwrap();
	
        tokio::spawn( async move {
           
            let mut socket = MyTcpStream { stream: s };
            let request_result = socket.srecv::<Request>().await;

            match request_result {
                Ok(Some(rr)) => {
                    println!("Received request: {:#?}", rr);
                    let response = response_collect();
                    let send_result = socket.ssend(&response).await;

                    match send_result {
                        Ok(_) => println!("Sent response: {:#?}", response),
                        Err(e) => {exit_decode!(e);},
                    }
                },
                Ok(None) => println!("Received empty request"),
                Err(e) => println!("Error receiving request: {:?}", e),
            }
        } );
    }
}
