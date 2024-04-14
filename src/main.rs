use clap::Parser;
use std::error::Error;
use std::fs;
use std::io::Write;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, group = "change", value_name = "AMOUNT")]
    increment: Option<u8>,
    #[arg(short, long, group = "change")]
    decrement: Option<u8>,
    #[arg(short, long, group = "change")]
    set: Option<u8>,
    #[arg(short, long, group = "change", action)]
    list_devs: bool,
    #[arg(short, long, group = "change", action)]
    get: bool,
    #[arg(short = 'f', long)]
    device: Option<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Handle list-devs
    if args.list_devs {
        match get_devices() {
            Ok(devs) => println!("\x1b[1;32mAvailable backlights\x1b[0m:\n{}", devs.join("\n")),
            Err(err) => println!("{}", err)
        }
        return Ok(());
    }

    if args.decrement.is_some() 
        || args.increment.is_some()
        || args.set.is_some() 
        || args.get == true {
            let device_path = match &args.device {
                Some(dev) => format!("/sys/class/backlight/{}/brightness", dev),
                None => {
                    println!("\x1b[1;31mError:\x1b[0m No device specified");
                    std::process::exit(1);
                }
            };
    
            if args.get {
                println!("{}", get_brightness(&device_path)?);
            } else {
                change_brightness(&device_path, &args)?;
            }
        }

    Ok(())
}

fn get_devices() -> Result<Vec<String>, Box<dyn Error>> {
    let mut devices: Vec<String> = vec![];
    let backlight_class_dir = fs::read_dir("/sys/class/backlight/")?;
    for file in backlight_class_dir {
        devices.push(file?.file_name().to_str().unwrap().to_owned());
    }

    Ok(devices)
}

fn change_brightness(device_path: &String, args: &Args) -> Result<(), Box<dyn Error>> {
    // Open backlight device
    let mut device = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(device_path)?;

    if let Some(val) = args.increment {
        // Increment by val, set to 255 if overflow
        let new_val = get_brightness(device_path)?
            .checked_add(val)
            .unwrap_or(255);
        device.write(new_val.to_string().as_bytes())?;
        return Ok(());
    }

    if let Some(val) = args.decrement {
        // Decrement by val, set to 0 if underflow
        let new_val = get_brightness(device_path)?
            .checked_sub(val)
            .unwrap_or(0);
        device.write(new_val.to_string().as_bytes())?;
        return Ok(());
    }

    if let Some(val) = args.set {
        device.write(val.to_string().as_bytes())?;
        return Ok(());
    }

    Ok(())
}

fn get_brightness(device_path: &String) -> Result<u8, Box<dyn Error>> {
    let brightness = fs::read_to_string(device_path)?.trim().to_owned();
    let val = brightness.parse::<u8>()?;
    
    Ok(val)
}
