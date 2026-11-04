use qsl_protocol_comparison::*;
#[test]
fn tcp_and_grpc_validate_all_records_over_persistent_connections() {
    let fixtures = (0..FIXTURES).map(Fixture::new).collect::<Vec<_>>();
    for batch in BATCHES {
        let mut grpc = transport::GrpcClient::start(&fixtures, batch).unwrap();
        let mut stream = transport::GrpcClient::start_stream(&fixtures, batch).unwrap();
        for codec in Codec::ALL {
            let mut tcp = transport::TcpClient::start(codec).unwrap();
            let mut scratch = vec![0; 8192];
            let mut encoder = Encoder::new(&mut scratch);
            for i in 0..4 {
                let expected = (0..batch).fold(0u64, |sum, j| {
                    sum.rotate_left(5) ^ expected(Kind::Mixed, &fixtures[i * batch + j].record)
                });
                assert_eq!(
                    tcp.call(codec, &mut encoder, &fixtures, i * batch, batch)
                        .unwrap(),
                    expected
                );
                assert_eq!(grpc.call(i).unwrap(), expected);
                assert_eq!(stream.call(i).unwrap(), expected);
            }
        }
    }
}
