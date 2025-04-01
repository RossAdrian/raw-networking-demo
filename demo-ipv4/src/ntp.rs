// NTP (application layer)
use chrono::DateTime;

/// Gets the NTP payload and returns the timestamp as string
pub fn get_timestamp(payload: &[u8]) -> String {
    // extract transmit timestamp from payload
    let ntp_timestamp: [u8; 8] = payload[40..48].try_into().unwrap();

    // Split into seconds (first 4 bytes) and fractional seconds (last 4 bytes)
    let seconds_part = u32::from_be_bytes([ntp_timestamp[0], ntp_timestamp[1], ntp_timestamp[2], ntp_timestamp[3]]);

    // Convert NTP seconds to Unix timestamp
    let unix_time = (seconds_part - 2208988800) as i64;

    // Format the Unix timestamp
    let naive_datetime = DateTime::from_timestamp(unix_time, 0).expect("Invalid timestamp");
    naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Gets the data to send, and adds the NTP request payload
pub fn add_request_payload(data: &mut Vec<u8>) {
    // Leap Indicator = 0, Version = 3, Mode = Client
    data.push(0x1B);

    // push 47 0x00 bytes
    data.extend_from_slice(&[0x00u8; 47]);
}