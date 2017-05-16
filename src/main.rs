extern crate url;
extern crate ws;
extern crate clap;

use ws::{Handler,Handshake,WebSocket,Sender,Message};
use url::Url;

#[derive(Debug)]
struct EchoFactory{
    flag: Arc<AtomicBool>
}

impl ws::Factory for EchoFactory {
    type Handler = EchoHandler;
    fn connection_made(&mut self, _:Sender) -> Self::Handler {
        EchoHandler{flag:self.flag.clone()}
    }

}
#[derive(Debug)]
struct EchoHandler {
    flag: Arc<AtomicBool>
}

impl Handler for EchoHandler {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        self.flag.store(true, Ordering::Relaxed);
        Ok(()) }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        use std::io::Write;
        match msg {
            Message::Text(s) => {
                let _ = std::io::stdout().write(s.as_bytes());
                let _ = std::io::stdout().write("\n".as_bytes());
            }
            Message::Binary(v) => {let _ = std::io::stdout().write(&v);}
        }
        Ok(())
    }
}

use std::sync::atomic::{AtomicBool,Ordering};
use std::sync::Arc;

fn spawn_client(addr:String, setup: Arc<AtomicBool>) -> Sender {
    use std::thread;
    if let Ok(u) = Url::parse(&addr) {
        let fac = EchoFactory{flag:setup};
        if let Ok(mut client) = WebSocket::new(fac) {
            if let Ok(_) = client.connect(u) {
                let outbound = client.broadcaster();
                thread::spawn(move || {
                    if let Ok(_) = client.run() {
                    } else { panic!("Run failed!")}
                });
                outbound
            } else { panic!("Connection Failed") }
        } else { panic!("Failed to create new Websocket") }
    } else { panic!("Url parsing error.")}
}

fn spawn_server(addr:String,setup: Arc<AtomicBool>) -> Sender {
    use std::thread;
    let fac = EchoFactory{flag:setup};
    if let Ok(server) = WebSocket::new(fac) {
        let outbound = server.broadcaster();
        thread::spawn(move || {
            if let Ok(_) =  server.listen(&addr) {

            } else { panic!("Server Listen failed.")}
        });
        outbound
    } else { panic!("New Server failed.")}
}


fn main() {
    let websocket_setup = Arc::new(AtomicBool::new(false));

    use clap::{App,Arg,SubCommand};
    let client_sub = SubCommand::with_name("client")
        .about("Create a websocket client that sends from STDIN, and echos to STDOUT")
        .arg(Arg::with_name("address")
             .required(true)
             .index(1));
    let server_sub = SubCommand::with_name("server")
        .about("Create a websocket server that broadcasts from STDIN, and echos to STDOUT")
        .arg(Arg::with_name("address")
             .required(true)
             .index(1));

    let matches = App::new("Blotto")
        .version("0.1")
        .about("Acts as an echo websocket client or server, using stdin and stdout.")
        .subcommand(client_sub)
        .subcommand(server_sub)
        .get_matches();

    let broadcast = match matches.subcommand() {
        ("client",Some(ref client_config)) => {
            let addr = client_config.value_of("address").unwrap().clone();
            spawn_client(String::from(addr),websocket_setup.clone())
        },
        ("server",Some(ref server_config)) => {
            let addr = server_config.value_of("address").unwrap().clone();
            spawn_server(String::from(addr),websocket_setup.clone())
        },
        _ => { return }
    };

    use std::io;
    use std::io::BufRead;

    while ! websocket_setup.load(Ordering::Relaxed) {}

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let _ = broadcast.send(line.unwrap());
    }
    use std::{thread,time};
    thread::sleep(time::Duration::from_millis(50));
}


