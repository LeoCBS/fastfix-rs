pub mod reader {

    use bytes::BytesMut;
    use nom::{IResult, number};

    struct UdpCodec {}

    impl pcap::PacketCodec for UdpCodec {
        type Item = UdpMessage;

        fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
            parse_message(&packet.data[42..]).expect("error to parse udp message")
        }
    }

    #[derive(Debug)]
    struct UdpMessage {
        header: UdpMessageHeader,
        data: bytes::Bytes,
    }

    #[derive(Debug)]
    struct UdpMessageHeader {
        msg_seq_number: u32,
        no_chunks: u16,
        current_chunk: u16,
        message_length: u16,
    }

    fn parse_message(input: &[u8]) -> anyhow::Result<UdpMessage> {
        let r: IResult<&[u8], u32> = number::complete::be_u32(input);
        let (rem, msg_seq_number) = r.map_err(|e| e.to_owned())?;
        let r: IResult<&[u8], u16> = number::complete::be_u16(rem);
        let (rem, no_chunks) = r.map_err(|e| e.to_owned())?;
        let r: IResult<&[u8], u16> = number::complete::be_u16(rem);
        let (rem, current_chunk) = r.map_err(|e| e.to_owned())?;
        let r: IResult<&[u8], u16> = number::complete::be_u16(rem);
        let (rem, message_length) = r.map_err(|e| e.to_owned())?;
        let buf = rem.to_owned();
        Ok(UdpMessage {
            header: UdpMessageHeader {
                msg_seq_number,
                no_chunks,
                current_chunk,
                message_length,
            },
            data: buf.into(),
        })
    }

    pub fn read_first_packet_data(pcap_path: &str) -> Option<BytesMut> {
        let packets = pcap::Capture::from_file(pcap_path).unwrap();
        let mut packet_iter = packets.iter(UdpCodec {}).peekable();
        let mut message_bytes = bytes::BytesMut::new();
        if let Some(Ok(first)) = packet_iter.peek() {
            message_bytes.extend(first.data.clone());
            Some(message_bytes)
        } else {
            None
        }
    }
}
