extern crate pcap;
extern crate libc;
extern crate packet;
use self::pcap::{Capture};
extern crate std;

const RATE_INTERVAL_SECONDS:f64 = 10f64;
const RATE_INTERVALS_LIMIT:usize = 6;

struct PacketHandler {
    total_bytes: u64, 
    start_seconds: f64,  
    rate_bytes_count: Vec<u64>,
    rate_last_start: f64,
}

fn timeval_to_f64(timeval: libc::timeval) -> f64 {
    timeval.tv_sec as f64 + (timeval.tv_usec as f64) / 1000000f64
}

impl PacketHandler {
    fn handle_packet(&mut self, packet: pcap::Packet, packet_number: u64) -> bool {
        let packet_size = packet.header.len as u64;
        let mut did_rate = false;
        let packet_seconds = timeval_to_f64(packet.header.ts);
        if packet_seconds > self.rate_last_start + RATE_INTERVAL_SECONDS {
          self.rate_last_start = packet_seconds;
          if self.rate_bytes_count.len() >= RATE_INTERVALS_LIMIT {
            self.rate_bytes_count.pop();
          }
          self.rate_bytes_count.insert(0, 0);
          did_rate = true;
        }
        
        self.rate_bytes_count[0] += packet_size;
         
        //println!("received packet! {:?} {:?}", packet.data.len(), packet.header);
        self.total_bytes += packet.header.len as u64;
         
        if self.start_seconds == 0f64 {
            self.start_seconds = packet_seconds;
        }
         
        let parsed_packet = packet::ether::Packet::new(packet.data);
         
        
         
        let diff = packet_seconds - self.start_seconds;
         
        let avg_bps = self.total_bytes as f64 / diff;
        
        if did_rate {
            println!("Parsed: {:?}", parsed_packet);
            println!("Total: {:?}  Time: {:?}  Avg Bps: {:?}", self.total_bytes, diff as u64, avg_bps);
            
            if self.rate_bytes_count.len() > 1 {
              println!("KiBps  {:?}", self.rate_bytes_count[1] as f64 / RATE_INTERVAL_SECONDS / 1024f64);  
            } 
        }
        
        did_rate
    }
    
    fn new() -> PacketHandler {
        PacketHandler {
            total_bytes: 0,
            start_seconds: 0.,
            rate_bytes_count: Vec::with_capacity(RATE_INTERVALS_LIMIT),
            rate_last_start: 0.
        }
    }
}

pub fn thread(tx : std::sync::mpsc::Sender<String>) -> Result<(), pcap::Error> {
    let inactive_capture = Capture::from_device("enp14s0")?;
    let mut cap = inactive_capture
                      .promisc(true)
                      //.snaplen(5000)
                      .open()?;


    //std::thread::sleep(std::time::Duration::from_millis(2000));

    let mut handler = PacketHandler::new();
    
    let mut packet_number = 0u64;

    loop {
        let did_rate = match cap.next() {
            Ok(packet) => handler.handle_packet(packet, packet_number),
            Err(error) => return Err(error),
        };
        
        if did_rate {
            println!("Stats: {:?}", cap.stats());
        }
        
        packet_number += 1;
    }
    
    // while let Ok(packet) = cap.next() {
    //     println!("received packet! {:?}", packet.data.len());
        
    //     println!("Stats: {:?}", cap.stats());
    // }
    
    Ok(())
}