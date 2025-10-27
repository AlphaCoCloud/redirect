use sib::network::http::{
    server::{H1Config, HFactory},
    session::{HService, Session},
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

struct Server;

impl HService for Server {
    fn call<S: Session>(&mut self, session: &mut S) -> std::io::Result<()> {
        use core::fmt::Write;
        use sib::network::http::h1_session;

        let mut res: heapless::String<256> = heapless::String::new();

        if write!(
            res,
            "HTTP/1.1 301 Moved Permanently\r\n\
             Server: sib\r\n\
             Date: {}\r\n\
             Location: https://playpod.ir/\r\n\
             Content-Length: 0\r\n\
             Connection: close\r\n\
             \r\n",
            h1_session::CURRENT_DATE.load()
        )
        .is_err()
        {
            eprintln!("Failed to format response string");
            return Err(std::io::Error::other("Failed to format redirect response"));
        }

        session.write_all_eom(res.as_bytes())
    }
}

impl HFactory for Server {
    type Service = Server;

    fn service(&self, _id: usize) -> Server {
        Server
    }
}

fn main() {
    let stack_size = 2 * 1024; // 2 KB stack
    let cpus = num_cpus::get();

    sib::init_global_poller(cpus, stack_size);

    // Pick a port and start the server
    let addr = "0.0.0.0:8080";
    let mut threads = Vec::with_capacity(cpus);

    for _ in 0..cpus {
        let handle = std::thread::spawn(move || {
            let id = std::thread::current().id();
            println!("Listening {addr} on thread: {id:?}");
            Server
                .start_h1(
                    addr,
                    H1Config {
                        io_timeout: std::time::Duration::from_secs(15),
                        stack_size,
                    },
                )
                .unwrap_or_else(|_| panic!("H1 server failed to start for thread {id:?}"))
                .join()
                .unwrap_or_else(|_| panic!("H1 server failed to join thread {id:?}"));
        });
        threads.push(handle);
    }

    // Wait for all threads to complete
    for handle in threads {
        handle.join().expect("Thread panicked");
    }
}
