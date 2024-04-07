use super::device_added::DeviceAddedV0;
use super::device_list::DeviceListV0;
use super::device_removed::DeviceRemoved;
use super::error::ErrorV0;
use super::fleshlight_launch_fw12_cmd::FleshlightLaunchFW12Cmd;
use super::kiiroo_cmd::KiirooCmd;
use super::log::Log;
use super::lovense_cmd::LovenseCmd;
use super::ok::Ok;
use super::ping::Ping;
use super::request_device_list::RequestDeviceList;
use super::request_log::RequestLog;
use super::request_server_info::RequestServerInfo;
use super::scanning_finished::ScanningFinished;
use super::server_info::ServerInfoV0;
use super::single_motor_vibrate_cmd::SingleMotorVibrateCmd;
use super::start_scanning::StartScanning;
use super::stop_all_devices::StopAllDevices;
use super::stop_device_cmd::StopDeviceCmd;
use super::stop_scanning::StopScanning;
use super::vorze_a10_cyclone_cmd::VorzeA10CycloneCmd;

use super::{ButtplugMessage, ButtplugClientMessage, ButtplugMessageValidator, ButtplugClientMessageType,
ButtplugMessageFinalizer, ButtplugServerMessageType, ButtplugServerMessage};

use crate::core::errors::ButtplugMessageError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Represents all client-to-server messages in v0 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugClientMessageType,
  ButtplugMessageFinalizer,
  TryFromButtplugClientMessage,
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV0ClientMessage {
  RequestLog(RequestLog),
  Ping(Ping),
  // Handshake messages
  RequestServerInfo(RequestServerInfo),
  // Device enumeration messages
  StartScanning(StartScanning),
  StopScanning(StopScanning),
  RequestDeviceList(RequestDeviceList),
  // Generic commands
  StopAllDevices(StopAllDevices),
  StopDeviceCmd(StopDeviceCmd),
  // Deprecated generic commands
  SingleMotorVibrateCmd(SingleMotorVibrateCmd),
  // Deprecated device specific commands
  FleshlightLaunchFW12Cmd(FleshlightLaunchFW12Cmd),
  LovenseCmd(LovenseCmd),
  KiirooCmd(KiirooCmd),
  VorzeA10CycloneCmd(VorzeA10CycloneCmd),
}

/// Represents all server-to-client messages in v0 of the Buttplug Spec
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  ButtplugMessage,
  ButtplugMessageValidator,
  ButtplugMessageFinalizer,
  ButtplugServerMessageType,
)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub enum ButtplugSpecV0ServerMessage {
  // Status messages
  Ok(Ok),
  Error(ErrorV0),
  Log(Log),
  // Handshake messages
  ServerInfo(ServerInfoV0),
  // Device enumeration messages
  DeviceList(DeviceListV0),
  DeviceAdded(DeviceAddedV0),
  DeviceRemoved(DeviceRemoved),
  ScanningFinished(ScanningFinished),
}

// This was implementated as a derive, but for some reason the .into() calls
// wouldn't work correctly when used as a device. If the actual implementation
// is here, things work fine. Luckily it won't ever be changed much.
impl TryFrom<ButtplugServerMessage> for ButtplugSpecV0ServerMessage {
  type Error = ButtplugMessageError;
  fn try_from(msg: ButtplugServerMessage) -> Result<Self, ButtplugMessageError> {
    match msg {
      ButtplugServerMessage::Ok(msg) => Ok(ButtplugSpecV0ServerMessage::Ok(msg)),
      ButtplugServerMessage::Error(msg) => Ok(ButtplugSpecV0ServerMessage::Error(msg.into())),
      ButtplugServerMessage::Log(msg) => Ok(ButtplugSpecV0ServerMessage::Log(msg)),
      ButtplugServerMessage::ServerInfo(msg) => {
        Ok(ButtplugSpecV0ServerMessage::ServerInfo(msg.into()))
      }
      ButtplugServerMessage::DeviceList(msg) => {
        Ok(ButtplugSpecV0ServerMessage::DeviceList(msg.into()))
      }
      ButtplugServerMessage::DeviceAdded(msg) => {
        Ok(ButtplugSpecV0ServerMessage::DeviceAdded(msg.into()))
      }
      ButtplugServerMessage::DeviceRemoved(msg) => {
        Ok(ButtplugSpecV0ServerMessage::DeviceRemoved(msg))
      }
      ButtplugServerMessage::ScanningFinished(msg) => {
        Ok(ButtplugSpecV0ServerMessage::ScanningFinished(msg))
      }
      _ => Err(ButtplugMessageError::VersionError(
        "ButtplugServerMessage".to_owned(),
        format!("{:?}", msg),
        "ButtplugSpecV0ServerMessage".to_owned(),
      )),
    }
  }
}