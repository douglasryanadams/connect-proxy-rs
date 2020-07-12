use futures::future::try_join;

use log::{info, debug, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

// const CRLF: [u8; 2] = [13, 10];
const SPACE: u8 = 32;
const COLON: u8 = 58;
//                               C   O   N   N   E   C   T  " "
const CONNECT_BYTES: [u8; 8] = [67, 79, 78, 78, 69, 67, 84, 32];

const BUFFER_SIZE: usize = 1024;
const UNSUPPORTED_METHOD: &str = "HTTP/1.1 405 Method Not Supported\r\n\
                                  Content-Type: text/plain;charset=utf-8\r\n\
                                  Content-Length: 26\r\n\
                                  \r\n\
                                  Only CONNECT Supported\r\n\r\n";
const CONNECT_SUCCESS: &str = "HTTP/1.1 200 Connection Established\r\n\
                               Proxy-agent: connect-proxy-rs/0.1.0\r\n\
                               \r\n";

/// This method blindly passes bytes from open stream to the other
/// Copied from Tokio Example: https://github.com/tokio-rs/tokio/blob/master/examples/proxy.rs
async fn pipe_data(mut listener: TcpStream, mut target: TcpStream) {
    let (mut listener_in, mut listener_out) = listener.split();
    let (mut target_in, mut target_out) = target.split();

    let client_to_server = io::copy(&mut listener_in, &mut target_out);
    let server_to_client = io::copy(&mut target_in, &mut listener_out);

    match try_join(client_to_server, server_to_client).await {
        Ok(_) => return,
        Err(e) => {
            warn!("Error joining pipe_data streams; error = {:?}", e);
            return;
        }
    };
}

async fn handle_connection(mut listener_socket: TcpStream) {
    let (mut listener_in, mut listener_out) = listener_socket.split();
    let mut listener_buffer = [0; BUFFER_SIZE];
    let mut target_domain = Vec::new();
    let mut target_port = Vec::new();
    let mut domain_complete = false;
    'connect_loop: loop {
        let bytes_read = match listener_in.read(&mut listener_buffer).await {
            Ok(br) => br,
            Err(e) => {
                warn!("Error reading bytes; error = {:?}", e);
                return;
            }
        };
        debug!("  read: {}", bytes_read);

        // Crude implementation that doesn't bother checking most of the HTTP spec (like headers
        // for example) in the favor of simplicity and speed
        let mut counter = 0;
        while counter < bytes_read {
            // debug!(" >: {:?}", listener_buffer[counter]);
            if counter < 8 { // Look for CONNECT at start of buffer
                if listener_buffer[counter] != CONNECT_BYTES[counter] {
                    let bytes_written = match listener_out.write(UNSUPPORTED_METHOD.as_bytes()).await {
                        Ok(bw) => bw,
                        Err(e) => {
                            warn!("Error writing out UNSUPPORTED_METHOD; error = {:?}", e);
                            return;
                        }
                    };
                    debug!("  bytes written: {:?}", bytes_written);
                    return;
                }
                counter += 1;
                continue;
            }

            if listener_buffer[counter] == SPACE {
                break 'connect_loop;
            }

            if listener_buffer[counter] == COLON {
                domain_complete = true;
                counter += 1;
                continue;
            }

            if domain_complete {
                &target_port.push(listener_buffer[counter]);
            } else {
                &target_domain.push(listener_buffer[counter]);
            }
            counter += 1;
        }
    }

    let target_domain = match std::str::from_utf8(&target_domain) {
        Ok(s) => s,
        Err(e) => {
            warn!("Error parsing target_domain; error = {:?}", e);
            return;
        }
    };

    let target_port = match std::str::from_utf8(&target_port) {
        Ok(s) => s,
        Err(e) => {
            warn!("Error parsing target_port; error = {:?}", e);
            return;
        }
    };
    debug!("  target_domain = {:?}", target_domain);
    debug!("  target_port = {:?}", target_port);


    // If it is, extract the domain name
    // do DNS resolution

    // Connect TCP socket to target domain/port
    let target_str = format!("{}:{}", target_domain, target_port);
    let target_str_copy = target_str.clone();
    let target_socket = match TcpStream::connect(target_str).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Error connecting to target ({}); error = {:?}", target_str_copy, e);
            return;
        }
    };
    let bytes_written = match listener_out.write(CONNECT_SUCCESS.as_bytes()).await {
        Ok(bw) => bw,
        Err(e) => {
            warn!("Error writing CONNECT_SUCCESS response; error = {:?}", e);
            return;
        }
    };
    debug!("  bytes written: {:?}", bytes_written);

    // start blindly passing packets
    tokio::spawn(pipe_data(listener_socket, target_socket));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("Listening on [127.0.0.1:8080]");

    while let Ok((listener_socket, _)) = listener.accept().await {
        tokio::spawn(handle_connection(listener_socket));
    }

    info!("Stopped listening on [127.0.0.1:8080]");
    Ok(())
}
