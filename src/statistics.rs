#[derive(Debug)]
pub struct PacketData {
    pub packet_number: u64,
}

pub enum Message {
    GotStatistics(pcap::Stat),
    GotPacket(PacketData),
    GetBitrate(std::sync::mpsc::Sender<String>),
}

mod throttled_printer;

pub fn thread(rx: crossbeam_channel::Receiver<Message>) -> Result<(), ()> {
    let mut printer = throttled_printer::ThrottledPrinter::new(1000);

    loop {
        let message = rx.recv().unwrap();

        match message {
            Message::GotStatistics(stats) => printer.print(&format!("Stats: {:?}", stats)),
            Message::GotPacket(pd) => printer.print(&format!("Packet: {:?}", pd)),
            Message::GetBitrate(_tx) => printer.print("GetBitrate"),
        }
    }

    Ok(())
}
