use std::collections::HashMap;

const RATE_INTERVAL_SECONDS: i32 = 1;
const RATE_INTERVALS_LIMIT: usize = 60;

#[derive(Debug)]
pub struct PacketData {
    pub packet_number: u64,
    pub packet_seconds: i32,
    pub size: u32,
    pub ethernet_destination: hwaddr::HwAddr,
    pub ethernet_source: hwaddr::HwAddr,
}

type BoxedFn<A> = Box<dyn FnMut(A) + Send + 'static>;

#[derive(Debug)]
pub struct Bitrates {
    pub avg_bytes_per_second: u64,
    pub seconds_rates: Vec<u64>,
}

pub enum Message {
    GotStatistics(pcap::Stat),
    GotPacket(PacketData),
    GetBitrate(BoxedFn<Bitrates>),
    //GetString(crossbeam_channel::Sender<String>),
    GetString(BoxedFn<String>),
}

mod throttled_printer;

#[derive(Debug)]
struct Statistics {
    total_seconds: u32,
    total_bytes: u64,
    start_seconds: i32,
    rate_bytes_count: Vec<u64>,
    rate_last_start: i32,
    
    ethernet_sources: HashMap<hwaddr::HwAddr, u64>,
    ethernet_destinations: HashMap<hwaddr::HwAddr, u64>,
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            total_seconds: 0,
            total_bytes: 0,
            start_seconds: 0,
            rate_bytes_count: Vec::with_capacity(RATE_INTERVALS_LIMIT),
            rate_last_start: 0,
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
        
        if pd.packet_seconds > self.rate_last_start + RATE_INTERVAL_SECONDS {
            self.rate_last_start = pd.packet_seconds;
            if self.rate_bytes_count.len() >= RATE_INTERVALS_LIMIT {
                self.rate_bytes_count.pop();
            }
            self.rate_bytes_count.insert(0, 0);
        }

        self.rate_bytes_count[0] += pd.size as u64;

        //println!("received packet! {:?} {:?}", packet.data.len(), packet.header);
        self.total_bytes += pd.size as u64;

        if self.start_seconds == 0 {
            self.start_seconds = pd.packet_seconds;
        }

        self.total_seconds = (pd.packet_seconds - self.start_seconds) as u32;
        
        //printer.print(&format!("Packet: {:?}, {}", pd, pd.ethernet_source.to_string()));
        //printer.print(&format!("Counts: {:?}", self));
    }

    fn get_bitrates(&self) -> Bitrates {
        Bitrates {
            avg_bytes_per_second: self.total_bytes / self.total_seconds as u64,
            seconds_rates: self.rate_bytes_count.clone(),
        }
    }
}

pub fn thread(rx: crossbeam_channel::Receiver<Message>) -> Result<(), Box<std::error::Error>> {
    let mut printer = throttled_printer::ThrottledPrinter::new(1000);
    let mut statistics = Statistics::new();

    loop {
        let message = rx.recv()?;

        match message {
            Message::GotStatistics(stats) => printer.print(&format!("Stats: {:?}", stats)),
            Message::GotPacket(pd) => statistics.got_packet(&pd, &mut printer),
            Message::GetBitrate(mut cb) => cb(statistics.get_bitrates()),
            Message::GetString(mut cb) => cb("Response".to_string()),
        }
    }

    Ok(())
}




// const RATE_INTERVAL_SECONDS: f64 = 2f64;
// const RATE_INTERVALS_LIMIT: usize = 6;

// struct PacketHandler {
//     total_bytes: u64,
//     start_seconds: f64,
//     rate_bytes_count: Vec<u64>,
//     rate_last_start: f64,
// }


// let packet_size = packet.header.len as u64;
//         let mut did_rate = false;
//         let packet_seconds = timeval_to_f64(packet.header.ts);
//         if packet_seconds > self.rate_last_start + RATE_INTERVAL_SECONDS {
//             self.rate_last_start = packet_seconds;
//             if self.rate_bytes_count.len() >= RATE_INTERVALS_LIMIT {
//                 self.rate_bytes_count.pop();
//             }
//             self.rate_bytes_count.insert(0, 0);
//             did_rate = true;
//         }

//         self.rate_bytes_count[0] += packet_size;

//         //println!("received packet! {:?} {:?}", packet.data.len(), packet.header);
//         self.total_bytes += packet.header.len as u64;

//         if self.start_seconds == 0f64 {
//             self.start_seconds = packet_seconds;
//         }

//         let parsed_packet = packet::ether::Packet::new(packet.data);

//         let diff = packet_seconds - self.start_seconds;

//         let avg_bps = self.total_bytes as f64 / diff;
