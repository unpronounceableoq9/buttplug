// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use std::sync::Arc;

use futures_util::FutureExt;

use crate::core::{errors::{ButtplugDeviceError, ButtplugError, ButtplugMessageError}, message::{SensorDeviceMessageAttributes, SensorType, ButtplugDeviceMessageType, SensorSubscribeCmd, SensorUnsubscribeCmd, SensorReadCmd, ButtplugCurrentSpecServerMessage, ClientDeviceMessageAttributes}};
use super::{create_boxed_future_client_error, ButtplugClientResultFuture, ButtplugClientMessageSender};

#[derive(Clone)]
pub struct ButtplugDeviceSensor {
  device_index: u32,
  attributes: SensorDeviceMessageAttributes,
  message_sender: Arc<ButtplugClientMessageSender>, 
  readable: bool,
  subscribable: bool
}

impl ButtplugDeviceSensor {
  fn new(device_index: u32, attributes: &SensorDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>, readable: bool, subscribable: bool) -> Self {
    return Self {
      device_index,
      attributes: attributes.clone(),
      message_sender: message_sender.clone(),
      readable,
      subscribable
    }
  }

  pub(super) fn from_sensor_read_attributes(device_index: u32, attributes: &SensorDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::new(device_index, attributes, message_sender, true, false)
  }

  pub(super) fn from_sensor_subscribe_attributes(device_index: u32, attributes: &SensorDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Self {
    Self::new(device_index, attributes, message_sender, false, true)
  }

  pub(super) fn from_sensor_attributes(device_index: u32, attributes: &ClientDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Vec<Self> {
    let mut sensors = vec!();
    sensors.extend(attributes.sensor_read_cmd().iter().flat_map(|v| v.iter()).map(|attr| Self::from_sensor_read_attributes(device_index, attr, message_sender)));
    sensors.extend(attributes.sensor_subscribe_cmd().iter().flat_map(|v| v.iter()).map(|attr| Self::from_sensor_subscribe_attributes(device_index, attr, message_sender)));
    sensors
  }

  pub fn sensor_type(&self) -> SensorType {
    *self.attributes.sensor_type()
  }

  pub fn descriptor(&self) -> &String {
    self.attributes.feature_descriptor()
  }

  pub fn readable(&self) -> bool {
    self.readable
  }

  pub fn subscribable(&self) -> bool {
    self.subscribable
  }

  pub fn subscribe(
    &self,
    sensor_index: u32,
    sensor_type: SensorType,
  ) -> ButtplugClientResultFuture {
    if !self.subscribable {
      return create_boxed_future_client_error(
        ButtplugDeviceError::MessageNotSupported(ButtplugDeviceMessageType::SensorSubscribeCmd)
          .into(),
      );
    }
    let msg = SensorSubscribeCmd::new(self.device_index, sensor_index, sensor_type).into();
    self.message_sender.send_message_expect_ok(msg)
  }

  pub fn unsubscribe(
    &self,
    sensor_index: u32,
    sensor_type: SensorType,
  ) -> ButtplugClientResultFuture {

    if !self.subscribable {
      return create_boxed_future_client_error(
        ButtplugDeviceError::MessageNotSupported(ButtplugDeviceMessageType::SensorSubscribeCmd)
          .into(),
      );
    }
    let msg = SensorUnsubscribeCmd::new(self.device_index, sensor_index, sensor_type).into();
    self.message_sender.send_message_expect_ok(msg)
  }

  fn read(&self) -> ButtplugClientResultFuture<Vec<i32>> {
    if !self.readable {
      return create_boxed_future_client_error(
        ButtplugDeviceError::MessageNotSupported(ButtplugDeviceMessageType::SensorReadCmd).into(),
      );
    }
    let msg = SensorReadCmd::new(self.device_index, *self.attributes.index(), *self.attributes.sensor_type()).into();
    let reply = self.message_sender.send_message(msg);
    async move {
      if let ButtplugCurrentSpecServerMessage::SensorReading(data) = reply.await? {
        Ok(data.data().clone())
      } else {
        Err(
          ButtplugError::ButtplugMessageError(ButtplugMessageError::UnexpectedMessageType(
            "SensorReading".to_owned(),
          ))
          .into(),
        )
      }
    }
    .boxed()
  }

  pub fn battery_level(&self) -> ButtplugClientResultFuture<f64> {
    if self.sensor_type() != SensorType::Battery {      
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Device sensor is not a Battery sensor"))
        .into(),
      );
    }
    let send_fut = self.read();
    Box::pin(async move {
      let data = send_fut.await?;
      let battery_level = data[0];
      Ok(battery_level as f64 / 100.0f64)
    })
  }

  pub fn rssi_level(&self) -> ButtplugClientResultFuture<i32> {
    if self.sensor_type() != SensorType::RSSI {      
      return create_boxed_future_client_error(
        ButtplugDeviceError::UnhandledCommand(format!("Device sensor is not a RSSI sensor"))
        .into(),
      );
    }
    let send_fut = self.read();
    Box::pin(async move {
      let data = send_fut.await?;
      Ok(data[0])
    })
  }
}
