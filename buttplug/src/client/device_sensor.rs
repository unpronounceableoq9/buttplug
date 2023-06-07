// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use std::sync::Arc;

use futures_util::FutureExt;

use super::{
  ButtplugClientMessageSender, ButtplugClientResultFuture,
};
use crate::core::{
  errors::{ButtplugError, ButtplugMessageError},
  message::{
    ButtplugCurrentSpecServerMessage, ClientDeviceMessageAttributes,
    SensorDeviceMessageAttributes, SensorReadCmd, SensorSubscribeCmd, SensorType,
    SensorUnsubscribeCmd,
  },
};

pub enum Sensor {
  Battery(BatterySensor),
  Rssi(RssiSensor),
  Pressure(PressureSensor),
  Button(ButtonSensor),
  Unknown(UnknownSensor),
}

impl Sensor {
  pub(super) fn from_sensor_attributes(
    device_index: u32,
    attributes: &ClientDeviceMessageAttributes,
    message_sender: &Arc<ButtplugClientMessageSender>,
  ) -> Vec<Self> {
    let mut sensors = vec![];
    if let Some(read_sensors) = attributes.sensor_read_cmd() {
      for read_sensor in read_sensors {
        match read_sensor.sensor_type() {
          SensorType::Battery => {
            sensors.push(Sensor::Battery(BatterySensor::new(device_index, read_sensor, message_sender)));
          }
          SensorType::RSSI => {
            sensors.push(Sensor::Rssi(RssiSensor::new(device_index, read_sensor, message_sender)));
          }
          _ => {
            sensors.push(Sensor::Unknown(UnknownSensor::new(device_index, read_sensor, message_sender)));
          }
        }
      }
    }
    if let Some(read_sensors) = attributes.sensor_read_cmd() {
      for read_sensor in read_sensors {
        match read_sensor.sensor_type() {
          SensorType::Pressure => {

          }
          SensorType::Button => {

          }
          _ => {

          }
        }
      }
    }
    sensors
  }
}

macro_rules! sensor_struct_declaration {
  ($struct_name:ident) => {
    #[derive(Clone)]
    pub struct $struct_name {
      device_index: u32,
      attributes: SensorDeviceMessageAttributes,
      message_sender: Arc<ButtplugClientMessageSender>,
    }
  };
}

macro_rules! sensor_struct_impl {
  () => {
    fn new(
      device_index: u32,
      attributes: &SensorDeviceMessageAttributes,
      message_sender: &Arc<ButtplugClientMessageSender>,
    ) -> Self {
      return Self {
        device_index,
        attributes: attributes.clone(),
        message_sender: message_sender.clone(),
      };
    }
  
    pub fn sensor_type(&self) -> SensorType {
      *self.attributes.sensor_type()
    }
  
    pub fn descriptor(&self) -> &String {
      self.attributes.feature_descriptor()
    }  
  };
}

macro_rules! sensor_read_impl {
  () => {
    fn read(&self) -> ButtplugClientResultFuture<Vec<i32>> {
      let msg = SensorReadCmd::new(
        self.device_index,
        *self.attributes.index(),
        *self.attributes.sensor_type(),
      )
      .into();
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
  };
}

macro_rules! sensor_subscribe_impl {
  () => {
    pub fn subscribe(
      &self,
      sensor_index: u32,
      sensor_type: SensorType,
    ) -> ButtplugClientResultFuture {
      let msg = SensorSubscribeCmd::new(self.device_index, sensor_index, sensor_type).into();
      self.message_sender.send_message_expect_ok(msg)
    }
  
    pub fn unsubscribe(
      &self,
      sensor_index: u32,
      sensor_type: SensorType,
    ) -> ButtplugClientResultFuture {
      let msg = SensorUnsubscribeCmd::new(self.device_index, sensor_index, sensor_type).into();
      self.message_sender.send_message_expect_ok(msg)
    }
  };
}

sensor_struct_declaration!(BatterySensor);

impl BatterySensor {
  sensor_struct_impl!();
  sensor_read_impl!();
  pub fn battery_level(&self) -> ButtplugClientResultFuture<f64> {
    let send_fut = self.read();
    Box::pin(async move {
      let data = send_fut.await?;
      let battery_level = data[0];
      Ok(battery_level as f64 / 100.0f64)
    })
  }
}

sensor_struct_declaration!(RssiSensor);

impl RssiSensor {
  sensor_struct_impl!();
  sensor_read_impl!();
  pub fn rssi_level(&self) -> ButtplugClientResultFuture<i32> {
    let send_fut = self.read();
    Box::pin(async move {
      let data = send_fut.await?;
      Ok(data[0])
    })
  }
}

sensor_struct_declaration!(PressureSensor);

impl PressureSensor {
  sensor_struct_impl!();
  sensor_subscribe_impl!();
}

sensor_struct_declaration!(ButtonSensor);

impl ButtonSensor {
  sensor_struct_impl!();
  sensor_subscribe_impl!();
}

sensor_struct_declaration!(UnknownSensor);

impl UnknownSensor {
  sensor_struct_impl!();
}
