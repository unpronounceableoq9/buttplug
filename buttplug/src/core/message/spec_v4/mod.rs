use super::device_added::{DeviceAddedV3, DeviceAddedV0, DeviceAddedV1, DeviceAddedV2, DeviceAdded};
use super::device_list::{DeviceListV3, DeviceListV0, DeviceListV1, DeviceListV2, DeviceList};
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

/// Represents all client-to-server messages in v4 of the Buttplug Spec
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
pub enum ButtplugSpecV4ClientMessage {
  // Handshake messages
  RequestServerInfo(RequestServerInfo),
  Ping(Ping),
  // Device enumeration messages
  StartScanning(StartScanning),
  StopScanning(StopScanning),
  RequestDeviceList(RequestDeviceList),
  // Generic commands
  StopAllDevices(StopAllDevices),
  ScalarCmd(ScalarCmd),
  LinearCmd(LinearCmd),
  RotateCmd(RotateCmd),
  StopDeviceCmd(StopDeviceCmd),
  // Raw commands
  RawWriteCmd(RawWriteCmd),
  RawReadCmd(RawReadCmd),
  RawSubscribeCmd(RawSubscribeCmd),
  RawUnsubscribeCmd(RawUnsubscribeCmd),
  // Sensor commands
  SensorReadCmd(SensorReadCmd),
  SensorSubscribeCmd(SensorSubscribeCmd),
  SensorUnsubscribeCmd(SensorUnsubscribeCmd),
}

/// Represents all server-to-client messages in v4 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugServerMessageType,
  FromSpecificButtplugMessage,
  TryFromButtplugServerMessage,
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV4ServerMessage {
  // Status messages
  Ok(Ok),
  Error(Error),
  // Handshake messages
  ServerInfo(ServerInfo),
  // Device enumeration messages
  DeviceList(DeviceList),
  DeviceAdded(DeviceAdded),
  DeviceRemoved(DeviceRemoved),
  ScanningFinished(ScanningFinished),
  // Generic commands
  RawReading(RawReading),
  // Sensor commands
  SensorReading(SensorReading),
}

impl ButtplugMessageFinalizer for ButtplugSpecV4ServerMessage {
  fn finalize(&mut self) {
    match self {
      ButtplugSpecV4ServerMessage::DeviceAdded(da) => da.finalize(),
      ButtplugSpecV4ServerMessage::DeviceList(dl) => dl.finalize(),
      _ => return,
    }
  }
}
