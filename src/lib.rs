use std::collections::BTreeMap;
use std::net::IpAddr;

use etherparse;
use ipnet::IpNet;
use log;
use riptun::TokioTun;
use serde::{Deserialize, Serialize};
use tokio;

use reticulum::destination::DestinationName;
use reticulum::destination::link::{LinkEvent, LinkId};
use reticulum::hash::AddressHash;
use reticulum::identity::PrivateIdentity;
use reticulum::transport::Transport;

// TODO: config?
const TUN_NQUEUES : usize = 1;
const MTU: usize = 1500;

const fn default_announce_freq_secs() -> u32 { 1 }

#[derive(Deserialize, Serialize)]
pub struct Config {
  pub vpn_ip: IpNet,
  /// Map of (IP, destination hash)
  pub peers: BTreeMap<IpAddr, String>,
  #[serde(default = "default_announce_freq_secs")]
  pub announce_freq_secs: u32
}

pub struct Client {
  config: Config,
  tun: Tun
}

#[derive(Debug)]
pub enum CreateClientError {
  ConfigError(String),
  RiptunError(riptun::Error),
  IpAddBroadcastError(std::io::Error),
  IpLinkUpError(std::io::Error),
  IpRouteAddError(std::io::Error),
  IptablesError(std::io::Error)
}

struct Peer {
  dest: AddressHash,
  link_id: Option<LinkId>,
  link_active: bool
}

struct Tun {
  tun: TokioTun,
  read_buf: tokio::sync::Mutex<[u8; MTU]>
}

impl Client {
  pub fn new(config: Config) -> Result<Self, CreateClientError> {
    if config.peers.contains_key(&config.vpn_ip.addr()) {
      log::error!("configured VPN IP ({}) conflicts with peer IPs: {:?}",
        config.vpn_ip, config.peers);
      return Err(CreateClientError::ConfigError(
        "configured VPN IP exists in peer IPs".to_owned()))
    }
    let tun = Tun::new(config.vpn_ip)?;
    Ok(Client { config, tun })
  }

  pub async fn run(&self, mut transport: Transport, id: PrivateIdentity) {
    // create in destination
    let in_destination = transport
      .add_destination(id, DestinationName::new("rns_vpn", "client")).await;
    let in_destination_hash = in_destination.lock().await.desc.address_hash;
    log::info!("created destination: {}",
      format!("{}", in_destination_hash).trim_matches('/'));
    // set up peer map
    let peer_map = {
      let mut peer_map = BTreeMap::<IpAddr, Peer>::new();
      for (ip, dest) in self.config.peers.iter().map(|(k,v)| (k.clone(), v.clone())) {
        let dest = match AddressHash::new_from_hex_string(dest.as_str()) {
          Ok(dest) => dest,
          Err(err) => {
            log::error!("error parsing peer destination hash: {err:?}");
            return
          }
        };
        let peer = Peer { dest, link_id: None, link_active: false };
        assert!(peer_map.insert(ip, peer).is_none());
      }
      tokio::sync::Mutex::new(peer_map)
    };
    // send announces
    let announce_loop = async || loop {
      transport.send_announce(&in_destination, None).await;
      tokio::time::sleep(
        std::time::Duration::from_secs(self.config.announce_freq_secs as u64)
      ).await;
    };
    // set up links
    let link_loop = async || {
      let mut announce_recv = transport.recv_announces().await;
      while let Ok(announce) = announce_recv.recv().await {
        let destination = announce.destination.lock().await;
        // loop up destination in peers
        for peer in peer_map.lock().await.values_mut() {
          if destination.desc.address_hash == peer.dest {
            if peer.link_id.is_none() {
              let link = transport.link(destination.desc).await;
              peer.link_id = Some(link.lock().await.id().clone());
              log::debug!("created link {} for peer {}",
                peer.link_id.as_ref().unwrap(), peer.dest);
              peer.link_active = false;   // wait for link activated event
            }
          }
        }
      }
    };
    // tun loop: read data from tun and send on links
    let tun_loop = async || {
      while let Ok(bytes) = self.tun.read().await {
        log::trace!("got tun bytes ({})", bytes.len());
        if let Ok((ip_header, _)) = etherparse::IpHeaders::from_slice(bytes.as_slice())
          .map_err(|e| log::error!("couldn't parse packet from tun: {e:?}"))
        {
          let mut destination_ip = None;
          if let Some((ipv4_header, _)) = ip_header.ipv4() {
            destination_ip = Some(IpAddr::from(ipv4_header.destination));
          } else if let Some((ipv6_header, _)) = ip_header.ipv6() {
            destination_ip = Some(IpAddr::from(ipv6_header.destination));
          } else {
            log::error!("failed to get ipv4 or ipv6 headers from ip header: {:?}", ip_header);
          }
          if let Some(destination_ip) = destination_ip {
            if let Some(peer) = peer_map.lock().await.get(&destination_ip) {
              if let Some(link_id) = peer.link_id.as_ref() {
                if let Some(link) = transport.find_out_link(&peer.dest).await {
                  log::trace!("sending to {} on link {}", peer.dest, link_id);
                  let link = link.lock().await;
                  let packet = link.data_packet(&bytes).unwrap();
                  drop(link);
                  transport.send_packet(packet).await;
                } else {
                  log::warn!("could not get link {} for peer {}", link_id, peer.dest);
                }
              }
            }
          }
        }
      }
    };
    // upstream link data: put link data into tun
    let upstream_loop = async || {
      let mut in_link_events = transport.in_link_events();
      while let Ok(link_event) = in_link_events.recv().await {
        match link_event.event {
          LinkEvent::Data(payload) => if link_event.address_hash == in_destination_hash {
            log::trace!("link {} payload ({})", link_event.id, payload.len());
            match self.tun.send(payload.as_slice()).await {
              Ok(n) => log::trace!("tun sent {n} bytes"),
              Err(err) => {
                log::error!("tun error sending bytes: {err:?}");
                break
              }
            }
          }
          LinkEvent::Activated => if link_event.address_hash == in_destination_hash {
            log::debug!("link activated {}", link_event.id);
            // look up destination in peers
            for peer in peer_map.lock().await.values_mut() {
              if peer.link_id == Some(link_event.id) {
                peer.link_active = true;
              }
            }
          }
          LinkEvent::Closed => if link_event.address_hash == in_destination_hash {
            log::debug!("link closed {}", link_event.id);
            // remove closed link
            for peer in peer_map.lock().await.values_mut() {
              if peer.link_id == Some(link_event.id) {
                peer.link_active = false;
                let _ = peer.link_id.take();
              }
            }
          }
        }
      }
    };
    tokio::select!{
      _ = announce_loop() => log::info!("announce loop exited: shutting down"),
      _ = link_loop() => log::info!("link loop exited: shutting down"),
      _ = tun_loop() => log::info!("tun loop exited: shutting down"),
      _ = upstream_loop() => log::info!("upstream loop exited: shutting down"),
      _ = tokio::signal::ctrl_c() => log::info!("got ctrl-c: shutting down")
    }
  }
}

