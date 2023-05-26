// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use std::sync::Arc;

use crate::core::{errors::ButtplugDeviceError, message::{ActuatorType, ClientGenericDeviceMessageAttributes, ClientDeviceMessageAttributes, ScalarCmd, ScalarSubcommand, VectorSubcommand, RotationSubcommand, LinearCmd, RotateCmd}};
use super::{create_boxed_future_client_error, ButtplugClientResultFuture, ButtplugClientMessageSender};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DeviceActuatorType {
  Unknown,
  Vibrate,
  Rotate,
  Oscillate,
  Position,
  Inflate,
  Constrict,
  PositionWithDuration,
  RotateWithDirection,  
}

impl From<ActuatorType> for DeviceActuatorType {
  fn from(value: ActuatorType) -> Self {
      match value {
        ActuatorType::Constrict => DeviceActuatorType::Constrict,
        ActuatorType::Inflate => DeviceActuatorType::Inflate,
        ActuatorType::Oscillate => DeviceActuatorType::Oscillate,
        ActuatorType::Position => DeviceActuatorType::Position,
        ActuatorType::Rotate => DeviceActuatorType::Rotate,
        ActuatorType::Unknown => DeviceActuatorType::Unknown,
        ActuatorType::Vibrate => DeviceActuatorType::Vibrate
      }
  }
}

impl Into<ActuatorType> for DeviceActuatorType {
  fn into(self) -> ActuatorType {
    match self {
      DeviceActuatorType::Constrict => ActuatorType::Constrict,
      DeviceActuatorType::Inflate => ActuatorType::Inflate,
      DeviceActuatorType::Oscillate => ActuatorType::Oscillate,
      DeviceActuatorType::Position => ActuatorType::Position,
      DeviceActuatorType::Rotate => ActuatorType::Rotate,
      DeviceActuatorType::Unknown => ActuatorType::Unknown,
      DeviceActuatorType::Vibrate => ActuatorType::Vibrate,
      DeviceActuatorType::PositionWithDuration => ActuatorType::Unknown,
      DeviceActuatorType::RotateWithDirection => ActuatorType::Unknown,
    }
  }
}

#[derive(Clone)]
pub struct ButtplugDeviceActuator {
  device_index: u32,
  actuator_type: DeviceActuatorType,
  attributes: ClientGenericDeviceMessageAttributes,
  message_sender: Arc<ButtplugClientMessageSender>,
}

impl ButtplugDeviceActuator {
  fn new(device_index: u32, actuator_type: DeviceActuatorType, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    return Self {
      device_index,
      actuator_type,
      attributes: attributes.clone(),
      message_sender: message_sender.clone()
    }
  }

  pub(super) fn from_scalarcmd_attributes(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::new(device_index, DeviceActuatorType::from(*attributes.actuator_type()), attributes, message_sender)
  }

  pub(super) fn from_rotatecmd_attributes(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::new(device_index, DeviceActuatorType::RotateWithDirection, attributes, message_sender)
  }

  pub(super) fn from_linearcmd_attributes(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::new(device_index, DeviceActuatorType::PositionWithDuration, attributes, message_sender)
  }

