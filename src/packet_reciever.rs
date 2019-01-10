use pcap::Capture;

use super::statistics;

const RATE_INTERVAL_SECONDS: f64 = 2f64;

struct PacketHandler {
    rate_last_start: f64,
}

fn timeval_to_f64(timeval: libc::timeval) -> f64 {
    timeval.tv_sec as f64 + (timeval.tv_usec as f64) / 1000000f64
}

impl PacketHandler {
    fn should_statistics(&mut self, packet: &pcap::Packet, packet_number: u64) -> bool {
        let packet_size = packet.header.len as u64;
        let mut did_rate = false;
        //Fixme: we don't care about subsecond, why use floating?
        let packet_seconds = timeval_to_f64(packet.header.ts);
        if packet_seconds > self.rate_last_start + RATE_INTERVAL_SECONDS {
            self.rate_last_start = packet_seconds;
            did_rate = true;
        }

        did_rate
    }

    fn new() -> PacketHandler {
        PacketHandler {
            rate_last_start: 0.,
        }
    }
}

fn handle_packet(packet: &pcap::Packet, packet_number: u64) -> statistics::PacketData {
    //Fixme, deal with parse error
    let parsed_packet = packet::ether::Packet::new(packet.data).unwrap();
    let packet_seconds = packet.header.ts.tv_sec as i32;
        
    statistics::PacketData {
            packet_number,
            packet_seconds,
            size: packet.header.len,
            ethernet_destination: parsed_packet.destination(),
            ethernet_source: parsed_packet.source(),
        }
}

pub fn thread(tx: crossbeam_channel::Sender<statistics::Message>) -> Result<(), pcap::Error> {
    let inactive_capture = Capture::from_device("enp14s0")?;
    let mut cap = inactive_capture
        .promisc(true)
        //.snaplen(5000)
        .open()?;

    //std::thread::sleep(std::time::Duration::from_millis(2000));

    let mut handler = PacketHandler::new();

    let mut packet_number = 0u64;

    loop {
        let packet = match cap.next() {
            Ok(packet) => packet,
            Err(error) => return Err(error),
        };

        let did_rate = handler.should_statistics(&packet, packet_number);

        //Fixme, deal with parse error
        //let parsed_packet = packet::ether::Packet::new(packet.data).unwrap();

        let packet_data = handle_packet(&packet, packet_number);

        tx.send(statistics::Message::GotPacket(packet_data));
        // tx.send(statistics::Message::GotPacket(statistics::PacketData {
        //     packet_number: packet_number,
        //     ethernet_destination: parsed_packet.destination(),
        //     ethernet_source: parsed_packet.source(),
        // }));

        if did_rate {
            //.stats() is expensive
            tx.send(statistics::Message::GotStatistics(cap.stats().unwrap()));
            //tx.send(statistics::Message::GotPacket(statistics::PacketData { message: format!("Stats: {:?}", cap.stats())}));
            //println!("Stats Periodic: {:?}", cap.stats());
        }

        packet_number += 1;
        if packet_number == std::u64::MAX {
            panic!("Number of packets exceeded u64, how many years was this running?!?!");
        }
    }

    Ok(())
}
