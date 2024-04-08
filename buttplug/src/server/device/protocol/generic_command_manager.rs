// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use crate::{
  core::{
    errors::{ButtplugDeviceError, ButtplugError},
    message::{
      ActuatorType, ButtplugDeviceCommandMessageUnion, DeviceFeature, LinearCmd, RotateCmd, RotationSubcommand, ScalarCmd, ScalarSubcommand
    },
  },
  server::device::configuration::ProtocolDeviceAttributes,
};
use getset::Getters;
use std::{
  ops::RangeInclusive,
  sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed},
};

#[derive(Getters, Default)]
#[getset(get = "pub")]
struct CommandCache {
  scalar: AtomicU32,
  rotation_clockwise: AtomicBool,
}

// In order to make our lives easier, we make some assumptions about what's internally mutable in
// the GenericCommandManager (GCM). Once the GCM is configured for a device, it won't change sizes,
// because we don't support things like adding motors to devices randomly while Buttplug is running.
// Therefore we know that we'll just be storing values like vibration/rotation speeds. We can assume
// our storage of those can stay immutable (the vec sizes won't change) and make their internals
// mutable. While this could be RefCell'd or whatever, they're also always atomic types (until the
// horrible day some sex toy decides to use floats in its protocol), so we can just use atomics and
// call it done.
pub struct GenericCommandManager {
  sent_scalar: AtomicBool,
  sent_rotation: AtomicBool,
  _sent_linear: bool,
  features: Vec<(DeviceFeature, CommandCache)>,
  stop_commands: Vec<ButtplugDeviceCommandMessageUnion>,
}

impl GenericCommandManager {
  pub fn new(features: &Vec<DeviceFeature>) -> Self {

    let feature_cache: Vec<(DeviceFeature, CommandCache)> = features.iter().map(|x| (x.clone(), CommandCache::default())).collect();

    let mut stop_commands = vec![];
    let mut scalar_stop_subcommands = vec![];
    let mut rotate_stop_subcommands = vec![];
    features
      .iter()
      .filter(|x| {
        if let Some(actuator) = x.actuator() {
          actuator.messages().contains(&crate::core::message::ButtplugDeviceMessageType::ScalarCmd)
        } else {
          false
        }
      })
      .enumerate()
      .for_each(|(index, feature)| {
        scalar_stop_subcommands.push(ScalarSubcommand::new(
          index as u32,
          0.0,
          feature.feature_type().try_into().unwrap(),
        ));
      });
    if !scalar_stop_subcommands.is_empty() {
      stop_commands.push(ScalarCmd::new(0, scalar_stop_subcommands).into());
    }

    features
    .iter()
    .filter(|x| {
      if let Some(actuator) = x.actuator() {
        actuator.messages().contains(&crate::core::message::ButtplugDeviceMessageType::RotateCmd)
      } else {
        false
      }
    })
    .enumerate()
    .for_each(|(index, feature)| {
      rotate_stop_subcommands.push(RotationSubcommand::new(
        index as u32,
        0.0,
        false,
      ));
    });
    if !rotate_stop_subcommands.is_empty() {
      stop_commands.push(RotateCmd::new(0, rotate_stop_subcommands).into());
    }
    Self {
      sent_scalar: AtomicBool::new(false),
      sent_rotation: AtomicBool::new(false),
      _sent_linear: false,
      features: feature_cache,
      stop_commands,
    }
  }

