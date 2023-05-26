use std::sync::Arc;

use futures_util::FutureExt;

use crate::core::{message::{Endpoint, ButtplugCurrentSpecClientMessage, RawWriteCmd, ButtplugCurrentSpecServerMessage, RawReadCmd, RawSubscribeCmd, RawUnsubscribeCmd, ClientDeviceMessageAttributes}, errors::{ButtplugError, ButtplugMessageError}};

use super::{ButtplugClientMessageSender, ButtplugClientResultFuture};

#[derive(Clone)]
pub struct ButtplugDeviceRawEndpoint {
  endpoint: Endpoint,
  device_index: u32,
  message_sender: Arc<ButtplugClientMessageSender>, 
}

impl ButtplugDeviceRawEndpoint {
  pub(super) fn from_message_attributes(device_index: u32, attributes: &ClientDeviceMessageAttributes, message_sender: &Arc<ButtplugClientMessageSender>) -> Vec<ButtplugDeviceRawEndpoint> {
    let mut endpoints = vec!();
    if let Some(raw_attrs) = attributes.raw_read_cmd() {
      raw_attrs.endpoints().iter().for_each(|endpoint| endpoints.push(Self {
        endpoint: *endpoint,
        device_index,
        message_sender: message_sender.clone()
      }));
  
    }
    endpoints
  }

  pub fn write(
    &self,
    data: &[u8],
    write_with_response: bool,
  ) -> ButtplugClientResultFuture {
    let msg = ButtplugCurrentSpecClientMessage::RawWriteCmd(RawWriteCmd::new(
      self.device_index,
      self.endpoint,
      data,
      write_with_response,
    ));
    self.message_sender.send_message_expect_ok(msg)
  }

  pub fn read(
    &self,
    expected_length: u32,
    timeout: u32,
  ) -> ButtplugClientResultFuture<Vec<u8>> {
    let msg = ButtplugCurrentSpecClientMessage::RawReadCmd(RawReadCmd::new(
      self.device_index,
      self.endpoint,
      expected_length,
      timeout,
    ));
    let send_fut = self.message_sender.send_message(msg);
    async move {
      match send_fut.await? {
        ButtplugCurrentSpecServerMessage::RawReading(reading) => Ok(reading.data().clone()),
        ButtplugCurrentSpecServerMessage::Error(err) => Err(ButtplugError::from(err).into()),
        msg => Err(
          ButtplugError::from(ButtplugMessageError::UnexpectedMessageType(format!(
            "{:?}",
            msg
          )))
          .into(),
        ),
      }
    }
    .boxed()
  }

  pub fn subscribe(&self) -> ButtplugClientResultFuture {
    let msg =
      ButtplugCurrentSpecClientMessage::RawSubscribeCmd(RawSubscribeCmd::new(self.device_index, self.endpoint));
    self.message_sender.send_message_expect_ok(msg)
  }

  pub fn unsubscribe(&self) -> ButtplugClientResultFuture {
    let msg = ButtplugCurrentSpecClientMessage::RawUnsubscribeCmd(RawUnsubscribeCmd::new(
      self.device_index, self.endpoint,
    ));
    self.message_sender.send_message_expect_ok(msg)
  }
}