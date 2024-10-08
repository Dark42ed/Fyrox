// Copyright (c) 2019-present Dmitry Stepanov and Fyrox Engine contributors.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Vertical cylinder emitter.

use crate::{
    core::{algebra::Vector3, numeric_range::RangeExt, reflect::prelude::*, visitor::prelude::*},
    scene::particle_system::{
        emitter::{
            base::{BaseEmitter, BaseEmitterBuilder},
            Emit, Emitter,
        },
        particle::Particle,
        ParticleSystemRng,
    },
};
use std::ops::{Deref, DerefMut};

/// See module docs.
#[derive(Clone, Debug, Visit, PartialEq, Reflect)]
pub struct CylinderEmitter {
    emitter: BaseEmitter,
    #[reflect(min_value = 0.0, step = 0.1)]
    height: f32,
    #[reflect(min_value = 0.0, step = 0.1)]
    radius: f32,
}

impl Default for CylinderEmitter {
    fn default() -> Self {
        Self {
            emitter: Default::default(),
            height: 1.0,
            radius: 0.5,
        }
    }
}

impl Deref for CylinderEmitter {
    type Target = BaseEmitter;

    fn deref(&self) -> &Self::Target {
        &self.emitter
    }
}

impl DerefMut for CylinderEmitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.emitter
    }
}

impl Emit for CylinderEmitter {
    fn emit(&self, particle: &mut Particle, rng: &mut ParticleSystemRng) {
        // Disk point picking extended in 3D - http://mathworld.wolfram.com/DiskPointPicking.html
        let scale: f32 = (0.0..1.0).random(rng);
        let theta = (0.0..2.0 * std::f32::consts::PI).random(rng);
        let z = (0.0..self.height).random(rng);
        let radius = scale.sqrt() * self.radius;
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        particle.position = self.position() + Vector3::new(x, y, z);
    }
}

impl CylinderEmitter {
    /// Returns radius of the cylinder emitter.
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Sets radius of the cylinder emitter.
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius.max(0.0);
    }

    /// Returns height of the cylinder emitter.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Sets height of the cylinder emitter.
    pub fn set_height(&mut self, height: f32) {
        self.height = height.max(0.0);
    }
}

/// Box emitter builder allows you to construct cylinder emitter in declarative manner.
/// This is typical implementation of Builder pattern.
pub struct CylinderEmitterBuilder {
    base: BaseEmitterBuilder,
    height: f32,
    radius: f32,
}

impl CylinderEmitterBuilder {
    /// Creates new cylinder emitter builder.
    pub fn new(base: BaseEmitterBuilder) -> Self {
        Self {
            base,
            height: 1.0,
            radius: 0.5,
        }
    }

    /// Sets desired height of the emitter.
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets desired radius of the emitter.
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Creates new cylinder emitter with given parameters.
    pub fn build(self) -> Emitter {
        Emitter::Cylinder(CylinderEmitter {
            emitter: self.base.build(),
            height: self.height,
            radius: self.radius,
        })
    }
}