  pub fn update_scalar(
    &self,
    msg: &ScalarCmd,
    match_all: bool,
  ) -> Result<Vec<Option<(ActuatorType, u32)>>, ButtplugError> {
    // First, make sure this is a valid command, that contains at least one
    // subcommand.
    //
    // TODO this should be part of message validity checks.
    if msg.scalars().is_empty() {
      return Err(
        ButtplugDeviceError::ProtocolRequirementError(
          "ScalarCmd has 0 commands, will not do anything.".to_owned(),
        )
        .into(),
      );
    }

    let scalar_features: Vec<&(DeviceFeature, CommandCache)> = self
      .features
      .iter()
      .filter(|(x, _)| {
        if let Some(actuator) = x.actuator() {
          actuator.messages().contains(&crate::core::message::ButtplugDeviceMessageType::ScalarCmd)
        } else {
          false
        }
      })
      .collect();

    // Now we convert from the generic 0.0-1.0 range to the StepCount
    // attribute given by the device config.

    // If we've already sent commands before, we should check against our
    // old values. Otherwise, we should always send whatever command we're
    // going to send.
    let mut result: Vec<Option<(ActuatorType, u32)>> = vec![None; scalar_features.len()];

    for scalar_command in msg.scalars() {
      let index = scalar_command.index() as usize;
      // Since we're going to iterate here anyways, we do our index check
      // here instead of in a filter above.
      if index >= scalar_features.len() {
        return Err(
          ButtplugDeviceError::ProtocolRequirementError(format!(
            "ScalarCmd has {} commands, device has {} features.",
            msg.scalars().len(),
            scalar_features.len()
          ))
          .into(),
        );
      }

      let range_start = scalar_features[index].0.actuator().as_ref().unwrap().step_range().as_ref().unwrap().start();
      let range = scalar_features[index].0.actuator().as_ref().unwrap().step_range().as_ref().unwrap().end() - range_start;
      let scalar_modifier = scalar_command.scalar() * range as f64;
      let scalar = if scalar_modifier < 0.0001 {
        0
      } else {
        // When calculating speeds, round up. This follows how we calculated
        // things in buttplug-js and buttplug-csharp, so it's more for history
        // than anything, but it's what users will expect.
        (scalar_modifier + *range_start as f64).ceil() as u32
      };
      trace!(
        "{:?} {} {} {}",
        scalar_features[index].0.actuator().as_ref().unwrap().step_range(),
        range,
        scalar_modifier,
        scalar
      );
      // If we've already sent commands, we don't want to send them again,
      // because some of our communication busses are REALLY slow. Make sure
      // these values get None in our return vector.
      let current_scalar = scalar_features[index].1.scalar().load(Relaxed);
      let sent_scalar = self.sent_scalar.load(Relaxed);
      if !sent_scalar || scalar != current_scalar {
        scalar_features[index].1.scalar().store(scalar, Relaxed);
        result[index] = Some((scalar_features[index].0.feature_type().try_into().unwrap(), scalar));
      }

      if !sent_scalar {
        self.sent_scalar.store(true, Relaxed);
      }
    }

    // If we have no changes to the device, just send back an empty command array. We have nothing
    // to do.
    if result.iter().all(|x| x.is_none()) {
      result.clear();
    } else if match_all {
      // If we're in a match all situation, set up the array with all prior
      // values before switching them out.
      for (index, (_, cmd)) in scalar_features.iter().enumerate() {
        if result[index].is_none() {
          result[index] = Some((scalar_features[index].0.feature_type().try_into().unwrap(), cmd.scalar().load(Relaxed)));
        }
      }
    }

    // Return the command vector for the protocol to turn into proprietary commands
    Ok(result)
  }

  // Test method
  #[cfg(test)]
  pub(super) fn scalars(&self) -> Vec<Option<(ActuatorType, u32)>> {
    self
      .features
      .iter()
      .filter(|(x, _)| {
        if let Some(actuator) = x.actuator() {
          actuator.messages().contains(&crate::core::message::ButtplugDeviceMessageType::ScalarCmd)
        } else {
          false
        }
      })
      .map(|(feature, cache)| Some((feature.feature_type().try_into().unwrap(), cache.scalar().load(Relaxed))))
      .collect()
  }

