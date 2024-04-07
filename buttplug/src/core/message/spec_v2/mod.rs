use super::battery_level_cmd::BatteryLevelCmd;
use super::battery_level_reading::BatteryLevelReading;
use super::device_added::DeviceAddedV2;
use super::device_list::DeviceListV2;
use super::device_removed::DeviceRemoved;
use super::error::{Error, };
use super::linear_cmd::{LinearCmd, };
use super::ok::Ok;
use super::ping::Ping;
use super::raw_read_cmd::RawReadCmd;
use super::raw_reading::RawReading;
use super::raw_subscribe_cmd::RawSubscribeCmd;
use super::raw_unsubscribe_cmd::RawUnsubscribeCmd;
use super::raw_write_cmd::RawWriteCmd;
use super::request_device_list::RequestDeviceList;
use super::request_server_info::RequestServerInfo;
use super::rotate_cmd::{RotateCmd, };
use super::rssi_level_cmd::RSSILevelCmd;
use super::rssi_level_reading::RSSILevelReading;
use super::scanning_finished::ScanningFinished;
use super::server_info::{ServerInfo, };
use super::start_scanning::StartScanning;
use super::stop_all_devices::StopAllDevices;
use super::stop_device_cmd::StopDeviceCmd;
use super::stop_scanning::StopScanning;
use super::vibrate_cmd::{VibrateCmd, };

use super::{ButtplugMessage, ButtplugClientMessage, ButtplugMessageValidator, ButtplugClientMessageType,
  ButtplugMessageFinalizer, ButtplugServerMessageType, ButtplugServerMessage};

use crate::core::errors::ButtplugMessageError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;


/// Represents all client-to-server messages in v2 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugClientMessageType,
  ButtplugMessageFinalizer,
  FromSpecificButtplugMessage,
  TryFromButtplugClientMessage,
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV2ClientMessage {
  // Handshake messages
  RequestServerInfo(RequestServerInfo),
  Ping(Ping),
  // Device enumeration messages
  StartScanning(StartScanning),
  StopScanning(StopScanning),
  RequestDeviceList(RequestDeviceList),
  // Generic commands
  StopAllDevices(StopAllDevices),
  VibrateCmd(VibrateCmd),
  LinearCmd(LinearCmd),
  RotateCmd(RotateCmd),
  RawWriteCmd(RawWriteCmd),
  RawReadCmd(RawReadCmd),
  StopDeviceCmd(StopDeviceCmd),
  RawSubscribeCmd(RawSubscribeCmd),
  RawUnsubscribeCmd(RawUnsubscribeCmd),
  // Sensor commands
  BatteryLevelCmd(BatteryLevelCmd),
  RSSILevelCmd(RSSILevelCmd),
}

/// Represents all server-to-client messages in v2 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugMessageFinalizer,
  ButtplugServerMessageType,
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV2ServerMessage {
  // Status messages
  Ok(Ok),
  Error(Error),
  // Handshake messages
  ServerInfo(ServerInfo),
  // Device enumeration messages
  DeviceList(DeviceListV2),
  DeviceAdded(DeviceAddedV2),
  DeviceRemoved(DeviceRemoved),
  ScanningFinished(ScanningFinished),
  // Generic commands
  RawReading(RawReading),
  // Sensor commands
  BatteryLevelReading(BatteryLevelReading),
  RSSILevelReading(RSSILevelReading),
}

// This was implementated as a derive, but for some reason the .into() calls
// wouldn't work correctly when used as a device. If the actual implementation
// is here, things work fine. Luckily it won't ever be changed much.
impl TryFrom<ButtplugServerMessage> for ButtplugSpecV2ServerMessage {
  type Error = ButtplugMessageError;
  fn try_from(msg: ButtplugServerMessage) -> Result<Self, ButtplugMessageError> {
    match msg {
      ButtplugServerMessage::Ok(msg) => Ok(ButtplugSpecV2ServerMessage::Ok(msg)),
      ButtplugServerMessage::Error(msg) => Ok(ButtplugSpecV2ServerMessage::Error(msg)),
      ButtplugServerMessage::ServerInfo(msg) => Ok(ButtplugSpecV2ServerMessage::ServerInfo(msg)),
      ButtplugServerMessage::DeviceList(msg) => {
        Ok(ButtplugSpecV2ServerMessage::DeviceList(msg.into()))
      }
      ButtplugServerMessage::DeviceAdded(msg) => {
        Ok(ButtplugSpecV2ServerMessage::DeviceAdded(msg.into()))
      }
      ButtplugServerMessage::DeviceRemoved(msg) => {
        Ok(ButtplugSpecV2ServerMessage::DeviceRemoved(msg))
      }
      ButtplugServerMessage::ScanningFinished(msg) => {
        Ok(ButtplugSpecV2ServerMessage::ScanningFinished(msg))
      }
      _ => Err(ButtplugMessageError::VersionError(
        "ButtplugServerMessage".to_owned(),
        format!("{:?}", msg),
        "ButtplugSpecV2ServerMessage".to_owned(),
      )),
    }
  }
}
