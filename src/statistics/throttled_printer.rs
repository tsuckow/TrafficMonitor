pub struct ThrottledPrinter {
    milliseconds: u32,
    tx: crossbeam_channel::Sender<String>,
    skipped: u32,
}

fn thread(rx: crossbeam_channel::Receiver<String>) {
    loop {
        let message = rx.recv().unwrap();
        println!("{}", message);
    }
}

impl ThrottledPrinter {
    pub fn new(milliseconds: u32) -> ThrottledPrinter {
        let (tx, rx) = crossbeam_channel::bounded(2);

        std::thread::spawn(move || {
            thread(rx);
        });

        ThrottledPrinter {
            milliseconds: milliseconds,
            tx: tx,
            skipped: 0,
        }
    }

    pub fn print(&mut self, message: &str) {
        if (self.tx.is_full()) {
            self.skipped += 1;
            return;
        }

        let string = if (self.skipped > 0) {
            format!("Skipped {} messages...\n{}", self.skipped, message)
        } else {
            message.to_string()
        };

        match self.tx.try_send(string) {
            Result::Ok(_v) => self.skipped = 0,
            Result::Err(crossbeam_channel::TrySendError::Full(_data)) => self.skipped += 1,
            Result::Err(crossbeam_channel::TrySendError::Disconnected(_data)) => {
                println!("PrinterDisconnected")
            }
        }
    }
}
