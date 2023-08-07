// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use super::device_message_info::{DeviceMessageInfoV0, DeviceMessageInfoV1, DeviceMessageInfoV2, DeviceMessageInfoV3, ServerActuatorInfo, SensorInfo, DeviceMessageInfo};
use super::*;

use getset::{CopyGetters, Getters};

#[cfg(feature = "serialize-json")]
use serde::{Deserialize, Serialize};

/// Notification that a device has been found and connected to the server.
#[derive(ButtplugMessage, Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceAdded {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  // DeviceAdded is not considered a device message because it only notifies of existence and is not
  // a command (and goes from server to client), therefore we have to define the getter ourselves.
  #[cfg_attr(feature = "serialize-json", serde(rename = "Index"))]
  #[getset(get_copy = "pub")]
  index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "Name"))]
  #[getset(get = "pub")]
  name: String,
  #[cfg_attr(
    feature = "serialize-json",
    serde(rename = "DisplayName", skip_serializing_if = "Option::is_none")
  )]
  #[getset(get = "pub")]
  display_name: Option<String>,
  #[cfg_attr(
    feature = "serialize-json",
    serde(
      rename = "MessageTimingGap",
      skip_serializing_if = "Option::is_none"
    )
  )]
  #[getset(get = "pub")]
  message_timing_gap: Option<u32>,
  #[cfg_attr(
    feature = "serialize-json",
    serde(
      rename = "Actuators",
      skip_serializing_if = "Option::is_none"
    )
  )]
  #[getset(get = "pub")]
  actuators: Option<Vec<ServerActuatorInfo>>,
  #[cfg_attr(
    feature = "serialize-json",
    serde(
      rename = "Sensors",
      skip_serializing_if = "Option::is_none"
    )
  )]
  #[getset(get = "pub")]
  sensors: Option<Vec<SensorInfo>>,
  #[cfg_attr(
    feature = "serialize-json",
    serde(
      rename = "Raw",
      skip_serializing_if = "Option::is_none"
    )
  )]
  #[getset(get = "pub")]
  raw: Option<Vec<Endpoint>>,
}

impl DeviceAdded {
  pub fn new(
    index: u32,
    name: &str,
    display_name: &Option<String>,
    message_timing_gap: &Option<u32>,
    actuators: &Option<Vec<ServerActuatorInfo>>,
    sensors: &Option<Vec<SensorInfo>>,
    raw: &Option<Vec<Endpoint>>
  ) -> Self {
    let mut obj = Self {
      id: 0,
      index,
      name: name.to_string(),
      display_name: display_name.clone(),
      message_timing_gap: *message_timing_gap,
      actuators: actuators.clone(),
      sensors: sensors.clone(),
      raw: raw.clone()
    };
    obj.finalize();
    obj
  }
}

impl ButtplugMessageValidator for DeviceAdded {
  fn is_valid(&self) -> Result<(), ButtplugMessageError> {
    self.is_system_id(self.id)
  }
}

impl ButtplugMessageFinalizer for DeviceAdded {
  fn finalize(&mut self) {
  }
}


/// Notification that a device has been found and connected to the server.
#[derive(ButtplugMessage, Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceAddedV3 {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  // DeviceAdded is not considered a device message because it only notifies of existence and is not
  // a command (and goes from server to client), therefore we have to define the getter ourselves.
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceIndex"))]
  #[getset(get_copy = "pub")]
  device_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceName"))]
  #[getset(get = "pub")]
  device_name: String,
  #[cfg_attr(
    feature = "serialize-json",
    serde(rename = "DeviceDisplayName", skip_serializing_if = "Option::is_none")
  )]
  #[getset(get = "pub")]
  device_display_name: Option<String>,
  #[cfg_attr(
    feature = "serialize-json",
    serde(
      rename = "DeviceMessageTimingGap",
      skip_serializing_if = "Option::is_none"
    )
  )]
  #[getset(get = "pub")]
  device_message_timing_gap: Option<u32>,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceMessages"))]
  #[getset(get = "pub")]
  device_messages: ClientDeviceMessageAttributes,
}

impl DeviceAddedV3 {
  pub fn new(
    device_index: u32,
    device_name: &str,
    device_display_name: &Option<String>,
    device_message_timing_gap: &Option<u32>,
    device_messages: &ClientDeviceMessageAttributes,
  ) -> Self {
    let mut obj = Self {
      id: 0,
      device_index,
      device_name: device_name.to_string(),
      device_display_name: device_display_name.clone(),
      device_message_timing_gap: *device_message_timing_gap,
      device_messages: device_messages.clone(),
    };
    obj.finalize();
    obj
  }
}

