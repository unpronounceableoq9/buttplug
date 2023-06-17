// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use std::{sync::Arc, ops::RangeInclusive};

use futures_util::{FutureExt, Stream};

use super::{
  ButtplugClientMessageSender, ButtplugClientResultFuture,
};
use crate::{core::{
  errors::{ButtplugError, ButtplugMessageError},
  message::{
    ButtplugCurrentSpecServerMessage, ClientDeviceMessageAttributes,
    SensorDeviceMessageAttributes, SensorReadCmd, SensorSubscribeCmd, SensorType,
    SensorUnsubscribeCmd, SensorReading
  },
}, util::stream::convert_broadcast_receiver_to_stream};
use async_stream::stream;
use tokio::sync::broadcast;

pub trait SensorAttributes {  
  fn sensor_type(&self) -> SensorType;
  fn descriptor(&self) -> &String;
  fn sensor_range(&self) -> &Vec<RangeInclusive<i32>>;
}

trait ReadableSensor {
  fn read(&self) -> ButtplugClientResultFuture<Vec<i32>>;
}

pub trait SubscribableSensor {
  fn subscribe(&self) -> ButtplugClientResultFuture;
  fn unsubscribe(&self) -> ButtplugClientResultFuture;
}

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
    // Subscription sensors aren't done yet, don't add those for now.
    /*
    if let Some(subscribe_sensors) = attributes.sensor_read_cmd() {
      for subscribe_sensor in subscribe_sensors {
        match subscribe_sensor.sensor_type() {
          SensorType::Pressure => {
            sensors.push(Sensor::Pressure(PressureSensor::new(device_index, subscribe_sensor, message_sender)));
          }
          SensorType::Button => {
            sensors.push(Sensor::Button(ButtonSensor::new(device_index, subscribe_sensor, message_sender)));
          }
          _ => {
            sensors.push(Sensor::Unknown(UnknownSensor::new(device_index, subscribe_sensor, message_sender)));
          }
        }
      }
    }
    */
    sensors
  }
}

macro_rules! sensor_struct_declaration {
  ($struct_name:ident) => {
    #[derive(Clone)]
    pub struct $struct_name {
      // Allow dead code for Unknown sensor
      #[allow(dead_code)]
      device_index: u32,
      attributes: SensorDeviceMessageAttributes,
      // Allow dead code for Unknown sensor
      #[allow(dead_code)]
      message_sender: Arc<ButtplugClientMessageSender>,
      internal_event_sender: broadcast::Sender<SensorReading>,
    }

    impl SensorAttributes for $struct_name {
      fn sensor_type(&self) -> SensorType {
        *self.attributes.sensor_type()
      }
    
      fn descriptor(&self) -> &String {
        self.attributes.feature_descriptor()
      }

      fn sensor_range(&self) -> &Vec<RangeInclusive<i32>> {
        self.attributes.sensor_range()
      }
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
      let (sender, _) = broadcast::channel(256);
      return Self {
        device_index,
        attributes: attributes.clone(),
        message_sender: message_sender.clone(),
        internal_event_sender: sender
      };
    }
  };
}

macro_rules! sensor_read_impl {
  ($struct_name:ident) => {
    impl ReadableSensor for $struct_name {
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
    }
  };
}

macro_rules! sensor_subscribe_impl {
  ($struct_name:ident) => {
    impl SubscribableSensor for $struct_name {
      fn subscribe(
        &self
      ) -> ButtplugClientResultFuture {
        let msg = SensorSubscribeCmd::new(self.device_index, *self.attributes.index(), *self.attributes.sensor_type()).into();
        self.message_sender.send_message_expect_ok(msg)
      }
    
      fn unsubscribe(
        &self
      ) -> ButtplugClientResultFuture {
        let msg = SensorUnsubscribeCmd::new(self.device_index, *self.attributes.index(), *self.attributes.sensor_type()).into();
        self.message_sender.send_message_expect_ok(msg)
      }
    }
  };
}

sensor_struct_declaration!(BatterySensor);

sensor_read_impl!(BatterySensor);
impl BatterySensor {
  sensor_struct_impl!();
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

sensor_read_impl!(RssiSensor);
impl RssiSensor {
  sensor_struct_impl!();
  pub fn rssi_level(&self) -> ButtplugClientResultFuture<i32> {
    let send_fut = self.read();
    Box::pin(async move {
      let data = send_fut.await?;
      Ok(data[0])
    })
  }
}


pub fn convert_single_value_sensor_broadcast_receiver_to_stream(
  receiver: broadcast::Receiver<SensorReading>,
) -> impl Stream<Item = i32>
{
  stream! {
    pin_mut!(receiver);
    while let Ok(val) = receiver.recv().await {      
      yield val.data()[0];
    }
  }
}

sensor_struct_declaration!(PressureSensor);

sensor_subscribe_impl!(PressureSensor);
impl PressureSensor {
  sensor_struct_impl!();

  pub fn event_stream(&self) -> Box<dyn Stream<Item = i32> + Send + Unpin> {
    Box::new(Box::pin(convert_single_value_sensor_broadcast_receiver_to_stream(
      self.internal_event_sender.subscribe(),
    )))
  }
}

sensor_struct_declaration!(ButtonSensor);

sensor_subscribe_impl!(ButtonSensor);
impl ButtonSensor {
  sensor_struct_impl!();
  pub fn event_stream(&self) -> Box<dyn Stream<Item = i32> + Send + Unpin> {
    Box::new(Box::pin(convert_single_value_sensor_broadcast_receiver_to_stream(
      self.internal_event_sender.subscribe(),
    )))
  }
}

sensor_struct_declaration!(UnknownSensor);

impl UnknownSensor {
  sensor_struct_impl!();
}
