use cpal::traits::{DeviceTrait, HostTrait};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    println!("Available input devices:");
    for device in host.input_devices().unwrap() {
        println!("{} / {}", device.name().unwrap(), device.description().unwrap());
    }

    Ok(())
}