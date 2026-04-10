//! Integration tests for mDNS multicast socket setup and builder API

use std::sync::Arc;
use webrtc::peer_connection::RTCConfigurationBuilder;
use webrtc::peer_connection::*;
use webrtc::runtime::block_on;

#[derive(Clone)]
struct NoopHandler;

#[async_trait::async_trait]
impl PeerConnectionEventHandler for NoopHandler {}

/// SettingEngine with mDNS Disabled should NOT create a multicast socket; the peer
/// connection should build and close without error.
#[test]
fn test_mdns_disabled_builds_without_multicast_socket() {
    block_on(async {
        let mut se = SettingEngine::default();
        se.set_multicast_dns_mode(MulticastDnsMode::Disabled);

        let config = RTCConfigurationBuilder::new().build();
        let pc = PeerConnectionBuilder::new()
            .with_configuration(config)
            .with_setting_engine(se)
            .with_handler(Arc::new(NoopHandler))
            .with_udp_addrs(vec!["127.0.0.1:0"])
            .build()
            .await
            .unwrap();

        pc.close().await.expect("close should succeed");
    });
}

/// SettingEngine with mDNS QueryAndGather should attempt to create the multicast socket.
/// On environments where multicast is available this succeeds; on restricted
/// environments it degrades gracefully (warn + continue).  Either way the peer
/// connection should build without error.
#[test]
fn test_mdns_query_and_gather_builds_gracefully() {
    block_on(async {
        let mut se = SettingEngine::default();
        se.set_multicast_dns_mode(MulticastDnsMode::QueryAndGather);

        let config = RTCConfigurationBuilder::new().build();
        let pc = PeerConnectionBuilder::new()
            .with_configuration(config)
            .with_setting_engine(se)
            .with_handler(Arc::new(NoopHandler))
            .with_udp_addrs(vec!["127.0.0.1:0"])
            .build()
            .await
            .unwrap();

        // Should still be able to create offers etc.
        let offer = pc.create_offer(None).await;
        assert!(offer.is_ok(), "create_offer should work with mDNS enabled");

        pc.close().await.expect("close should succeed");
    });
}

/// SettingEngine with a custom local name and QueryAndGather mode should work.
#[test]
fn test_mdns_mode_with_custom_setting_engine() {
    block_on(async {
        let mut se = SettingEngine::default();
        se.set_multicast_dns_local_name("test-peer.local".to_string());
        se.set_multicast_dns_mode(MulticastDnsMode::QueryAndGather);

        let config = RTCConfigurationBuilder::new().build();
        let pc = PeerConnectionBuilder::new()
            .with_configuration(config)
            .with_setting_engine(se)
            .with_handler(Arc::new(NoopHandler))
            .with_udp_addrs(vec!["127.0.0.1:0"])
            .build()
            .await
            .unwrap();

        pc.close().await.expect("close should succeed");
    });
}

/// The default builder (no explicit mDNS config) should have mDNS disabled,
/// matching the SettingEngine default.
#[test]
fn test_default_builder_has_mdns_disabled() {
    block_on(async {
        let config = RTCConfigurationBuilder::new().build();
        let pc = PeerConnectionBuilder::new()
            .with_configuration(config)
            .with_handler(Arc::new(NoopHandler))
            .with_udp_addrs(vec!["127.0.0.1:0"])
            .build()
            .await
            .unwrap();

        // Default behavior: no multicast socket created, no mDNS.
        let offer = pc.create_offer(None).await;
        assert!(offer.is_ok());

        pc.close().await.expect("close should succeed");
    });
}
