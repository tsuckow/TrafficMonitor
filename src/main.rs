mod statistics;
mod packet_reciever;
mod webservice;

fn main() {
    println!("Starting Traffic Monitor...");
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    std::thread::spawn(move || {
        statistics::thread(rx).unwrap();
    });
    
    std::thread::spawn(move || {
        packet_reciever::thread(tx).unwrap();
    });
    
    webservice::main();
}