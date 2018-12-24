extern crate std;

pub fn thread(rx : std::sync::mpsc::Receiver<String>) -> Result<(), ()> {
    Ok(())
}