  pub fn update_rotation(
    &self,
    msg: &RotateCmd,
    match_all: bool,
  ) -> Result<Vec<Option<(u32, bool)>>, ButtplugError> {
    // First, make sure this is a valid command, that contains at least one
    // command.
    // TODO Move this to message validity checks
    if msg.rotations().is_empty() {
      return Err(
        ButtplugDeviceError::ProtocolRequirementError(
          "RotateCmd has 0 commands, will not do anything.".to_owned(),
        )
        .into(),
      );
    }

    // Now we convert from the generic 0.0-1.0 range to the StepCount
    // attribute given by the device config.

    // If we've already sent commands before, we should check against our
    // old values. Otherwise, we should always send whatever command we're
    // going to send.
    let rotate_features: Vec<&(DeviceFeature, CommandCache)> = self
      .features
      .iter()
      .filter(|(x, _)| {
        if let Some(actuator) = x.actuator() {
          actuator.messages().contains(&crate::core::message::ButtplugDeviceMessageType::RotateCmd)
        } else {
          false
        }
      })
      .collect();
    let mut result: Vec<Option<(u32, bool)>> = vec![None; rotate_features.len()];

    
    for rotate_command in msg.rotations() {
      let index = rotate_command.index() as usize;
      // Since we're going to iterate here anyways, we do our index check
      // here instead of in a filter above.
      if index >= rotate_features.len() {
        return Err(
          ButtplugDeviceError::ProtocolRequirementError(format!(
            "RotateCmd has {} commands, device has {} rotators.",
            msg.rotations().len(),
            rotate_features.len()
          ))
          .into(),
        );
      }

      // When calculating speeds, round up. This follows how we calculated
      // things in buttplug-js and buttplug-csharp, so it's more for history
      // than anything, but it's what users will expect.
      let range_start = rotate_features[index].0.actuator().as_ref().unwrap().step_range().as_ref().unwrap().start();
      let range = rotate_features[index].0.actuator().as_ref().unwrap().step_range().as_ref().unwrap().end() - range_start;      
      let speed_modifier = rotate_command.speed() * range as f64;
      let speed = if speed_modifier < 0.0001 {
        0
      } else {
        // When calculating speeds, round up. This follows how we calculated
        // things in buttplug-js and buttplug-csharp, so it's more for history
        // than anything, but it's what users will expect.
        (speed_modifier + *range_start as f64).ceil() as u32
      };
      let clockwise = rotate_command.clockwise();
      // If we've already sent commands, we don't want to send them again,
      // because some of our communication busses are REALLY slow. Make sure
      // these values get None in our return vector.
      let sent_rotation = self.sent_rotation.load(Relaxed);
      if !sent_rotation
        || speed != self.features[index].1.scalar().load(Relaxed)
        || clockwise != self.features[index].1.rotation_clockwise().load(Relaxed)
      {
        self.features[index].1.scalar().store(speed, Relaxed);
        self.features[index].1.rotation_clockwise().store(clockwise, Relaxed);
        result[index] = Some((speed, clockwise));
      }
      if !sent_rotation {
        self.sent_rotation.store(true, Relaxed);
      }
    }

    // If we're in a match all situation, set up the array with all prior
    // values before switching them out.
    if match_all && !result.iter().all(|x| x.is_none()) {
      for (index, (_, rotation)) in rotate_features.iter().enumerate() {
        if result[index].is_none() {
          result[index] = Some((rotation.scalar().load(Relaxed), rotation.rotation_clockwise().load(Relaxed)));
        }
      }
    }

    // Return the command vector for the protocol to turn into proprietary commands
    Ok(result)
  }

  pub fn _update_linear(&self, _msg: &LinearCmd) -> Result<Option<Vec<(u32, u32)>>, ButtplugError> {
    // First, make sure this is a valid command, that doesn't contain an
    // index we can't reach.

    // If we've already sent commands before, we should check against our
    // old values. Otherwise, we should always send whatever command we're
    // going to send.

    // Now we convert from the generic 0.0-1.0 range to the StepCount
    // attribute given by the device config.

    // If we've already sent commands, we don't want to send them again,
    // because some of our communication busses are REALLY slow. Make sure
    // these values get None in our return vector.

    // Return the command vector for the protocol to turn into proprietary commands
    Ok(None)
  }

  pub fn stop_commands(&self) -> Vec<ButtplugDeviceCommandMessageUnion> {
    self.stop_commands.clone()
  }
}

