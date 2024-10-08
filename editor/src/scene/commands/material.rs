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

use crate::fyrox::{
    asset::ResourceData,
    core::{log::Log, sstorage::ImmutableString},
    material::{shader::ShaderResource, Material, MaterialResource, PropertyValue},
};
use crate::{command::CommandTrait, scene::commands::GameSceneContext};

#[derive(Debug)]
pub struct SetMaterialPropertyValueCommand {
    material: MaterialResource,
    name: ImmutableString,
    value: PropertyValue,
}
use crate::command::CommandContext;

fn try_save(material: &MaterialResource) {
    let header = material.header();
    if let Some(path) = header.kind.path_owned() {
        drop(header);
        Log::verify(material.data_ref().save(&path));
    }
}

impl SetMaterialPropertyValueCommand {
    pub fn new(material: MaterialResource, name: ImmutableString, value: PropertyValue) -> Self {
        Self {
            material,
            name,
            value,
        }
    }

    fn swap(&mut self) {
        let mut material = self.material.data_ref();

        let old_value = material.property_ref(&self.name).unwrap().clone();

        material
            .set_property(
                self.name.clone(),
                std::mem::replace(&mut self.value, old_value),
            )
            .unwrap();

        drop(material);
        try_save(&self.material);
    }
}

impl CommandTrait for SetMaterialPropertyValueCommand {
    fn name(&mut self, _: &dyn CommandContext) -> String {
        format!("Set Material {} Property Value", self.name)
    }

    fn execute(&mut self, _: &mut dyn CommandContext) {
        self.swap();
    }

    fn revert(&mut self, _: &mut dyn CommandContext) {
        self.swap();
    }
}

#[derive(Debug)]
enum SetMaterialShaderCommandState {
    Undefined,
    NonExecuted { new_shader: ShaderResource },
    Executed { old_material: Material },
    Reverted { new_material: Material },
}

#[derive(Debug)]
pub struct SetMaterialShaderCommand {
    material: MaterialResource,
    state: SetMaterialShaderCommandState,
}

impl SetMaterialShaderCommand {
    pub fn new(material: MaterialResource, shader: ShaderResource) -> Self {
        Self {
            material,
            state: SetMaterialShaderCommandState::NonExecuted { new_shader: shader },
        }
    }

    fn swap(&mut self, context: &mut dyn CommandContext) {
        let context = context.get_mut::<GameSceneContext>();
        match std::mem::replace(&mut self.state, SetMaterialShaderCommandState::Undefined) {
            SetMaterialShaderCommandState::Undefined => {
                unreachable!()
            }
            SetMaterialShaderCommandState::NonExecuted { new_shader } => {
                let mut material = self.material.data_ref();

                let old_material = std::mem::replace(
                    &mut *material,
                    Material::from_shader(new_shader, Some(context.resource_manager.clone())),
                );

                self.state = SetMaterialShaderCommandState::Executed { old_material };
            }
            SetMaterialShaderCommandState::Executed { old_material } => {
                let mut material = self.material.data_ref();

                let new_material = std::mem::replace(&mut *material, old_material);

                self.state = SetMaterialShaderCommandState::Reverted { new_material };
            }
            SetMaterialShaderCommandState::Reverted { new_material } => {
                let mut material = self.material.data_ref();

                let old_material = std::mem::replace(&mut *material, new_material);

                self.state = SetMaterialShaderCommandState::Executed { old_material };
            }
        }

        try_save(&self.material);
    }
}

impl CommandTrait for SetMaterialShaderCommand {
    fn name(&mut self, _: &dyn CommandContext) -> String {
        "Set Material Shader".to_owned()
    }

    fn execute(&mut self, ctx: &mut dyn CommandContext) {
        self.swap(ctx);
    }

    fn revert(&mut self, ctx: &mut dyn CommandContext) {
        self.swap(ctx);
    }
}
