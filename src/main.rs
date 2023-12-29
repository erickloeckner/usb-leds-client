use std::env;
use std::str::FromStr;
use serialport::{available_ports, ClearBuffer, FlowControl, SerialPortType};
use std::time::{Duration, Instant};
use std::thread::sleep;

fn parse_color(data: &[u8]) -> [f32; 3] {
    let mut out = [0.0, 0.0, 0.0];
    for (index, value) in data.chunks(4).enumerate() {
        if index > 2 { break }
        out[index] = f32::from_le_bytes([value[0], value[1], value[2], value[3]]);
    }
    out
}

fn send_command(command: u8, pattern: u8, color_vals: [f32; 6], name: &str) {
    let color_1_h = color_vals[0].to_le_bytes();
    let color_1_s = color_vals[1].to_le_bytes();
    let color_1_v = color_vals[2].to_le_bytes();
    let color_2_h = color_vals[3].to_le_bytes();
    let color_2_s = color_vals[4].to_le_bytes();
    let color_2_v = color_vals[5].to_le_bytes();

    let builder = serialport::new(name, 115200);
    let mut port = builder.open().unwrap();
    port.set_flow_control(FlowControl::Hardware).ok();
    //~ let flow_control = port.flow_control();
    //~ println!("flow control: {:?}", flow_control);
    //~ println!("CTS: {:?}", port.read_clear_to_send());
    
    match command {
        0 => {}
        1 => {
            port.write(&[
                command,
                pattern,
                color_1_h[0], color_1_h[1], color_1_h[2], color_1_h[3],
                color_1_s[0], color_1_s[1], color_1_s[2], color_1_s[3],
                color_1_v[0], color_1_v[1], color_1_v[2], color_1_v[3],
                color_2_h[0], color_2_h[1], color_2_h[2], color_2_h[3],
                color_2_s[0], color_2_s[1], color_2_s[2], color_2_s[3],
                color_2_v[0], color_2_v[1], color_2_v[2], color_2_v[3],
            ]).ok();
        }
        2 => {
            let mut buf = [0; 26];
            port.clear(ClearBuffer::Input).ok();
            
            port.write_request_to_send(false).ok();
            port.write(&[command]).ok();
            
            let timeout = Instant::now();
            let mut buffer_full = false;
            port.write_request_to_send(true).ok();
            loop {
                if timeout.elapsed() >= Duration::from_secs(10) {
                    break;
                }
                if port.bytes_to_read().unwrap_or(0) >= 26 { 
                    //buffer_full = true;
                    //port.read_exact(&mut buf).ok();
                    match port.read_exact(&mut buf) {
                        Ok(_) => {
                            buffer_full = true;
                        }
                        Err(e) => {
                            println!("read_exact() error: {:?}", e);
                        }
                    }
                    break;
                }
                sleep(Duration::from_millis(1));
            }
            
            if buffer_full {
                //port.read_exact(&mut buf).ok();
                let color_1 = parse_color(&buf[1..13]);
                let color_2 = parse_color(&buf[13..25]);
                println!("pattern: {}", buf[0]);
                println!("color1: {:?}", color_1);
                println!("color2: {:?}", color_2);
            } else {
                println!("timeout reached");
            }
        }
        _ => {}
    }
}

fn main() {
    let serial_number = env::args().nth(1).unwrap_or("".to_string());
    
    let command_string = env::args().nth(2).unwrap_or("0".to_string());
    let command = u8::from_str_radix(&command_string, 10).unwrap_or(0);
    let pattern_string = env::args().nth(3).unwrap_or("0".to_string());
    let pattern = u8::from_str_radix(&pattern_string, 10).unwrap_or(0);
    
    let mut color_vals = [0.0; 6];
    for (index, value) in env::args().skip(4).enumerate() {
        if index > 5 { break }
        color_vals[index] = f32::from_str(&value).unwrap_or(0.0).max(0.0).min(1.0);
    }

    match available_ports() {
        Ok(ports) => {
            for p in ports {
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        match info.serial_number {
                            Some(s) => {
                                if s == serial_number { 
                                    send_command(command, pattern, color_vals, &p.port_name);
                                }
                            }
                            None => {}
                        }
                    }
                    SerialPortType::BluetoothPort => {},
                    SerialPortType::PciPort => {},
                    SerialPortType::Unknown => {},
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
}
