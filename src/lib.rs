use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use cidr;
use etherparse;
use ipnet::IpNet;
use log;
use riptun::TokioTun;
use serde::{Deserialize, Serialize};
use tokio;
use tokio::sync::Mutex;

use reticulum::destination::SingleInputDestination;
use reticulum::destination::link::{LinkEvent, LinkId};
use reticulum::hash::AddressHash;
use reticulum::transport::Transport;

// TODO: config?
const TUN_NQUEUES : usize = 1;
const MTU: usize = 1500;

const fn default_announce_freq_secs() -> u32 { 1 }

#[derive(Deserialize, Serialize)]
pub struct Config {
  pub network: cidr::Ipv4Cidr,
  /// List of peer destination hashes
  pub peers: Vec<String>,
  #[serde(default = "default_announce_freq_secs")]
  pub announce_freq_secs: u32
}

pub struct Client {
  config: Config,
  in_destination: Arc<Mutex<SingleInputDestination>>,
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
  pub async fn new(config: Config, in_destination: Arc<Mutex<SingleInputDestination>>)
    -> Result<Self, CreateClientError>
  {
    if config.peers.is_empty() {
      log::warn!("no peers configured");
      return Err(CreateClientError::ConfigError(
          "no peers: add at least one peer destination to configuration".to_string()))
    }
    let destination_hash = in_destination.lock().await.desc.address_hash;
    let vpn_ip = destination_to_ip(destination_hash, config.network);
    let tun = Tun::new(vpn_ip)?;
    Ok(Client { config, in_destination, tun })
  }

  pub async fn run(&self, transport: Transport) {
    let in_destination_hash = self.in_destination.lock().await.desc.address_hash;
    // set up peer map
    let peer_map = {
      let mut peer_map = BTreeMap::<IpAddr, Peer>::new();
      for dest in self.config.peers.iter() {
        let dest = match AddressHash::new_from_hex_string(dest.as_str()) {
          Ok(dest) => dest,
          Err(err) => {
            log::error!("error parsing peer destination hash: {err:?}");
            return
          }
        };
        let peer = Peer { dest, link_id: None, link_active: false };
        let ip = destination_to_ip(dest, self.config.network);
        assert!(peer_map.insert(ip.addr(), peer).is_none());
      }
      tokio::sync::Mutex::new(peer_map)
    };
    // send announces
    let announce_loop = async || loop {
      transport.send_announce(&self.in_destination, None).await;
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
      loop {
        match in_link_events.recv().await {
          Ok(link_event) => match link_event.event {
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
          Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
            log::debug!("recv in link event lagged: {n}");
          }
          Err(err) => {
            log::error!("recv in link event error: {err:?}");
            break
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

fn destination_to_ip(destination: AddressHash, prefix: cidr::Ipv4Cidr) -> IpNet {
  let n = u32::from_be_bytes((&destination.as_slice()[12..16]).try_into().unwrap());
  let network_bits = prefix.mask().to_bits();
  let host_bits = (!network_bits) & n;
  let addr = Ipv4Addr::from_bits(prefix.first_address().to_bits() | host_bits);
  IpNet::new(IpAddr::V4(addr), network_bits.count_ones() as u8).unwrap()
}

#[cfg(test)]
mod tests {
  use reticulum::hash::AddressHash;
  use super::*;
  #[test]
  fn dest_to_ip() {
    use std::str::FromStr;
    let destination =
      AddressHash::new_from_hex_string("fb08aff16ec6f5ccf0d3eb179028e9c3").unwrap();
    // 0xe9 = 233, 0xc3 = 195
    let prefix = cidr::Ipv4Cidr::from_str("10.1.0.0/16").unwrap();
    let ip = destination_to_ip(destination, prefix);
    assert_eq!(ip.addr(), std::net::IpAddr::from_str("10.1.233.195").unwrap());
  }
}
