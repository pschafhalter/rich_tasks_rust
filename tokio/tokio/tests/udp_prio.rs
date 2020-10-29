#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use futures::future::poll_fn;
use std::sync::Arc;
use tokio::{io::ReadBuf, net::UdpSocket};

const MSG: &[u8] = b"hello";
const MSG_LEN: usize = MSG.len();


#[tokio::test]
async fn split() -> std::io::Result<()> {
    println! ("hi");
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    let s = Arc::new(socket);
    let r = s.clone();

    let addr = s.local_addr()?;
    tokio::spawn_with_priority(async move {
        s.send_to(MSG, &addr).await.unwrap();
    });
    let mut recv_buf = [0u8; 32];
    let (len, _) = r.recv_from(&mut recv_buf[..]).await?;
    assert_eq!(&recv_buf[..len], MSG);
    Ok(())
}

