use bevy::{ecs::spawn::SpawnWith, input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::{Release, *};
use test_log::test;

#[test]
fn explicit() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Test>::new(), Down::default(), bindings![Test::KEY])]),
    ));

    app.update();

    let mut actions = app.world_mut().query::<(&Action<Test>, &TriggerState)>();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, TriggerState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, TriggerState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Test::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, TriggerState::None);
}

#[test]
fn implicit() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let release = context
                .spawn((
                    Action::<OnRelease>::new(),
                    Release::default(),
                    bindings![OnRelease::KEY],
                ))
                .id();
            context.spawn((Action::<Test>::new(), Chord::single(release)));
        })),
    ));

    app.update();

    let mut release_actions = app
        .world_mut()
        .query::<(&Action<OnRelease>, &TriggerState)>();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, TriggerState::None);

    let mut test_actions = app.world_mut().query::<(&Action<Test>, &TriggerState)>();

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, TriggerState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(OnRelease::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(*release_action);
    assert_eq!(release_state, TriggerState::Ongoing);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, TriggerState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(OnRelease::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, TriggerState::Fired);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, TriggerState::Fired);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, TriggerState::None);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, TriggerState::None);
}

#[test]
fn blocker() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let release = context
                .spawn((
                    Action::<OnRelease>::new(),
                    Release::default(),
                    bindings![OnRelease::KEY],
                ))
                .id();
            context.spawn((
                Action::<Test>::new(),
                BlockBy::single(release),
                bindings![Test::KEY],
            ));
        })),
    ));

    app.update();

    let mut release_actions = app
        .world_mut()
        .query::<(&Action<OnRelease>, &TriggerState)>();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, TriggerState::None);

    let mut test_actions = app.world_mut().query::<(&Action<Test>, &TriggerState)>();

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, TriggerState::None);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(OnRelease::KEY);
    keys.press(Test::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(*release_action);
    assert_eq!(release_state, TriggerState::Ongoing);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(*test_action);
    assert_eq!(test_state, TriggerState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(OnRelease::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, TriggerState::Fired);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(*test_action);
    assert_eq!(test_state, TriggerState::None);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, TriggerState::None);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(*test_action);
    assert_eq!(test_state, TriggerState::Fired);
}

#[test]
fn blocking_chord() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let modifier = context
                .spawn((Action::<OnRelease>::new(), bindings![OnRelease::KEY]))
                .id();
            // The key's second job, under the modifier.
            context.spawn((
                Action::<Test>::new(),
                bindings![(Test::KEY, Chord::single(modifier).with_blocking(true))],
            ));
            // The key's own job, which stands down under the modifier.
            context.spawn((
                Action::<Plain>::new(),
                bindings![(Test::KEY, BlockBy::single(modifier))],
            ));
        })),
    ));

    app.update();

    let mut chorded = app.world_mut().query::<(&Action<Test>, &TriggerState)>();
    let mut plain = app.world_mut().query::<(&Action<Plain>, &TriggerState)>();
    let mut read = |app: &mut App| {
        let (&c, &cs) = chorded.single(app.world()).unwrap();
        let (&p, &ps) = plain.single(app.world()).unwrap();
        ((*c, cs), (*p, ps))
    };

    // The key alone: its own job, and no trace of the chord (not even ongoing).
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);
    app.update();
    assert_eq!(
        read(&mut app),
        ((false, TriggerState::None), (true, TriggerState::Fired))
    );

    // Modifier and key: the second job only.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(OnRelease::KEY);
    app.update();
    assert_eq!(
        read(&mut app),
        ((true, TriggerState::Fired), (false, TriggerState::None))
    );

    // The modifier alone: neither.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Test::KEY);
    app.update();
    assert_eq!(
        read(&mut app),
        ((false, TriggerState::None), (false, TriggerState::None))
    );
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;

impl Test {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(InputAction)]
#[action_output(bool)]
struct OnRelease;

impl OnRelease {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(InputAction)]
#[action_output(bool)]
struct Plain;
