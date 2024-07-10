use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn send_successful_to_socket(socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("Could not connect to server");

    let message = "FN_OVERRIDE_SUCCESSFUL";

    // Send the message to the server
    stream
        .write_all(message.as_bytes())
        .await
        .expect("Failed to write to stream");
}

pub async fn send_failed_to_socket(socket_path: &str) {
    // Connect to the Unix socket
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("Could not connect to server");

    let message = "FN_OVERRIDE_FAILED";

    // Send the message to the server
    stream
        .write_all(message.as_bytes())
        .await
        .expect("Failed to write to stream");
}

// Function to handle a single client connection
pub async fn handle_client(mut stream: UnixStream, buffer_sender: async_channel::Sender<String>) {
    // Buffer to store incoming data
    let mut buffer = [0; 1024];

    // Read data from the stream
    match stream.read(&mut buffer).await {
        Ok(size) => {
            // Print the received message
            buffer_sender
                .send_blocking(String::from_utf8_lossy(&buffer[..size]).to_string())
                .expect("Buffer channel closed")
        }
        Err(e) => {
            // Print error message if reading fails
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
