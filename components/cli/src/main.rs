pub mod cli;

extern crate hiro_system_kit;

#[cfg(feature = "tcmalloc")]
#[global_allocator]
static GLOBAL: tcmalloc2::TcMalloc = tcmalloc2::TcMalloc;

fn main() {
    // Windows: initialise Winsock once on the main thread (std does this lazily
    // on first std::net use); database clients created on worker threads
    // otherwise fail with WSANOTINITIALISED (os error 10093).
    #[cfg(windows)]
    let _ = std::net::UdpSocket::bind("127.0.0.1:0");
    cli::main();
}
