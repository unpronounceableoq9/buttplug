// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use std::sync::Arc;

use crate::core::{errors::ButtplugDeviceError, message::{ActuatorType, ClientGenericDeviceMessageAttributes, ClientDeviceMessageAttributes, ScalarCmd, ScalarSubcommand, VectorSubcommand, RotationSubcommand, LinearCmd, RotateCmd}};
use super::{create_boxed_future_client_error, ButtplugClientResultFuture, ButtplugClientMessageSender};

pub trait ScalarActuator {
  fn scalar(&self, scalar: f64) -> ButtplugClientResultFuture;
}

pub trait ActuatorAttributes {  
  fn descriptor(&self) -> &String;
  fn step_count(&self) -> u32;
}


#[derive(Clone)]
pub enum Actuator {
  Unknown(UnknownActuator),
  Vibrate(VibrateActuator),
  Rotate(RotateActuator),
  Oscillate(OscillateActuator),
  Position(PositionActuator),
  Inflate(InflateActuator),
  Constrict(ConstrictActuator),
  PositionWithDuration(PositionWithDurationActuator),
  RotateWithDirection(RotateWithDirectionActuator),
}

impl Actuator {

  pub(super) fn from_scalarcmd_attributes(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    match attributes.actuator_type() {
      ActuatorType::Vibrate => Self::Vibrate(VibrateActuator::new(device_index, attributes, message_sender)),
      ActuatorType::Constrict => Self::Constrict(ConstrictActuator::new(device_index, attributes, message_sender)),
      ActuatorType::Inflate => Self::Inflate(InflateActuator::new(device_index, attributes, message_sender)),
      ActuatorType::Oscillate => Self::Oscillate(OscillateActuator::new(device_index, attributes, message_sender)),
      ActuatorType::Position => Self::Position(PositionActuator::new(device_index, attributes, message_sender)),
      ActuatorType::Rotate => Self::Rotate(RotateActuator::new(device_index, attributes, message_sender)),
      ActuatorType::Unknown => Self::Unknown(UnknownActuator::new(device_index, attributes, message_sender)),
    }
  }

  pub(super) fn from_rotatecmd_attributes(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::RotateWithDirection(RotateWithDirectionActuator::new(device_index, attributes, message_sender))
  }

  pub(super) fn from_linearcmd_attributes(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::PositionWithDuration(PositionWithDurationActuator::new(device_index, attributes, message_sender))
  }

  pub(super) fn from_client_device_message_attributes(device_index: u32, attributes: &ClientDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Vec<Self> {
    let mut actuator_vec = vec!();
    actuator_vec.extend(attributes.scalar_cmd().iter().flat_map(|v| v.iter()).map(|attr| Actuator::from_scalarcmd_attributes(device_index, attr, message_sender)));
    actuator_vec.extend(attributes.rotate_cmd().iter().flat_map(|v| v.iter()).map(|attr| Actuator::from_rotatecmd_attributes(device_index, attr, message_sender)));
    actuator_vec.extend(attributes.linear_cmd().iter().flat_map(|v| v.iter()).map(|attr| Actuator::from_linearcmd_attributes(device_index, attr, message_sender)));
    actuator_vec
  }
}

macro_rules! actuator_struct_declaration {
  ($struct_name:ident) => {
    #[derive(Clone)]
    pub struct $struct_name {
      device_index: u32,
      attributes: ClientGenericDeviceMessageAttributes,
      message_sender: Arc<ButtplugClientMessageSender>,
    }

    impl ActuatorAttributes for $struct_name {    
      fn descriptor(&self) -> &String {
        self.attributes.feature_descriptor()
      }

      fn step_count(&self) -> u32 {
        *self.attributes.step_count()
      }
    }
  }
}

macro_rules! actuator_struct_impl {
  () => {
    fn new(device_index: u32, attributes: &ClientGenericDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
      return Self {
        device_index,
        attributes: attributes.clone(),
        message_sender: message_sender.clone()
      }
    }
  }
}

macro_rules! scalar_trait_impl {
  ($struct_name:ident) => {
    impl ScalarActuator for $struct_name {
      fn scalar(&self, scalar: f64) -> ButtplugClientResultFuture {
        let subcmd = ScalarSubcommand::new(*self.attributes.index(), scalar, *self.attributes.actuator_type());
        let scalarcmd = ScalarCmd::new(self.device_index, vec![subcmd]);
        self.message_sender.send_message_expect_ok(scalarcmd.into())
      }
    }
  }
}

macro_rules! scalar_actuator_struct {
  ($struct_name:ident, $actuation_name:ident) => {
    actuator_struct_declaration!($struct_name);
    
    impl $struct_name {
      actuator_struct_impl!();

      pub fn $actuation_name(&self, speed: f64) -> ButtplugClientResultFuture {
        self.scalar(speed)
      }
    }
    
    scalar_trait_impl!($struct_name);
  }
}

scalar_actuator_struct!(VibrateActuator, vibrate);
scalar_actuator_struct!(RotateActuator, rotate);
scalar_actuator_struct!(OscillateActuator, oscillate);
scalar_actuator_struct!(PositionActuator, position);
scalar_actuator_struct!(InflateActuator, inflate);
scalar_actuator_struct!(ConstrictActuator, constrict);
actuator_struct_declaration!(PositionWithDurationActuator);

impl PositionWithDurationActuator {
  actuator_struct_impl!();

  pub fn position_with_duration(
    &self,
    position: f64,
    duration: u32,
  ) -> ButtplugClientResultFuture {
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
}

actuator_struct_declaration!(RotateWithDirectionActuator);

impl RotateWithDirectionActuator {
  actuator_struct_impl!();

  pub fn rotate_with_direction(&self, speed: f64, clockwise: bool) -> ButtplugClientResultFuture {
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
actuator_struct_declaration!(UnknownActuator);

impl UnknownActuator {
  actuator_struct_impl!();
}

scalar_trait_impl!(UnknownActuator);
