// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use std::ops::RangeInclusive;

use super::*;
use getset::{CopyGetters, Getters, MutGetters};
#[cfg(feature = "serialize-json")]
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};

/// Substructure of device messages, used for actuator information (name, messages supported, etc...)
#[derive(Clone, Debug, PartialEq, Eq, MutGetters, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct ServerActuatorInfo {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Descriptor"))]
  #[getset(get = "pub")]
  descriptor: String,
  #[getset(get = "pub")]
  #[serde(rename = "ActuatorType")]
  actuator_type: ActuatorType,
  #[serde(rename = "StepCount")]
  #[getset(get = "pub")]
  step_range: RangeInclusive<u32>,
}

impl ServerActuatorInfo {
  pub fn new(
    descriptor: &str,
    actuator_type: ActuatorType,
    step_range: &RangeInclusive<u32>
  ) -> Self {
    Self {
      descriptor: descriptor.to_owned(),
      actuator_type,
      step_range: step_range.clone()
    }
  }
}

/// Substructure of device messages, used for actuator information (name, messages supported, etc...)
#[derive(Clone, Debug, PartialEq, Eq, MutGetters, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct ClientActuatorInfo {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Index"))]
  #[getset(get_copy = "pub")]
  index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "Descriptor"))]
  #[getset(get = "pub")]
  descriptor: String,
  #[getset(get = "pub")]
  #[serde(rename = "ActuatorType")]
  actuator_type: ActuatorType,
  #[serde(rename = "StepCount")]
  #[getset(get = "pub")]
  step_count: u32,  
}

impl ClientActuatorInfo {
  pub fn new(
    index: u32,
    descriptor: &str,
    actuator_type: ActuatorType,
    messages: &Vec<ButtplugDeviceMessageType>,
    step_count: u32
  ) -> Self {
    Self {
      index,
      descriptor: descriptor.to_owned(),
      actuator_type,
      step_count
    }
  }
}


fn range_sequence_serialize<S>(
  range_vec: &Vec<RangeInclusive<i32>>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let mut seq = serializer.serialize_seq(Some(range_vec.len()))?;
  for range in range_vec {
    seq.serialize_element(&vec![*range.start(), *range.end()])?;
  }
  seq.end()
}

/// Substructure of device messages, used for sensor information (name, messages supported, etc...)
#[derive(Clone, Debug, PartialEq, Eq, MutGetters, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct SensorInfo {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Index"))]
  #[getset(get_copy = "pub")]
  index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "Descriptor"))]
  #[getset(get = "pub")]
  descriptor: String,
  #[getset(get = "pub")]
  #[serde(rename = "SensorType")]
  sensor_type: SensorType,
  #[getset(get = "pub")]
  #[serde(rename = "SensorRange", serialize_with = "range_sequence_serialize")]
  sensor_range: Vec<RangeInclusive<i32>>,
  #[getset(get = "pub")]
  #[serde(rename = "Readable")]
  readable: bool,
  #[getset(get = "pub")]
  #[serde(rename = "Subscribable")]
  subscribable: bool
}

/// Substructure of device messages, used for attribute information (name, messages supported, etc...)
#[derive(Clone, Debug, PartialEq, Eq, MutGetters, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceMessageInfo {
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

impl From<DeviceAdded> for DeviceMessageInfo {
  fn from(device_added: DeviceAdded) -> Self {
    Self {
      index: device_added.index(),
      name: device_added.name().clone(),
      display_name: device_added.display_name().clone(),
      message_timing_gap: *device_added.message_timing_gap(),
      actuators: device_added.actuators().clone(),
      sensors: device_added.sensors().clone(),
      raw: device_added.raw().clone()
    }
  }
}

/// Substructure of device messages, used for attribute information (name, messages supported, etc...)
#[derive(Clone, Debug, PartialEq, Eq, MutGetters, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceMessageInfoV3 {
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
  #[getset(get = "pub", get_mut = "pub(super)")]
  device_messages: ClientDeviceMessageAttributes,
}

