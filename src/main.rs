mod ip_iterator;
mod entry_alloc;

use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::os::fd::{AsRawFd, OwnedFd};
use std::time::Duration;
use craftping::Chat;
use io_uring::{IoUring, opcode, squeue};
use io_uring::types::{Fd, Timespec};
use nix::sys::socket::{AddressFamily, SockaddrIn, SockaddrLike, socket, SockFlag, SockType};
use crate::entry_alloc::EntryAllocator;
use crate::ip_iterator::IpIterator;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_CONNECTIONS: u32 = 2048;
const RING_BATCH_SIZE: usize = 32;

struct Connection {
    ip: SockaddrIn,
    sock: OwnedFd,
}

fn chat_to_str(chat: &Chat) -> String {
    fn recurse(str: &mut String, chat: &Chat) {
        str.push_str(&chat.text);
        chat.extra.iter().for_each(|e| recurse(str, e));
    }

    let mut str = String::with_capacity(chat.text.len());
    recurse(&mut str, &chat);
    return str;
}

fn process(conn: Connection) {
    let mut stream = TcpStream::from(conn.sock);
    stream.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();
    stream.set_write_timeout(Some(Duration::from_millis(1000))).unwrap();

    let ip = Ipv4Addr::from(conn.ip.ip());

    match craftping::sync::ping(&mut stream, &ip.to_string(), 25565) {
        Ok(resp) => {
            println!("{}: version: {}, online players: {}, desc: {}", conn.ip, resp.version, resp.online_players, chat_to_str(&resp.description));
        }
        Err(_) => {
            //eprintln!("failed to ping {}: {}", conn.ip, e)
        }
    }
}

fn main() -> std::io::Result<()> {
    let timeout = Timespec::from(CONNECT_TIMEOUT);

    let mut ring = IoUring::new(MAX_CONNECTIONS * 2)?;
    let mut connections = EntryAllocator::<Connection>::new(MAX_CONNECTIONS as usize);

    let mut ips = IpIterator::new();

    loop {
        while !connections.is_full() {
            let ip = ips.next().unwrap();

            let (idx, conn) = connections.alloc(Connection {
                ip: SockaddrIn::from(SocketAddrV4::new(ip, 25565)),
                sock: socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None)?,
            }).unwrap();

            let connect = opcode::Connect::new(Fd(conn.sock.as_raw_fd()), conn.ip.as_ptr(), conn.ip.len())
                .build()
                .flags(squeue::Flags::IO_LINK)
                .user_data(idx as u64);

            let timeout = opcode::LinkTimeout::new(&timeout)
                .build()
                .user_data(u64::MAX);

            unsafe { ring.submission().push_multiple(&[connect, timeout]).unwrap() }
        }

        ring.submit_and_wait(RING_BATCH_SIZE)?;

        for cq in ring.completion() {
            if cq.user_data() == u64::MAX { continue }
            let conn = connections.dealloc(cq.user_data() as usize).unwrap();
            if cq.result() == 0 {
                std::thread::spawn(move || process(conn));
            }
        }
    }
}