  pub(super) fn from_client_device_message_attributes(device_index: u32, attributes: &ClientDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Vec<Self> {
    let mut actuator_vec = vec!();
    actuator_vec.extend(attributes.scalar_cmd().iter().flat_map(|v| v.iter()).map(|attr| ButtplugDeviceActuator::from_scalarcmd_attributes(device_index, attr, message_sender)));
    actuator_vec.extend(attributes.rotate_cmd().iter().flat_map(|v| v.iter()).map(|attr| ButtplugDeviceActuator::from_rotatecmd_attributes(device_index, attr, message_sender)));
    actuator_vec.extend(attributes.linear_cmd().iter().flat_map(|v| v.iter()).map(|attr| ButtplugDeviceActuator::from_linearcmd_attributes(device_index, attr, message_sender)));
    actuator_vec
  }

  pub fn actuator_type(&self) -> DeviceActuatorType {
    self.actuator_type
  }

  pub fn descriptor(&self) -> &String {
    self.attributes.feature_descriptor()
  }

  pub fn can_scalar(&self) -> bool {
    [DeviceActuatorType::Constrict, DeviceActuatorType::Inflate, DeviceActuatorType::Oscillate, DeviceActuatorType::Position, DeviceActuatorType::Rotate, DeviceActuatorType::Vibrate].contains(&self.actuator_type)
  }

  pub fn scalar(&self, scalar: f64) -> ButtplugClientResultFuture {
    /*
    if scalar.is_sign_negative() || scalar > 1.0 {
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Value must be 0 <= x <= 1, but {scalar} was sent"))
          .into(),
      );
    }
    */
    let subcmd = ScalarSubcommand::new(*self.attributes.index(), scalar, self.actuator_type().into());
    let scalarcmd = ScalarCmd::new(self.device_index, vec![subcmd]);
    self.message_sender.send_message_expect_ok(scalarcmd.into())
  }

  pub fn send_scalar_if_supported(&self, actuator_type: DeviceActuatorType, scalar: f64) -> ButtplugClientResultFuture {
    if self.actuator_type != DeviceActuatorType::Vibrate {
      create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Actuator does not support {actuator_type:?} command"))
          .into(),
      )
    } else {
      self.scalar(scalar)
    }
  }

  pub fn vibrate(&self, speed: f64) -> ButtplugClientResultFuture {
    self.send_scalar_if_supported(DeviceActuatorType::Vibrate, speed)
  }

  pub fn rotate(&self, speed: f64) -> ButtplugClientResultFuture {
    self.send_scalar_if_supported(DeviceActuatorType::Rotate, speed)
  }

  pub fn oscillate(&self, speed: f64) -> ButtplugClientResultFuture {
    self.send_scalar_if_supported(DeviceActuatorType::Oscillate, speed)
  }

  pub fn position(&self, position: f64) -> ButtplugClientResultFuture {
    self.send_scalar_if_supported(DeviceActuatorType::Position, position)
  }

  pub fn inflate(&self, level: f64) -> ButtplugClientResultFuture {
    self.send_scalar_if_supported(DeviceActuatorType::Inflate, level)
  }

  pub fn constrict(&self, level: f64) -> ButtplugClientResultFuture {
    self.send_scalar_if_supported(DeviceActuatorType::Constrict, level)    
  }

  pub fn position_with_duration(
    &self,
    position: f64,
    duration: u32,
  ) -> ButtplugClientResultFuture {
    if self.actuator_type != DeviceActuatorType::PositionWithDuration {
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Actuator does not support Position With Duration command"))
          .into(),
      );
    }
    if position.is_sign_negative() || position > 1.0 {
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Value must be 0 <= x <= 1, but {position} was sent"))
          .into(),
      );
    }
    let subcmd = VectorSubcommand::new(*self.attributes.index(), duration, position);
    let linearcmd = LinearCmd::new(self.device_index, vec![subcmd]);
    self.message_sender.send_message_expect_ok(linearcmd.into())
  }

  pub fn rotate_with_direction(&self, speed: f64, clockwise: bool) -> ButtplugClientResultFuture {
    if self.actuator_type != DeviceActuatorType::RotateWithDirection {
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Actuator does not support Position With Duration command"))
          .into(),
      );
    }
    if speed.is_sign_negative() || speed > 1.0 {
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Value must be 0 <= x <= 1, but {speed} was sent"))
          .into(),
      );
    }
    let subcmd = RotationSubcommand::new(*self.attributes.index(), speed, clockwise);
    let rotatecmd = RotateCmd::new(self.device_index, vec![subcmd]);
    self.message_sender.send_message_expect_ok(rotatecmd.into())
  }
}