impl ButtplugMessageValidator for DeviceAddedV3 {
  fn is_valid(&self) -> Result<(), ButtplugMessageError> {
    self.is_system_id(self.id)
  }
}

impl ButtplugMessageFinalizer for DeviceAddedV3 {
  fn finalize(&mut self) {
    self.device_messages.finalize();
  }
}


impl From<DeviceAdded> for DeviceAddedV3 {
  fn from(msg: DeviceAdded) -> Self {
    let id = msg.id();
    let dmi = DeviceMessageInfo::from(msg);
    let dmiv3 = DeviceMessageInfoV3::from(dmi);

    Self {
      id,
      device_index: dmiv3.device_index(),
      device_display_name: dmiv3.device_display_name().clone(),
      device_name: dmiv3.device_name().clone(),
      device_messages: dmiv3.device_messages().clone(),
      device_message_timing_gap: *dmiv3.device_message_timing_gap()
    }
  }
}

#[derive(ButtplugMessage, Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceAddedV2 {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceIndex"))]
  #[getset(get_copy = "pub")]
  device_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceName"))]
  #[getset(get = "pub")]
  device_name: String,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceMessages"))]
  #[getset(get = "pub")]
  device_messages: ClientDeviceMessageAttributesV2,
}

impl From<DeviceAdded> for DeviceAddedV2 {
  fn from(msg: DeviceAdded) -> Self {
    let id = msg.id();
    let dmi = DeviceMessageInfo::from(msg);
    let dmiv3 = DeviceMessageInfoV3::from(dmi);
    let dmiv2 = DeviceMessageInfoV2::from(dmiv3);

    Self {
      id,
      device_index: dmiv2.device_index(),
      device_name: dmiv2.device_name().clone(),
      device_messages: dmiv2.device_messages().clone(),
    }
  }
}

impl ButtplugMessageValidator for DeviceAddedV2 {
  fn is_valid(&self) -> Result<(), ButtplugMessageError> {
    self.is_system_id(self.id)
  }
}

impl ButtplugMessageFinalizer for DeviceAddedV2 {
}

#[derive(ButtplugMessage, Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceAddedV1 {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceIndex"))]
  #[getset(get_copy = "pub")]
  device_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceName"))]
  #[getset(get = "pub")]
  device_name: String,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceMessages"))]
  #[getset(get = "pub")]
  device_messages: ClientDeviceMessageAttributesV1,
}

impl From<DeviceAdded> for DeviceAddedV1 {
  fn from(msg: DeviceAdded) -> Self {
    let id = msg.id();
    let dmi = DeviceMessageInfoV3::from(DeviceMessageInfo::from(msg));
    let dmiv2 = DeviceMessageInfoV2::from(dmi);
    let dmiv1 = DeviceMessageInfoV1::from(dmiv2);

    Self {
      id,
      device_index: dmiv1.device_index(),
      device_name: dmiv1.device_name().clone(),
      device_messages: dmiv1.device_messages().clone(),
    }
  }
}

impl ButtplugMessageValidator for DeviceAddedV1 {
  fn is_valid(&self) -> Result<(), ButtplugMessageError> {
    self.is_system_id(self.id)
  }
}

impl ButtplugMessageFinalizer for DeviceAddedV1 {
}

#[derive(Default, ButtplugMessage, Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceAddedV0 {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceIndex"))]
  #[getset(get_copy = "pub")]
  device_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceName"))]
  #[getset(get = "pub")]
  device_name: String,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceMessages"))]
  #[getset(get = "pub")]
  device_messages: Vec<ButtplugDeviceMessageType>,
}

impl From<DeviceAdded> for DeviceAddedV0 {
  fn from(msg: DeviceAdded) -> Self {
    let id = msg.id();
    let dmi = DeviceMessageInfoV3::from(DeviceMessageInfo::from(msg));
    let dmiv2 = DeviceMessageInfoV2::from(dmi);
    let dmiv1 = DeviceMessageInfoV1::from(dmiv2);
    let dmiv0 = DeviceMessageInfoV0::from(dmiv1);

    Self {
      id,
      device_index: dmiv0.device_index(),
      device_name: dmiv0.device_name().clone(),
      device_messages: dmiv0.device_messages().clone(),
    }
  }
}

impl ButtplugMessageValidator for DeviceAddedV0 {
  fn is_valid(&self) -> Result<(), ButtplugMessageError> {
    self.is_system_id(self.id)
  }
}

impl ButtplugMessageFinalizer for DeviceAddedV0 {
}

// TODO Test repeated message type in attributes in JSON
