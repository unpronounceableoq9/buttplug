// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use crate::{
  core::{
    errors::ButtplugDeviceError,
    message::{ActuatorType, Endpoint},
  },
  server::device::{
    hardware::{HardwareCommand, HardwareWriteCmd},
    protocol::{generic_protocol_setup, ProtocolHandler},
  },
};

generic_protocol_setup!(LiboShark, "libo-shark");

#[derive(Default)]
pub struct LiboShark {}

impl ProtocolHandler for LiboShark {
  fn handle_scalar_cmd(
    &self,
    cmds: &[Option<(ActuatorType, u32)>],
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    // Store off result before the match, so we drop the lock ASAP.
    let mut data = 0u8;
    if let Some((_, speed)) = cmds[0] {
      data |= (speed as u8) << 4;
    }
    if let Some((_, speed)) = cmds[1] {
      data |= speed as u8;
    }
    Ok(vec![
      HardwareWriteCmd::new(Endpoint::Tx, vec![data], false).into()
    ])
  }
}

/*
#[cfg(all(test, feature = "server"))]
mod test {
  use crate::{
    core::messages::{Endpoint, StopDeviceCmd, VibrateCmd, VibrateSubcommand},
    server::device::{
      hardware::{HardwareCommand, HardwareWriteCmd},
      hardware::communication::test::{
        check_test_recv_empty,
        check_test_recv_value,
        new_bluetoothle_test_device,
      },
    },
    util::async_manager,
  };

  #[test]
  pub fn test_libo_shark_protocol() {
    async_manager::block_on(async move {
      let (device, test_device) = new_bluetoothle_test_device("ShaYu")
        .await
        .expect("Test, assuming infallible");
      let command_receiver_tx = test_device
        .endpoint_receiver(&Endpoint::Tx)
        .expect("Test, assuming infallible");
      let command_receiver_tx_mode = test_device
        .endpoint_receiver(&Endpoint::TxMode)
        .expect("Test, assuming infallible");
      device
        .parse_message(
          VibrateCmd::new(
            0,
            vec![
              VibrateSubcommand::new(0, 0.5),
              VibrateSubcommand::new(1, 0.5),
            ],
          )
          .into(),
        )
        .await
        .expect("Test, assuming infallible");
      check_test_recv_value(
        &command_receiver_tx,
        HardwareCommand::Write(HardwareWriteCmd::new(Endpoint::Tx, vec![0x22], false)),
      );
      assert!(check_test_recv_empty(&command_receiver_tx));

      device
        .parse_message(VibrateCmd::new(0, vec![VibrateSubcommand::new(1, 1.0)]).into())
        .await
        .expect("Test, assuming infallible");
      check_test_recv_value(
        &command_receiver_tx,
        HardwareCommand::Write(HardwareWriteCmd::new(Endpoint::Tx, vec![0x23], false)),
      );
      assert!(check_test_recv_empty(&command_receiver_tx));

      device
        .parse_message(VibrateCmd::new(0, vec![VibrateSubcommand::new(0, 0.5)]).into())
        .await
        .expect("Test, assuming infallible");
      assert!(check_test_recv_empty(&command_receiver_tx));

      device
        .parse_message(StopDeviceCmd::new(0).into())
        .await
        .expect("Test, assuming infallible");
      check_test_recv_value(
        &command_receiver_tx,
        HardwareCommand::Write(HardwareWriteCmd::new(Endpoint::Tx, vec![0x00], false)),
      );
      assert!(check_test_recv_empty(&command_receiver_tx));
      assert!(check_test_recv_empty(&command_receiver_tx_mode));
    });
  }
}
 */