impl Tun {
  pub fn new(ip: IpNet) -> Result<Self, CreateClientError> {
    log::debug!("creating tun device");
    let ip: IpNet = ip.into();
    let tun = TokioTun::new("rip%d", TUN_NQUEUES)
      .map_err(CreateClientError::RiptunError)?;
    log::debug!("created tun device: {}", tun.name());
    log::debug!("adding broadcast ip addr: {}", ip);
    let output = std::process::Command::new("ip")
      .arg("addr")
      .arg("add")
      .arg(ip.to_string())
      .arg("brd")
      .arg(ip.addr().to_string())
      .arg("dev")
      .arg(tun.name())
      .output()
      .map_err(CreateClientError::IpAddBroadcastError)?;
    if !output.status.success() {
      return Err(CreateClientError::IpAddBroadcastError(
        std::io::Error::other(format!("ip addr add command failed ({:?})",
          output.status.code())).into()));
    }
    log::debug!("{} setting link up", tun.name());
    let output = std::process::Command::new("ip")
      .arg("link")
      .arg("set")
      .arg("dev")
      .arg(tun.name())
      .arg("up")
      .output()
      .map_err(CreateClientError::IpLinkUpError)?;
    if !output.status.success() {
      return Err(CreateClientError::IpLinkUpError(
        std::io::Error::other(format!("ip link set command failed ({:?})",
          output.status.code()))))
    }
    let adapter = Tun {
      tun, read_buf: tokio::sync::Mutex::new([0x0; MTU])
    };
    Ok(adapter)
  }

  #[allow(dead_code)]
  pub fn tun(&self) -> &TokioTun {
    &self.tun
  }

  // TODO: can we return a lock of &[u8] to avoid creating vec?
  pub async fn read(&self) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = self.read_buf.lock().await;
    let nbytes = self.tun.recv(&mut buf[..]).await?;
    Ok(buf[..nbytes].to_vec())
  }

  pub async fn send(&self, datagram: &[u8]) -> Result<usize, std::io::Error> {
    self.tun.send(datagram).await
  }
}
