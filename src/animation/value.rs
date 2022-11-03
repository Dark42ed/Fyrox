use crate::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        reflect::{prelude::*, ResolvePath},
        visitor::prelude::*,
    },
    scene::node::Node,
    utils::log::Log,
};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum TrackValue {
    Vector3(Vector3<f32>),
    UnitQuaternion(UnitQuaternion<f32>),
}

impl TrackValue {
    pub fn weighted_clone(&self, weight: f32) -> Self {
        match self {
            TrackValue::Vector3(v) => TrackValue::Vector3(v.scale(weight)),
            TrackValue::UnitQuaternion(v) => TrackValue::UnitQuaternion(*v),
        }
    }

    pub fn blend_with(&mut self, other: &Self, weight: f32) {
        match (self, other) {
            (Self::Vector3(a), Self::Vector3(b)) => *a += b.scale(weight),
            (Self::UnitQuaternion(a), Self::UnitQuaternion(b)) => *a = a.nlerp(b, weight),
            _ => (),
        }
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Option<Self> {
        match (self, other) {
            (Self::Vector3(a), Self::Vector3(b)) => Some(Self::Vector3(a.lerp(b, t))),
            (Self::UnitQuaternion(a), Self::UnitQuaternion(b)) => {
                Some(Self::UnitQuaternion(a.nlerp(b, t)))
            }
            _ => None,
        }
    }

    pub fn boxed_value(&self) -> Box<dyn Reflect> {
        match self {
            TrackValue::Vector3(v) => Box::new(*v),
            TrackValue::UnitQuaternion(v) => Box::new(*v),
        }
    }
}

#[derive(Clone, Visit, Reflect, Debug, PartialEq, Eq)]
pub enum ValueBinding {
    Position,
    Scale,
    Rotation,
    Property(String),
}

#[derive(Clone, Debug)]
pub struct BoundValue {
    pub binding: ValueBinding,
    pub value: TrackValue,
}

impl BoundValue {
    pub fn weighted_clone(&self, weight: f32) -> Self {
        Self {
            binding: self.binding.clone(),
            value: self.value.weighted_clone(weight),
        }
    }

    pub fn blend_with(&mut self, other: &Self, weight: f32) {
        assert_eq!(self.binding, other.binding);
        self.value.blend_with(&other.value, weight);
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Option<Self> {
        assert_eq!(self.binding, other.binding);
        self.value.interpolate(&other.value, t).map(|value| Self {
            binding: self.binding.clone(),
            value,
        })
    }

    pub fn boxed_value(&self) -> Box<dyn Reflect> {
        self.value.boxed_value()
    }
}

#[derive(Clone, Debug, Default)]
pub struct BoundValueCollection {
    pub values: Vec<BoundValue>,
}

impl BoundValueCollection {
    pub fn weighted_clone(&self, weight: f32) -> Self {
        Self {
            values: self
                .values
                .iter()
                .map(|v| v.weighted_clone(weight))
                .collect::<Vec<_>>(),
        }
    }

    pub fn blend_with(&mut self, other: &Self, weight: f32) {
        for value in self.values.iter_mut() {
            if let Some(other_value) = other.values.iter().find(|v| v.binding == value.binding) {
                value.blend_with(other_value, weight);
            }
        }
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let mut new_values = Vec::new();
        for value in self.values.iter() {
            if let Some(other_value) = other.values.iter().find(|v| v.binding == value.binding) {
                new_values.push(value.interpolate(other_value, t).unwrap());
            }
        }

        Self { values: new_values }
    }

    pub fn apply(&self, node_ref: &mut Node) {
        for bound_value in self.values.iter() {
            match bound_value.binding {
                ValueBinding::Position => {
                    if let TrackValue::Vector3(v) = bound_value.value {
                        node_ref.local_transform_mut().set_position(v);
                    } else {
                        Log::err(
                            "Unable to apply position, because underlying type is not Vector3!",
                        )
                    }
                }
                ValueBinding::Scale => {
                    if let TrackValue::Vector3(v) = bound_value.value {
                        node_ref.local_transform_mut().set_scale(v);
                    } else {
                        Log::err("Unable to apply scaling, because underlying type is not Vector3!")
                    }
                }
                ValueBinding::Rotation => {
                    if let TrackValue::UnitQuaternion(v) = bound_value.value {
                        node_ref.local_transform_mut().set_rotation(v);
                    } else {
                        Log::err("Unable to apply rotation, because underlying type is not UnitQuaternion!")
                    }
                }
                ValueBinding::Property(ref property_name) => {
                    match node_ref.as_reflect_mut().resolve_path_mut(property_name) {
                        Ok(property) => {
                            if property.set(bound_value.boxed_value()).is_err() {
                                Log::err(format!(
                                    "Failed to set property {}! Types mismatch.",
                                    property_name
                                ));
                            }
                        }
                        Err(err) => {
                            Log::err(format!(
                                "Unable to find property {}! Reason: {:?}",
                                property_name, err
                            ));
                        }
                    }
                }
            }
        }
    }
}