impl DeviceMessageInfoV3 {
  pub fn new(
    device_index: u32,
    device_name: &str,
    device_display_name: &Option<String>,
    device_message_timing_gap: &Option<u32>,
    device_messages: ClientDeviceMessageAttributes,
  ) -> Self {
    Self {
      device_index,
      device_name: device_name.to_owned(),
      device_display_name: device_display_name.clone(),
      device_message_timing_gap: *device_message_timing_gap,
      device_messages,
    }
  }
}

impl From<DeviceMessageInfo> for DeviceMessageInfoV3 {
  fn from(device_info: DeviceMessageInfo) -> Self {
    unimplemented!("Implement this conversion at some point when I have more sanity");
  }
}

impl From<DeviceAddedV3> for DeviceMessageInfoV3 {
  fn from(device_added: DeviceAddedV3) -> Self {
    Self {
      device_index: device_added.device_index(),
      device_name: device_added.device_name().clone(),
      device_display_name: device_added.device_display_name().clone(),
      device_message_timing_gap: *device_added.device_message_timing_gap(),
      device_messages: device_added.device_messages().clone(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceMessageInfoV2 {
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

impl From<DeviceAddedV3> for DeviceMessageInfoV2 {
  fn from(device_added: DeviceAddedV3) -> Self {
    let dmi = DeviceMessageInfoV3::from(device_added);
    DeviceMessageInfoV2::from(dmi)
  }
}

impl From<DeviceAddedV2> for DeviceMessageInfoV2 {
  fn from(device_added: DeviceAddedV2) -> Self {
    // No structural difference, it's all content changes
    Self {
      device_index: device_added.device_index(),
      device_name: device_added.device_name().clone(),
      device_messages: device_added.device_messages().clone(),
    }
  }
}

impl From<DeviceMessageInfoV3> for DeviceMessageInfoV2 {
  fn from(device_message_info: DeviceMessageInfoV3) -> Self {
    // No structural difference, it's all content changes
    Self {
      device_index: device_message_info.device_index,
      device_name: device_message_info.device_name,
      device_messages: device_message_info.device_messages.into(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceMessageInfoV1 {
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

impl From<DeviceAddedV3> for DeviceMessageInfoV1 {
  fn from(device_added: DeviceAddedV3) -> Self {
    let dmi = DeviceMessageInfoV2::from(device_added);
    DeviceMessageInfoV1::from(dmi)
  }
}

impl From<DeviceMessageInfoV2> for DeviceMessageInfoV1 {
  fn from(device_message_info: DeviceMessageInfoV2) -> Self {
    // No structural difference, it's all content changes
    Self {
      device_index: device_message_info.device_index,
      device_name: device_message_info.device_name,
      device_messages: device_message_info.device_messages.into(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct DeviceMessageInfoV0 {
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

impl From<DeviceAddedV3> for DeviceMessageInfoV0 {
  fn from(device_added: DeviceAddedV3) -> Self {
    let dmi = DeviceMessageInfoV3::from(device_added);
    let dmi_v2: DeviceMessageInfoV2 = dmi.into();
    let dmi_v1: DeviceMessageInfoV1 = dmi_v2.into();
    dmi_v1.into()
  }
}

impl From<DeviceMessageInfoV1> for DeviceMessageInfoV0 {
  fn from(device_message_info: DeviceMessageInfoV1) -> Self {
    // Convert to array of message types.
    let mut device_messages: Vec<ButtplugDeviceMessageType> = vec![];

    device_messages.push(ButtplugDeviceMessageType::StopDeviceCmd);
    if device_message_info
      .device_messages
      .single_motor_vibrate_cmd()
      .is_some()
    {
      device_messages.push(ButtplugDeviceMessageType::SingleMotorVibrateCmd);
    }
    if device_message_info
      .device_messages
      .fleshlight_launch_fw12_cmd()
      .is_some()
    {
      device_messages.push(ButtplugDeviceMessageType::FleshlightLaunchFW12Cmd);
    }
    if device_message_info
      .device_messages
      .vorze_a10_cyclone_cmd()
      .is_some()
    {
      device_messages.push(ButtplugDeviceMessageType::VorzeA10CycloneCmd);
    }

    device_messages.sort();

    // SingleMotorVibrateCmd is added as part of the V1 conversion, so we
    // can expect we'll have it here.
    Self {
      device_name: device_message_info.device_name,
      device_index: device_message_info.device_index,
      device_messages,
    }
  }
}
