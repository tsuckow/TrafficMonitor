mod packet_reciever;
mod statistics;
mod webservice;

fn main() {
    println!("Starting Traffic Monitor...");

    //let (tx, rx) = std::sync::mpsc::sync_channel(5);
    let (tx, rx) = crossbeam_channel::bounded(2);
    let tx2 = tx.clone();

    std::thread::spawn(move || {
        statistics::thread(rx).unwrap();
    });

    std::thread::spawn(move || {
        packet_reciever::thread(tx).unwrap();
    });

    webservice::thread(tx2);
}
