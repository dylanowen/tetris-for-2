use amethyst::core::HideHierarchySystem;
use amethyst::ecs::DispatcherBuilder;
use amethyst::ecs::System;
use amethyst::prelude::SystemDesc;
use amethyst::GameDataBuilder;

use crate::systems::KnownSystems;

pub trait KnownSystem<'s>: System<'s> {
    fn name() -> KnownSystems;
    fn dependencies() -> &'static [KnownSystems];
}

pub trait WithKnownSystem<'a, 'b> {
    fn with_known<S>(self, system: S) -> Self
    where
        S: for<'s> KnownSystem<'s> + 'static + Send;
}

pub trait WithKnownSystemDesc<'a, 'b> {
    fn with_known_desc<SD, S>(self, system_desc: SD) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'static,
        S: for<'c> KnownSystem<'c> + 'static + Send;
}

pub trait AddKnownSystem<'a, 'b>: Sized {
    fn add_known<S>(&mut self, system: S)
    where
        S: for<'s> KnownSystem<'s> + 'static + Send;
}

impl<'a, 'b> WithKnownSystem<'a, 'b> for GameDataBuilder<'a, 'b> {
    fn with_known<S>(self, system: S) -> Self
    where
        S: for<'s> KnownSystem<'s> + 'static + Send,
    {
        self.with(system, S::name(), S::dependencies())
    }
}

impl<'a, 'b> WithKnownSystemDesc<'a, 'b> for GameDataBuilder<'a, 'b> {
    fn with_known_desc<SD, S>(self, system_desc: SD) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'static,
        S: for<'c> KnownSystem<'c> + 'static + Send,
    {
        self.with_system_desc(system_desc, S::name(), S::dependencies())
    }
}

impl<'a, 'b> AddKnownSystem<'a, 'b> for DispatcherBuilder<'a, 'b> {
    fn add_known<S>(&mut self, system: S)
    where
        S: for<'s> KnownSystem<'s> + 'static + Send,
    {
        let dependencies: Vec<&'static str> = S::dependencies().iter().map(|k| k.into()).collect();
        self.add(system, S::name().into(), &dependencies);
    }
}

impl<'a, 'b> WithKnownSystem<'a, 'b> for DispatcherBuilder<'a, 'b> {
    fn with_known<S>(mut self, system: S) -> Self
    where
        S: for<'s> KnownSystem<'s> + 'static + Send,
    {
        self.add_known(system);

        self
    }
}

impl KnownSystem<'_> for HideHierarchySystem {
    fn name() -> KnownSystems {
        KnownSystems::HideHierarchy
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[KnownSystems::ParentHierarchy]
    }
}
