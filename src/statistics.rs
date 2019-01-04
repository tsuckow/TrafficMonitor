use std::collections::HashMap;

#[derive(Debug)]
pub struct PacketData {
    pub packet_number: u64,
    pub ethernet_destination: hwaddr::HwAddr,
    pub ethernet_source: hwaddr::HwAddr,
}

type BoxedFn = Box<dyn FnMut() + Send + 'static>;

pub enum Message {
    GotStatistics(pcap::Stat),
    GotPacket(PacketData),
    GetBitrate(crossbeam_channel::Sender<String>),
    //GetString(crossbeam_channel::Sender<String>),
    GetString(BoxedFn),
}

mod throttled_printer;

#[derive(Debug)]
struct Statistics {
    ethernet_sources: HashMap<hwaddr::HwAddr, u64>,
    ethernet_destinations: HashMap<hwaddr::HwAddr, u64>,
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            ethernet_sources: HashMap::new(),
            ethernet_destinations: HashMap::new(),
        }
    }

    fn got_packet(&mut self, pd: &PacketData, printer: &mut throttled_printer::ThrottledPrinter) {
        {
            let source_count = self.ethernet_sources.entry(pd.ethernet_source).or_insert(0);
            *source_count += 1;
        }
        {
            let source_count = self
                .ethernet_destinations
                .entry(pd.ethernet_destination)
                .or_insert(0);
            *source_count += 1;
        }
        //printer.print(&format!("Packet: {:?}, {}", pd, pd.ethernet_source.to_string()));
        printer.print(&format!("Counts: {:?}", self));
    }

    fn to_string(&self) -> String {
        "".to_string()
    }
}

pub fn thread(rx: crossbeam_channel::Receiver<Message>) -> Result<(), ()> {
    let mut printer = throttled_printer::ThrottledPrinter::new(1000);
    let mut statistics = Statistics::new();

    loop {
        let message = rx.recv().unwrap();

        match message {
            Message::GotStatistics(stats) => printer.print(&format!("Stats: {:?}", stats)),
            Message::GotPacket(pd) => statistics.got_packet(&pd, &mut printer),
            Message::GetBitrate(_tx) => printer.print("GetBitrate"),
            //Message::GetString(tx) => tx.send("String".to_string()).unwrap(),
            Message::GetString(mut func) => {
                func();
            }
        }
    }

    Ok(())
}
