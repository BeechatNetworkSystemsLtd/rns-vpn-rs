//! Reticulum VPN client

use std::{fs, process};

use clap::Parser;
use ed25519_dalek;
use env_logger;
use log;
use pem;
use reticulum::destination::DestinationName;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::kaonic::kaonic_grpc::KaonicGrpc;
use reticulum::iface::kaonic::RadioConfig;
use reticulum::iface::udp::UdpInterface;
use reticulum::transport::{Transport, TransportConfig};
use serde::{Deserialize, Serialize};
use tokio;
use x25519_dalek;

use rns_vpn;

const DEFAULT_CONFIG_PATH: &str = "Config.toml";

/// Choose one of `-a <kaonic-grpc-address>` or
/// `-p <udp-listen-port> -f <udp-forward-address>`
#[derive(Parser)]
#[command(name = "Reticulum VPN Client", version)]
pub struct Command {
  /// Reticulum UDP listen port number
  #[arg(short = 'p', long, group = "transport",
    required_unless_present = "kaonic_grpc_address")]
  pub udp_listen_port: Option<u16>,
  /// Reticulum UDP forward link address
  #[arg(short = 'f', long, requires = "udp_listen_port")]
  pub udp_forward_address: Option<std::net::SocketAddr>,
  /// Reticulum Kaonic gRPC address
  #[arg(short = 'a', long, group = "transport",
    required_unless_present = "udp_listen_port")]
  pub kaonic_grpc_address: Option<String>,
  /// [Optional] Reticulum private ID from name string
  #[arg(short, long)]
  pub id_string: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct Config {
  #[serde(flatten)]
  pub vpn_config: rns_vpn::Config,
  pub kaonic_radio_config: Option<RadioConfig>
}

#[tokio::main]
async fn main() -> Result<(), process::ExitCode> {
  // parse command line args
  let cmd = Command::parse();
  // load config
  let config: Config = {
    let path = if let Ok(path) = std::env::var("RNS_VPN_CONFIG_PATH") {
      path
    } else {
      DEFAULT_CONFIG_PATH.to_string()
    };
    let s = fs::read_to_string(path).unwrap();
    toml::from_str(&s).unwrap()
  };
  // init logging
  env_logger::Builder::new().filter_level(log::LevelFilter::Info).parse_default_env()
    .init();
  // start reticulum
  log::info!("starting reticulum");
  let id = if let Some(name) = cmd.id_string {
    log::info!("using identity string to create reticulum private identity: {name:?}");
    PrivateIdentity::new_from_name(&name)
  } else {
    log::info!("loading reticulum private identity parameters");
    let private_key = {
      let path = std::env::var("RNS_VPN_PRIVKEY_PATH").map_err(|err|{
        log::error!("env variable RNS_VPN_PRIVKEY_PATH not found: {err:?}");
        process::ExitCode::FAILURE
      })?;
      log::info!("loading privkey: {path}");
      let pem_data = fs::read(&path).map_err(|err|{
        log::error!("failed to read privkey {path}: {err:?}");
        process::ExitCode::FAILURE
      })?;
      let pem = pem::parse(pem_data).map_err(|err|{
        log::error!("failed to parse privkey {path}: {err:?}");
        process::ExitCode::FAILURE
      })?;
      let pem_bytes: [u8; 32] = pem.contents()[pem.contents().len()-32..].try_into()
        .map_err(|err|{
          log::error!("invalid privkey bytes: {err:?}");
          process::ExitCode::FAILURE
        })?;
      x25519_dalek::StaticSecret::from(pem_bytes)
    };
    let sign_key = {
      use ed25519_dalek::pkcs8::DecodePrivateKey;
      let path = std::env::var("RNS_VPN_SIGNKEY_PATH").map_err(|err|{
        log::error!("env variable RNS_VPN_SIGNKEY_PATH not found: {err:?}");
        process::ExitCode::FAILURE
      })?;
      log::info!("loading signkey: {path}");
      ed25519_dalek::SigningKey::read_pkcs8_pem_file(&path).map_err(|err|{
        log::error!("failed to parse signkey {path}: {err:?}");
        process::ExitCode::FAILURE
      })?
    };
    PrivateIdentity::new(private_key, sign_key)
  };
  let mut transport = Transport::new(TransportConfig::new("server", &id, true));
  if let Some (port) = cmd.udp_listen_port {
    // udp
    let forward = cmd.udp_forward_address.unwrap();
    log::info!("creating RNS UDP interface with listen port {port} and forward IP \
      {forward}");
    let _ = transport.iface_manager().lock().await.spawn(
      UdpInterface::new(format!("0.0.0.0:{}", port), Some(forward.to_string())),
      UdpInterface::spawn);
  } else {
    // kaonic
    let address = cmd.kaonic_grpc_address.unwrap();
    let radio_config = config.kaonic_radio_config.ok_or_else(||{
      log::error!("config is missing kaonic_radio_config");
      process::ExitCode::FAILURE
    })?;
    log::info!("creating RNS kaonic interface with kaonic grpc address {address}");
    let _ = transport.iface_manager().lock().await.spawn(
      KaonicGrpc::new(address, radio_config, None), KaonicGrpc::spawn);
  }
  // create in destination
  let in_destination = transport
    .add_destination(id, DestinationName::new("rns_vpn", "client")).await;
  let in_destination_hash = in_destination.lock().await.desc.address_hash;
  log::info!("created destination: {}",
    format!("{}", in_destination_hash).trim_matches('/'));
  // client
  let client = match rns_vpn::Client::new(config.vpn_config, in_destination).await {
    Ok(client) => client,
    Err(err) => match err {
      rns_vpn::CreateClientError::RiptunError(riptun::Error::Unix {
        source: nix::errno::Errno::EPERM
      }) => {
        log::error!("EPERM error creating TUN interface: \
          need to run with root permissions");
        return Err(process::ExitCode::FAILURE)
      }
      _ => {
        log::error!("error creating VPN client: {:?}", err);
        return Err(process::ExitCode::FAILURE)
      }
    }
  };
  // run
  client.run(transport).await;
  log::info!("server exit");
  Ok(())
}
