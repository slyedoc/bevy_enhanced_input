use bevy::prelude::*;
use log::warn;
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/**
Set of actions that need to be triggered at the same time.

- [`TriggerState::Fired`] if all chorded actions fire.
- [`TriggerState::Ongoing`] if any are active and [`Self::ongoing`] is `true`.
- [`TriggerState::None`] otherwise.

Useful for defining a composite action that fires only when all listed actions fire.

With [`Self::blocking`] the chord is a held modifier instead: a [`ConditionKind::Blocker`] that
lets its binding (or action) through only while every listed action fires and contributes nothing
of its own, so the state is the input's (and its other conditions'). Put it on a binding to give a
button a second job under a modifier (a controller has no [`ModKeys`]): the button alone and the
modifier alone are both [`TriggerState::None`]. An implicit chord cannot say that: it does not
read the input, so on its own it fires on the modifier alone, and beside [`Down`] either part
alone is [`TriggerState::Ongoing`], which starts the action. Pair the button's plain binding with
[`BlockBy`] on the modifier, so it stands down while the modifier is held.

Requires using [`SpawnRelated::spawn`] or separate spawning with [`ActionOf`]/[`BindingOf`]
because you need to pass [`Entity`] for step and cancel actions.

# Examples

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

# let mut world = World::new();
world.spawn((
    Player,
    Actions::<Player>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
        let modifier = context
            .spawn((Action::<Modifier>::new(), bindings![GamepadButton::LeftTrigger]))
            .id();

        // Use `Heal` if `Modifier` is active.
        context.spawn((
            Action::<Heal>::new(),
            Chord::single(modifier),
            bindings![GamepadButton::West],
        ));

        // Otherwise use `Attack`.
        context.spawn((Action::<Attack>::new(), bindings![GamepadButton::West]));
    })),
));

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct Attack;

#[derive(InputAction)]
#[action_output(bool)]
struct Modifier;

#[derive(InputAction)]
#[action_output(bool)]
struct Heal;
```
*/
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct Chord {
    /// Actions whose state will be inherited when they are firing.
    pub actions: SmallVec<[Entity; 2]>,

    /// Enables returning [`TriggerState::Ongoing`] when any action is active
    /// but not all have fired.
    ///
    /// When disabled, partial activation results in [`TriggerState::None`].
    ///
    /// Defaults to `true`. Ignored when [`Self::blocking`] is set.
    pub ongoing: bool,
    /// Makes the chord a [`ConditionKind::Blocker`]: [`TriggerState::None`] (blocking the
    /// binding or action it is on) until all chorded actions fire, otherwise
    /// [`TriggerState::Fired`], which leaves the state to the input and the other conditions.
    ///
    /// Defaults to `false`.
    pub blocking: bool,
}

impl Chord {
    /// Creates a new instance for a single action.
    #[must_use]
    pub fn single(action: Entity) -> Self {
        Self::new(smallvec![action])
    }

    /// Creates a new instance for multiple actions.
    #[must_use]
    pub fn new(actions: impl Into<SmallVec<[Entity; 2]>>) -> Self {
        Self {
            actions: actions.into(),
            ongoing: true,
            blocking: false,
        }
    }

    /// Sets [`Self::ongoing`].
    #[must_use]
    pub fn with_ongoing(mut self, enable: bool) -> Self {
        self.ongoing = enable;
        self
    }

    /// Sets [`Self::blocking`].
    #[must_use]
    pub fn with_blocking(mut self, enable: bool) -> Self {
        self.blocking = enable;
        self
    }
}

impl InputCondition for Chord {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        _value: ActionValue,
    ) -> TriggerState {
        let mut has_active = false;
        let mut all_fired = true;
        for &action in &self.actions {
            let Ok((_, &state, ..)) = actions.get(action) else {
                // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
                warn!("`{action}` is not a valid action");
                continue;
            };

            if state != TriggerState::None {
                has_active = true;
            }

            if state != TriggerState::Fired {
                all_fired = false;
            }
        }

        if has_active && all_fired {
            TriggerState::Fired
        } else if has_active && self.ongoing && !self.blocking {
            TriggerState::Ongoing
        } else {
            TriggerState::None
        }
    }

    fn kind(&self) -> ConditionKind {
        if self.blocking {
            ConditionKind::Blocker
        } else {
            ConditionKind::Implicit
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::context;

    #[test]
    fn fired() {
        let (mut world, mut state) = context::init_world();
        let action = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::Fired,
        );
    }

    #[test]
    fn with_ongoing() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::new([action1, action2]).with_ongoing(true);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::Ongoing,
        );
    }

    #[test]
    fn without_ongoing() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::new([action1, action2]).with_ongoing(false);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[test]
    fn none() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::new([action1, action2]);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[test]
    fn blocking() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();
        let mut partial = Chord::new([action1, action2]).with_blocking(true);
        assert!(matches!(partial.kind(), ConditionKind::Blocker));
        assert_eq!(
            partial.evaluate(&actions, &time, true.into()),
            TriggerState::None,
            "a partial chord blocks instead of going ongoing"
        );
        let mut full = Chord::single(action1).with_blocking(true);
        assert_eq!(
            full.evaluate(&actions, &time, true.into()),
            TriggerState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::single(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
