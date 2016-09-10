extern crate time;
extern crate byteorder;
extern crate rand;

use std::net::TcpStream;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::{Read, Write, Cursor};
use std::mem;
use rand::{thread_rng, Rng};
use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let num_messages = args[1].parse::<usize>().unwrap();
    let message_size_bytes = args[2].parse::<usize>().unwrap();
    let (duration, latencies) = send_messages(num_messages, message_size_bytes);
    let duration_sec = duration as f64 / (1000.0 * 1000.0 * 1000.0);
    let bytes_proc = (num_messages * (message_size_bytes + 4)) as f64;
    let mb_proc = bytes_proc / (1000.0 * 1000.0);
    let thru = mb_proc / duration_sec;
    let mean_lat = latencies.iter().fold(0, |acc, &x| acc + x) as f64 / (latencies.len() as f64) /
                   (1000.0);

    println!("Processed {} bytes in {} seconds. Throughput: {} MBps, recorded batches: {}, mean \
              latency: {} microseconds",
             bytes_proc,
             duration_sec,
             thru,
             latencies.len(),
             mean_lat);

}


fn send_messages(num_messages: usize, message_size_bytes: usize) -> (u64, Vec<u64>) {


    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:7777").unwrap();
    stream.set_nodelay(true).unwrap();
    let expected_response = (0..100).collect::<Vec<u32>>();
    let mut lat_tracker = Vec::new();
    let message = gen_message(message_size_bytes);

    let bench_start_time = time::precise_time_ns();
    for m in 0..num_messages {
        if m % 200 == 0 {
            println!("Sent {} messages", m);
        }
        let start_time = time::precise_time_ns();
        // let message = gen_message(message_size_bytes);
        stream.write_all(&message[..]).unwrap();
        stream.flush().unwrap();
        let num_response_bytes = 100 * mem::size_of::<u32>();
        let mut response_buffer: Vec<u8> = vec![0; num_response_bytes];
        stream.read_exact(&mut response_buffer).unwrap();
        let mut cursor = Cursor::new(response_buffer);
        let mut responses: Vec<u32> = Vec::with_capacity(100);
        for _ in 0..100 {
            responses.push(cursor.read_u32::<LittleEndian>().unwrap());
        }
        assert_eq!(responses, expected_response);
        let end_time = time::precise_time_ns();
        lat_tracker.push(end_time - start_time);
    }
    let bench_end_time = time::precise_time_ns();
    (bench_end_time - bench_start_time, lat_tracker)
}

fn gen_message(size: usize) -> Vec<u8> {
    let mut message = Vec::new();
    message.write_u32::<LittleEndian>(size as u32).unwrap();
    let mut rng = thread_rng();
    // let x = rng.gen_iter::<u8>().take(size).collect::<Vec<u8>>();
    message.extend(rng.gen_iter::<u8>().take(size));
    assert!(message.len() == size + 4);
    message
}
