use smol::{net, prelude::*};
use bytes::{BytesMut, BufMut};

/**************************************************************************/
/**************************************************************************/

const MAGIC:u8 = 0x7f;
const MAX_BLOCK_SIZE:u8 = 0x30; // 2^12 bytes = 4k
const SERIALIZER:u8 = 0x01; // JSON serializer

const RAWSOCKET_MESSAGE_TYPE_REGULAR:u8 = 0;

// Connect and say hello to the server. We need to do the
// handshake here.
const HANDSHAKE:[u8;4] = [
                            MAGIC, // Flags to crossbar that we're speaking the same language
                            MAX_BLOCK_SIZE | SERIALIZER,
                            0, 0,
                        ]; 

#[derive(Debug, Clone)]
pub struct Transport {
    stream: net::TcpStream,
}

impl Transport {

    pub async fn message_send(&mut self, buf:Vec<u8>) {
        let mut message_length_buf = BytesMut::with_capacity(10);
        message_length_buf.put_u8(RAWSOCKET_MESSAGE_TYPE_REGULAR);
        message_length_buf.put_u8(0);
        message_length_buf.put_u16(buf.len().try_into().unwrap());

        println!("Queueing data: {:?}", message_length_buf);
        self.stream.write(&message_length_buf.to_vec()).await;
        println!("Queueing data: {:?}", buf);
        self.stream.write(&buf).await;
    }

    pub async fn message_get(&mut self) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 4096];

        let read_bytes = self.stream.read(&mut buf).await;

        // FIXME: need to keep collecting data or dealing with chunked
        // data or a burst with multiple packets at the same time?
        if buf.len() < 4 {
            return None;
        }

        Some(
            buf[4..read_bytes.unwrap()].to_vec()
        )
    }

    pub async fn negotiate(&mut self) {
        let mut buf = vec![0u8; 4096];

        println!("Attempting handshake");

        // Perform the handshake

        // We start things off by doing the raw socket handshake with nexus
        // which determines if this is a nexus server, the protocol to use
        // and so on
        self.stream.write(&HANDSHAKE).await;

        // Let's get the server's response
        // FIXME: handle errors properly
        let read_bytes = self.stream.read(&mut buf).await.unwrap();
        if read_bytes != 4 {
            panic!("Did not get 4 bytes!")
        }
        if buf[0] != MAGIC {
            panic!("Did not get MAGIC")
        }

        let server_serializer = buf[1] & 0x0f;
        if server_serializer != SERIALIZER {
            panic!("Server did not agree to use JSON")
        }
        let server_buffer_size = buf[1] >> 4;
        println!("Server buffer size is: {}", server_buffer_size);
    }

    pub fn connect( url:&str ) -> Transport {
        let stream = smol::block_on(async {
                            net::TcpStream::connect(url).await.unwrap()
                        });
        let mut transport = Transport { stream };

        smol::block_on(async {
            transport.negotiate().await;
        });

        transport
    }
}