#[cfg(test)]
mod test {
/*
  use super::{GenericCommandManager, ProtocolDeviceAttributes};
  use crate::{
    core::message::{ActuatorType, RotateCmd, RotationSubcommand, ScalarCmd, ScalarSubcommand},
    server::device::configuration::{
      ProtocolAttributesType,
      ServerDeviceMessageAttributesBuilder,
      ServerGenericDeviceMessageAttributes,
    },
  };
  use std::ops::RangeInclusive;

  #[test]
  pub fn test_command_generator_vibration() {
    let scalar_attrs = ServerGenericDeviceMessageAttributes::new(
      "Test",
      &RangeInclusive::new(0, 20),
      ActuatorType::Vibrate,
    );
    let scalar_attributes = ServerDeviceMessageAttributesBuilder::default()
      .scalar_cmd(&vec![scalar_attrs.clone(), scalar_attrs])
      .finish();
    let device_attributes = ProtocolDeviceAttributes::new(
      ProtocolAttributesType::Default,
      None,
      None,
      scalar_attributes,
      None,
    );
    let mgr = GenericCommandManager::new(&device_attributes);
    let vibrate_msg = ScalarCmd::new(
      0,
      vec![
        ScalarSubcommand::new(0, 0.5, ActuatorType::Vibrate),
        ScalarSubcommand::new(1, 0.5, ActuatorType::Vibrate),
      ],
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg, false)
        .expect("Test, assuming infallible"),
      vec![
        Some((ActuatorType::Vibrate, 10)),
        Some((ActuatorType::Vibrate, 10))
      ]
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg, false)
        .expect("Test, assuming infallible"),
      vec![]
    );
    let vibrate_msg_2 = ScalarCmd::new(
      0,
      vec![
        ScalarSubcommand::new(0, 0.5, ActuatorType::Vibrate),
        ScalarSubcommand::new(1, 0.75, ActuatorType::Vibrate),
      ],
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg_2, false)
        .expect("Test, assuming infallible"),
      vec![None, Some((ActuatorType::Vibrate, 15))]
    );
    let vibrate_msg_invalid = ScalarCmd::new(
      0,
      vec![ScalarSubcommand::new(2, 0.5, ActuatorType::Vibrate)],
    );
    assert!(mgr.update_scalar(&vibrate_msg_invalid, false).is_err());

    assert_eq!(
      mgr.scalars(),
      vec![
        Some((ActuatorType::Vibrate, 10)),
        Some((ActuatorType::Vibrate, 15))
      ]
    );
  }

  #[test]
  pub fn test_command_generator_vibration_match_all() {
    let scalar_attrs = ServerGenericDeviceMessageAttributes::new(
      "Test",
      &RangeInclusive::new(0, 20),
      ActuatorType::Vibrate,
    );
    let scalar_attributes = ServerDeviceMessageAttributesBuilder::default()
      .scalar_cmd(&vec![scalar_attrs.clone(), scalar_attrs])
      .finish();
    let device_attributes = ProtocolDeviceAttributes::new(
      ProtocolAttributesType::Default,
      None,
      None,
      scalar_attributes,
      None,
    );
    let mgr = GenericCommandManager::new(&device_attributes);
    let vibrate_msg = ScalarCmd::new(
      0,
      vec![
        ScalarSubcommand::new(0, 0.5, ActuatorType::Vibrate),
        ScalarSubcommand::new(1, 0.5, ActuatorType::Vibrate),
      ],
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg, true)
        .expect("Test, assuming infallible"),
      vec![
        Some((ActuatorType::Vibrate, 10)),
        Some((ActuatorType::Vibrate, 10))
      ]
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg, true)
        .expect("Test, assuming infallible"),
      vec![]
    );
    let vibrate_msg_2 = ScalarCmd::new(
      0,
      vec![
        ScalarSubcommand::new(0, 0.5, ActuatorType::Vibrate),
        ScalarSubcommand::new(1, 0.75, ActuatorType::Vibrate),
      ],
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg_2, true)
        .expect("Test, assuming infallible"),
      vec![
        Some((ActuatorType::Vibrate, 10)),
        Some((ActuatorType::Vibrate, 15))
      ]
    );
    let vibrate_msg_invalid = ScalarCmd::new(
      0,
      vec![ScalarSubcommand::new(2, 0.5, ActuatorType::Vibrate)],
    );
    assert!(mgr.update_scalar(&vibrate_msg_invalid, false).is_err());

    assert_eq!(
      mgr.scalars(),
      vec![
        Some((ActuatorType::Vibrate, 10)),
        Some((ActuatorType::Vibrate, 15))
      ]
    );
  }

  #[test]
  pub fn test_command_generator_vibration_step_range() {
    let mut vibrate_attrs_1 = ServerGenericDeviceMessageAttributes::new(
      "Test",
      &RangeInclusive::new(0, 20),
      ActuatorType::Vibrate,
    );
    vibrate_attrs_1.set_step_range(RangeInclusive::new(10, 15));
    let mut vibrate_attrs_2 = ServerGenericDeviceMessageAttributes::new(
      "Test",
      &RangeInclusive::new(0, 20),
      ActuatorType::Vibrate,
    );
    vibrate_attrs_2.set_step_range(RangeInclusive::new(10, 20));

    let vibrate_attributes = ServerDeviceMessageAttributesBuilder::default()
      .scalar_cmd(&vec![vibrate_attrs_1, vibrate_attrs_2])
      .finish();
    let device_attributes = ProtocolDeviceAttributes::new(
      ProtocolAttributesType::Default,
      None,
      None,
      vibrate_attributes,
      None,
    );
    let mgr = GenericCommandManager::new(&device_attributes);
    let vibrate_msg = ScalarCmd::new(
      0,
      vec![
        ScalarSubcommand::new(0, 0.5, ActuatorType::Vibrate),
        ScalarSubcommand::new(1, 0.5, ActuatorType::Vibrate),
      ],
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg, false)
        .expect("Test, assuming infallible"),
      vec![
        Some((ActuatorType::Vibrate, 13)),
        Some((ActuatorType::Vibrate, 15))
      ]
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg, false)
        .expect("Test, assuming infallible"),
      vec![]
    );
    let vibrate_msg_2 = ScalarCmd::new(
      0,
      vec![
        ScalarSubcommand::new(0, 0.5, ActuatorType::Vibrate),
        ScalarSubcommand::new(1, 0.75, ActuatorType::Vibrate),
      ],
    );
    assert_eq!(
      mgr
        .update_scalar(&vibrate_msg_2, false)
        .expect("Test, assuming infallible"),
      vec![None, Some((ActuatorType::Vibrate, 18))]
    );
    let vibrate_msg_invalid = ScalarCmd::new(
      0,
      vec![ScalarSubcommand::new(2, 0.5, ActuatorType::Vibrate)],
    );
    assert!(mgr.update_scalar(&vibrate_msg_invalid, false).is_err());

    assert_eq!(
      mgr.scalars(),
      vec![
        Some((ActuatorType::Vibrate, 13)),
        Some((ActuatorType::Vibrate, 18))
      ]
    );
  }

  #[test]
  pub fn test_command_generator_rotation() {
    let rotate_attrs = ServerGenericDeviceMessageAttributes::new(
      "Test",
      &RangeInclusive::new(0, 20),
      ActuatorType::Rotate,
    );

    let rotate_attributes = ServerDeviceMessageAttributesBuilder::default()
      .rotate_cmd(&vec![rotate_attrs.clone(), rotate_attrs])
      .finish();
    let device_attributes = ProtocolDeviceAttributes::new(
      ProtocolAttributesType::Default,
      None,
      None,
      rotate_attributes,
      None,
    );
    let mgr = GenericCommandManager::new(&device_attributes);

    let rotate_msg = RotateCmd::new(
      0,
      vec![
        RotationSubcommand::new(0, 0.5, true),
        RotationSubcommand::new(1, 0.5, true),
      ],
    );
    assert_eq!(
      mgr
        .update_rotation(&rotate_msg, false)
        .expect("Test, assuming infallible"),
      vec![Some((10, true)), Some((10, true))]
    );
    assert_eq!(
      mgr
        .update_rotation(&rotate_msg, false)
        .expect("Test, assuming infallible"),
      vec![None, None]
    );
    let rotate_msg_2 = RotateCmd::new(
      0,
      vec![
        RotationSubcommand::new(0, 0.5, true),
        RotationSubcommand::new(1, 0.75, false),
      ],
    );
    assert_eq!(
      mgr
        .update_rotation(&rotate_msg_2, false)
        .expect("Test, assuming infallible"),
      vec![None, Some((15, false))]
    );
    let rotate_msg_3 = RotateCmd::new(
      0,
      vec![
        RotationSubcommand::new(0, 0.75, false),
        RotationSubcommand::new(1, 0.75, false),
      ],
    );
    assert_eq!(
      mgr
        .update_rotation(&rotate_msg_3, true)
        .expect("Test, assuming infallible"),
      vec![Some((15, false)), Some((15, false))]
    );
    let rotate_msg_invalid = RotateCmd::new(0, vec![RotationSubcommand::new(2, 0.5, true)]);
    assert!(mgr.update_rotation(&rotate_msg_invalid, false).is_err());
  }
  // TODO Write test for vibration stop generator
  */
}
