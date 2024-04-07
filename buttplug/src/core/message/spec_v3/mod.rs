use super::device_added::{DeviceAddedV3, DeviceAddedV0, DeviceAddedV1, DeviceAddedV2, DeviceAdded};
use super::device_list::{DeviceListV3, DeviceListV0, DeviceListV1, DeviceListV2, DeviceList};
use super::device_message_info::DeviceMessageInfoV0;
use super::device_removed::DeviceRemoved;
use super::endpoint::Endpoint;
use super::error::{Error, ErrorCode, ErrorV0};
use super::fleshlight_launch_fw12_cmd::FleshlightLaunchFW12Cmd;
use super::kiiroo_cmd::KiirooCmd;
use super::linear_cmd::{LinearCmd, VectorSubcommand};
use super::log_level::LogLevel;
use super::lovense_cmd::LovenseCmd;
use super::ok::Ok;
use super::ping::Ping;
use super::raw_read_cmd::RawReadCmd;
use super::raw_reading::RawReading;
use super::raw_subscribe_cmd::RawSubscribeCmd;
use super::raw_unsubscribe_cmd::RawUnsubscribeCmd;
use super::raw_write_cmd::RawWriteCmd;
use super::request_device_list::RequestDeviceList;
use super::request_log::RequestLog;
use super::request_server_info::RequestServerInfo;
use super::rotate_cmd::{RotateCmd, RotationSubcommand};
use super::rssi_level_cmd::RSSILevelCmd;
use super::rssi_level_reading::RSSILevelReading;
use super::scalar_cmd::{ScalarCmd, ScalarSubcommand};
use super::scanning_finished::ScanningFinished;
use super::sensor_read_cmd::SensorReadCmd;
use super::sensor_reading::SensorReading;
use super::sensor_subscribe_cmd::SensorSubscribeCmd;
use super::sensor_unsubscribe_cmd::SensorUnsubscribeCmd;
use super::server_info::{ServerInfo, ServerInfoV0};
use super::single_motor_vibrate_cmd::SingleMotorVibrateCmd;
use super::start_scanning::StartScanning;
use super::stop_all_devices::StopAllDevices;
use super::stop_device_cmd::StopDeviceCmd;
use super::stop_scanning::StopScanning;
use super::test::Test;
use super::vibrate_cmd::{VibrateCmd, VibrateSubcommand};
use super::vorze_a10_cyclone_cmd::VorzeA10CycloneCmd;

use super::{ButtplugMessage, ButtplugClientMessage, ButtplugMessageValidator, ButtplugClientMessageType,
  ButtplugMessageFinalizer, ButtplugServerMessageType, ButtplugServerMessage};

use crate::core::errors::ButtplugMessageError;
use serde::{Deserialize, Serialize};
#[cfg(feature = "serialize-json")]
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::Ordering;
use std::convert::TryFrom;


/// Represents all client-to-server messages in v3 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugClientMessageType,
  ButtplugMessageFinalizer,
  FromSpecificButtplugMessage,
  TryFromButtplugClientMessage
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV3ClientMessage {
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
  ScalarCmd(ScalarCmd),
  // Sensor commands
  SensorReadCmd(SensorReadCmd),
  SensorSubscribeCmd(SensorSubscribeCmd),
  SensorUnsubscribeCmd(SensorUnsubscribeCmd),
}

/// Represents all server-to-client messages in v3 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugServerMessageType,
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV3ServerMessage {
  // Status messages
  Ok(Ok),
  Error(Error),
  // Handshake messages
  ServerInfo(ServerInfo),
  // Device enumeration messages
  DeviceList(DeviceListV3),
  DeviceAdded(DeviceAddedV3),
  DeviceRemoved(DeviceRemoved),
  ScanningFinished(ScanningFinished),
  // Generic commands
  RawReading(RawReading),
  // Sensor commands
  SensorReading(SensorReading),
}

impl ButtplugMessageFinalizer for ButtplugSpecV3ServerMessage {
  fn finalize(&mut self) {
    match self {
      ButtplugSpecV3ServerMessage::DeviceAdded(da) => da.finalize(),
      ButtplugSpecV3ServerMessage::DeviceList(dl) => dl.finalize(),
      _ => return,
    }
  }
}

// This was implementated as a derive, but for some reason the .into() calls
// wouldn't work correctly when used as a device. If the actual implementation
// is here, things work fine. Luckily it won't ever be changed much.
impl TryFrom<ButtplugServerMessage> for ButtplugSpecV3ServerMessage {
  type Error = ButtplugMessageError;
  fn try_from(msg: ButtplugServerMessage) -> Result<Self, ButtplugMessageError> {
    match msg {
      ButtplugServerMessage::Ok(msg) => Ok(ButtplugSpecV3ServerMessage::Ok(msg)),
      ButtplugServerMessage::Error(msg) => Ok(ButtplugSpecV3ServerMessage::Error(msg)),
      ButtplugServerMessage::ServerInfo(msg) => Ok(ButtplugSpecV3ServerMessage::ServerInfo(msg)),
      ButtplugServerMessage::DeviceList(msg) => {
        Ok(ButtplugSpecV3ServerMessage::DeviceList(msg.into()))
      }
      ButtplugServerMessage::DeviceAdded(msg) => {
        Ok(ButtplugSpecV3ServerMessage::DeviceAdded(msg.into()))
      }
      ButtplugServerMessage::DeviceRemoved(msg) => {
        Ok(ButtplugSpecV3ServerMessage::DeviceRemoved(msg))
      }
      ButtplugServerMessage::ScanningFinished(msg) => {
        Ok(ButtplugSpecV3ServerMessage::ScanningFinished(msg))
      }
      _ => Err(ButtplugMessageError::VersionError(
        "ButtplugServerMessage".to_owned(),
        format!("{:?}", msg),
        "ButtplugSpecV3ServerMessage".to_owned(),
      )),
    }
  }
}
