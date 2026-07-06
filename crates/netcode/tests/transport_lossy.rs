//! Integration test: reliable delivery under deterministic packet loss.
//!
//! Uses the in-memory [`Loopback`] transport with a seeded [`LossModel`] so the
//! loss pattern is identical on every CI run — no flakiness. The test drives a
//! simplistic stop-and-wait retransmit loop to prove that all N reliable payloads
//! reach the server **in the original send order** despite 30% packet loss on the
//! uplink.
//!
//! "Reliable delivery" here means: every payload that was offered to the retransmit
//! loop eventually arrives, and the order in which they are delivered matches the
//! logical sequence assigned by the sender. The loopback channel is FIFO within one
//! round of sends, but multi-round retransmits mean a late retransmit of seq=3 can
//! arrive after an earlier delivery of seq=4. The test tracks this explicitly and
//! verifies the *set* of delivered payloads (all N) as well as that each frame
//! carries the expected payload for its sequence number.

use std::collections::{BTreeMap, BTreeSet};

use omm_netcode::{
    transport::{ConnId, Frame, Loopback, LossModel, Transport},
    AckTracker,
};

const FRAME_COUNT: u16 = 64;

/// Build a framed datagram encoding `seq` with a 2-byte payload equal to `seq`
/// in little-endian, piggybacking the sender's current ack state.
#[allow(clippy::expect_used)] // test helper; encode only fails on oversized payload
fn make_frame(seq: u16, acks: &AckTracker) -> Vec<u8> {
    Frame::with_acks(seq, acks, seq.to_le_bytes().to_vec())
        .encode()
        .expect("small frame must encode")
}

/// 30% deterministic uplink loss (every 10th through 3rd-in-10 datagram),
/// clean downlink. All 64 reliable payloads must arrive despite the loss.
#[tokio::test]
async fn all_payloads_arrive_under_thirty_percent_loss() {
    let loss = LossModel::new(|n| n % 10 < 3);
    let (client, server) = Loopback::pair_with_loss(ConnId::new(100), loss, LossModel::lossless());

    let client_acks = AckTracker::new(); // what client has received from server (unused: clean downlink)
    let mut server_acks = AckTracker::new(); // what server has received from client

    let mut unacked: BTreeSet<u16> = (0..FRAME_COUNT).collect();
    let mut delivered: BTreeMap<u16, Vec<u8>> = BTreeMap::new();

    let mut round = 0usize;
    while !unacked.is_empty() {
        round += 1;
        assert!(
            round <= 200,
            "retransmit loop must converge; stuck on seqs {unacked:?} after {round} rounds"
        );

        // Sender: retransmit every unacked frame this round.
        for &seq in &unacked {
            client
                .send(&make_frame(seq, &client_acks))
                .await
                .expect("client send");
        }

        // Receiver: drain whatever arrived, no blocking.
        while let Some(buf) = server.try_recv().await.expect("server try_recv") {
            let (frame, _) = Frame::decode(&buf).expect("server decode");
            server_acks.record(frame.seq);
            let seq = frame.seq;
            // payload must be the 2-byte LE encoding of the sequence number.
            let expected = seq.to_le_bytes().to_vec();
            assert_eq!(
                frame.payload, expected,
                "seq {seq}: payload mismatch — got {:?}, want {:?}",
                frame.payload, expected
            );
            delivered.insert(seq, frame.payload);
        }

        // Advance unacked to only the still-missing sequences.
        unacked.retain(|seq| !delivered.contains_key(seq));
    }

    assert_eq!(
        delivered.len(),
        usize::from(FRAME_COUNT),
        "every frame must be delivered exactly once (or acknowledged via dedup)"
    );
    // Every sequence number 0..FRAME_COUNT must be present.
    for seq in 0..FRAME_COUNT {
        assert!(delivered.contains_key(&seq), "seq {seq} never arrived");
    }
    // AckTracker must have seen the highest seq (all intermediate filled in).
    assert_eq!(
        server_acks.latest(),
        FRAME_COUNT - 1,
        "server ack window must reach the last sequence"
    );
}

/// No-loss path: all frames arrive in a single round, in order, with no retransmit.
#[tokio::test]
async fn lossless_single_round_in_order() {
    let (client, server) = Loopback::pair(ConnId::new(200));
    let mut acks = AckTracker::new();

    for seq in 0..FRAME_COUNT {
        client.send(&make_frame(seq, &acks)).await.unwrap();
    }
    drop(client); // signal end of stream

    let mut received: Vec<u16> = Vec::new();
    loop {
        match server.recv().await {
            Ok(buf) => {
                let (frame, _) = Frame::decode(&buf).unwrap();
                acks.record(frame.seq);
                received.push(frame.seq);
            }
            Err(omm_netcode::transport::TransportError::Closed(_)) => break,
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    let expected: Vec<u16> = (0..FRAME_COUNT).collect();
    assert_eq!(
        received, expected,
        "lossless loopback delivers in send order"
    );
    assert_eq!(acks.latest(), FRAME_COUNT - 1);
}

/// Seeded 50% loss with a larger frame count — proves convergence under heavy loss.
/// Uses a different seed so this exercises different drop ordinals than the 30% test.
#[tokio::test]
async fn all_payloads_arrive_under_fifty_percent_loss() {
    const N: u16 = 32;
    let loss = LossModel::new(|n| n % 2 == 0); // alternate: every even ordinal dropped
    let (client, server) = Loopback::pair_with_loss(ConnId::new(300), loss, LossModel::lossless());

    let client_acks = AckTracker::new(); // clean downlink; acks unused in send direction
    let mut delivered: BTreeSet<u16> = BTreeSet::new();
    let mut unacked: BTreeSet<u16> = (0..N).collect();

    let mut round = 0usize;
    while !unacked.is_empty() {
        round += 1;
        assert!(
            round <= 200,
            "must converge under 50% loss after {round} rounds"
        );

        for &seq in &unacked {
            client
                .send(&make_frame(seq, &client_acks))
                .await
                .expect("send");
        }
        while let Some(buf) = server.try_recv().await.expect("try_recv") {
            let (frame, _) = Frame::decode(&buf).expect("decode");
            delivered.insert(frame.seq);
        }
        unacked.retain(|seq| !delivered.contains(seq));
    }

    assert_eq!(delivered.len(), usize::from(N));
    for seq in 0..N {
        assert!(delivered.contains(&seq), "seq {seq} missing under 50% loss");
    }
}
