// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use super::lovense_connect_service_comm_manager::LovenseServiceToyInfo;
use crate::{
  core::{
    errors::{ButtplugDeviceError, ButtplugError},
    messages::{Endpoint, RawReading},
    ButtplugResultFuture,
  },
  server::device::{
    configuration::{ProtocolCommunicationSpecifier, LovenseConnectServiceSpecifier, ProtocolDeviceConfiguration},
    hardware::{
    HardwareEvent,
    HardwareCreator,
    Hardware,
    HardwareInternal,
    HardwareReadCmd,
    HardwareSubscribeCmd,
    HardwareUnsubscribeCmd,
    HardwareWriteCmd,
    },
  },
  util::async_manager,
};
use async_trait::async_trait;
use futures::future::{self, BoxFuture};
use futures_timer::Delay;
use std::{
  fmt::{self, Debug},
  sync::Arc,
  time::Duration,
};
use tokio::sync::{broadcast, RwLock};

pub struct LovenseServiceHardwareCreator {
  http_host: String,
  toy_info: Arc<RwLock<LovenseServiceToyInfo>>,
}

impl LovenseServiceHardwareCreator {
  pub(super) fn new(http_host: &str, toy_info: Arc<RwLock<LovenseServiceToyInfo>>) -> Self {
    debug!("Emitting a new lovense service device impl creator!");
    Self {
      http_host: http_host.to_owned(),
      toy_info,
    }
  }
}

impl Debug for LovenseServiceHardwareCreator {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("LovenseServiceHardwareCreator").finish()
  }
}

#[async_trait]
impl HardwareCreator for LovenseServiceHardwareCreator {
  fn specifier(&self) -> ProtocolCommunicationSpecifier {
    ProtocolCommunicationSpecifier::LovenseConnectService(LovenseConnectServiceSpecifier::default())
  }

  async fn try_create_hardware(
    &mut self,
    _protocol: ProtocolDeviceConfiguration,
  ) -> Result<Hardware, ButtplugError> {
    let toy_info = self.toy_info.read().await;

    let hardware_internal =
      LovenseServiceHardware::new(&self.http_host, self.toy_info.clone(), &toy_info.id);
    let hardware = Hardware::new(
      &toy_info.name,
      &toy_info.id,
      &[Endpoint::Tx],
      Box::new(hardware_internal),
    );
    Ok(hardware)
  }
}

#[derive(Clone, Debug)]
pub struct LovenseServiceHardware {
  event_sender: broadcast::Sender<HardwareEvent>,
  http_host: String,
  toy_info: Arc<RwLock<LovenseServiceToyInfo>>,
}

impl LovenseServiceHardware {
  fn new(http_host: &str, toy_info: Arc<RwLock<LovenseServiceToyInfo>>, toy_id: &str) -> Self {
    let (device_event_sender, _) = broadcast::channel(256);
    let sender_clone = device_event_sender.clone();
    let toy_id = toy_id.to_owned();
    let toy_info_clone = toy_info.clone();
    async_manager::spawn(async move {
      while toy_info_clone.read().await.connected {
        Delay::new(Duration::from_secs(1)).await;
      }
      let _ = sender_clone.send(HardwareEvent::Disconnected(toy_id));
      info!("Exiting lovense service device connection check loop.");
    });
    Self {
      event_sender: device_event_sender,
      http_host: http_host.to_owned(),
      toy_info,
    }
  }
}

impl HardwareInternal for LovenseServiceHardware {
  fn event_stream(&self) -> broadcast::Receiver<HardwareEvent> {
    self.event_sender.subscribe()
  }

  fn connected(&self) -> bool {
    true
  }

  fn disconnect(&self) -> ButtplugResultFuture {
    Box::pin(future::ready(Ok(())))
  }

  // Assume the only thing we'll read is battery.
  fn read_value(
    &self,
    _msg: HardwareReadCmd,
  ) -> BoxFuture<'static, Result<RawReading, ButtplugError>> {
    let toy_info = self.toy_info.clone();
    Box::pin(async move {
      let battery_level = toy_info.read().await.battery.clamp(0, 100) as u8;
      Ok(RawReading::new(0, Endpoint::Rx, vec![battery_level]))
    })
  }

  fn write_value(&self, msg: HardwareWriteCmd) -> ButtplugResultFuture {
    let command_url = format!(
      "{}/{}",
      self.http_host,
      std::str::from_utf8(&msg.data)
        .expect("We build this in the protocol then have to serialize to [u8], but it's a string.")
    );
    Box::pin(async move {
      match reqwest::get(command_url).await {
        Ok(_) => Ok(()),
        Err(err) => {
          error!("Got http error: {}", err);
          Err(ButtplugDeviceError::UnhandledCommand(err.to_string()).into())
        }
      }
    })
  }

  fn subscribe(&self, _msg: HardwareSubscribeCmd) -> ButtplugResultFuture {
    panic!("We should never get here!");
  }

  fn unsubscribe(&self, _msg: HardwareUnsubscribeCmd) -> ButtplugResultFuture {
    panic!("We should never get here!");
  }
}