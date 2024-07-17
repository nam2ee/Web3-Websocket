use pcap::{Capture, Device};
use std::net::Ipv4Addr;

fn main() {
    // 네트워크 인터페이스를 선택합니다.
    let device = Device::lookup().unwrap();
    println!("사용 중인 네트워크 인터페이스: {:?}", device);

    // 네트워크 인터페이스에서 패킷을 캡처합니다.
    let mut cap = Capture::from_device(device)
        .unwrap()
        .promisc(true)
        .snaplen(5000)
        .open()
        .unwrap();

    // 패킷을 읽고 로그를 출력합니다.
    while let Ok(packet) = cap.next() {
        // 패킷 데이터에서 IP 헤더를 추출합니다.
        let data = packet.data;
        if data.len() >= 20 {
            let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
            let dst_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);
            println!("출발지 IP: {}, 목적지 IP: {}", src_ip, dst_ip);
        } else {
            println!("패킷 데이터가 너무 짧습니다.");
        }
    }
}
