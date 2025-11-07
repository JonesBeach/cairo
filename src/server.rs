use std::{
    io::{ErrorKind, Result},
    sync::Arc,
};

use tokio::net::TcpListener;

use crate::{connection::Connection, Router};

/// Serve incoming TCP connections using the provided `Router`.
///
/// This function listens for incoming TCP connections on the given `TcpListener` and spawns a
/// tokio task to handle each connection concurrently. Each connection is parsed into a
/// `Request`, which is then routed using the `Router`. The resulting `Response` is sent back to
/// the client.
pub async fn serve(listener: TcpListener, router: Router) -> Result<()> {
    // We create an `Arc` so we can share the `Router` between threads.
    let router = Arc::new(router);

    loop {
        let (stream, _) = listener.accept().await?;
        let router = router.clone();

        tokio::spawn(async move {
            let mut conn = Connection::new(stream);
            let request = match conn.read_request().await {
                Ok(req) => req,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    println!("Client closed connection.");
                    return;
                }
                Err(e) => {
                    eprintln!("Error reading request: {e}");
                    return;
                }
            };

            let response = router.call(request);
            if let Err(e) = conn.send_response(response).await {
                eprintln!("Error sending response: {e}");
            }
        });
    }
}
