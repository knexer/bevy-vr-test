use bevy::prelude::*;
use bevy_oxr::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
    xr_input::{oculus_touch::OculusController, Hand},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<InputState>(InputState::default())
            .add_systems(Update, update_input_state);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TouchableButton {
    pub pressed: bool,
    pub just_pressed: bool,
    pub touched: bool,
    pub just_touched: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TouchableThumbstick {
    pub clicked: bool,
    pub just_clicked: bool,
    pub touched: bool,
    pub just_touched: bool,
    pub position: Vec2,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TouchableTrigger {
    pub touched: bool,
    pub just_touched: bool,
    pub value: f32,
    pub prev_value: f32,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct InputState {
    pub left_trigger: TouchableTrigger,
    pub right_trigger: TouchableTrigger,
    pub left_grip: TouchableTrigger,
    pub right_grip: TouchableTrigger,
    pub a_button: TouchableButton,
    pub b_button: TouchableButton,
    pub x_button: TouchableButton,
    pub y_button: TouchableButton,
    pub menu_button: TouchableButton,
    pub left_thumbstick: TouchableThumbstick,
    pub right_thumbstick: TouchableThumbstick,
}

fn update_input_state(
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut input_state: ResMut<InputState>,
) {
    // magic code to get the controller
    let frame_state = *frame_state.lock().unwrap();
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

    // New code
    input_state.left_trigger.prev_value = input_state.left_trigger.value;
    input_state.left_trigger.value = controller.trigger(Hand::Left);
    input_state.left_trigger.just_touched =
        input_state.left_trigger.touched == false && controller.trigger_touched(Hand::Left);
    input_state.left_trigger.touched = controller.trigger_touched(Hand::Left);

    input_state.right_trigger.prev_value = input_state.right_trigger.value;
    input_state.right_trigger.value = controller.trigger(Hand::Right);
    input_state.right_trigger.just_touched =
        input_state.right_trigger.touched == false && controller.trigger_touched(Hand::Right);
    input_state.right_trigger.touched = controller.trigger_touched(Hand::Right);

    input_state.left_grip.prev_value = input_state.left_grip.value;
    input_state.left_grip.value = controller.squeeze(Hand::Left);
    input_state.left_grip.just_touched = false; // TODO - implement grip touch
    input_state.left_grip.touched = false; // TODO - implement grip touch

    input_state.right_grip.prev_value = input_state.right_grip.value;
    input_state.right_grip.value = controller.squeeze(Hand::Right);
    input_state.right_grip.just_touched = false; // TODO - implement grip touch
    input_state.right_grip.touched = false; // TODO - implement grip touch

    input_state.a_button.just_pressed =
        input_state.a_button.pressed == false && controller.a_button();
    input_state.a_button.pressed = controller.a_button();
    input_state.a_button.just_touched =
        input_state.a_button.touched == false && controller.a_button_touched();
    input_state.a_button.touched = controller.a_button_touched();

    input_state.b_button.just_pressed =
        input_state.b_button.pressed == false && controller.b_button();
    input_state.b_button.pressed = controller.b_button();
    input_state.b_button.just_touched =
        input_state.b_button.touched == false && controller.b_button_touched();
    input_state.b_button.touched = controller.b_button_touched();

    input_state.x_button.just_pressed =
        input_state.x_button.pressed == false && controller.x_button();
    input_state.x_button.pressed = controller.x_button();
    input_state.x_button.just_touched =
        input_state.x_button.touched == false && controller.x_button_touched();
    input_state.x_button.touched = controller.x_button_touched();

    input_state.y_button.just_pressed =
        input_state.y_button.pressed == false && controller.y_button();
    input_state.y_button.pressed = controller.y_button();
    input_state.y_button.just_touched =
        input_state.y_button.touched == false && controller.y_button_touched();
    input_state.y_button.touched = controller.y_button_touched();

    input_state.menu_button.just_pressed =
        input_state.menu_button.pressed == false && controller.menu_button();
    input_state.menu_button.pressed = controller.menu_button();

    input_state.left_thumbstick.just_clicked =
        input_state.left_thumbstick.clicked == false && controller.thumbstick(Hand::Left).click;
    input_state.left_thumbstick.clicked = controller.thumbstick(Hand::Left).click;
    input_state.left_thumbstick.just_touched =
        input_state.left_thumbstick.touched == false && controller.thumbstick_touch(Hand::Left);
    input_state.left_thumbstick.touched = controller.thumbstick_touch(Hand::Left);
    input_state.left_thumbstick.position = Vec2::new(
        controller.thumbstick(Hand::Left).x,
        controller.thumbstick(Hand::Left).y,
    );

    input_state.right_thumbstick.just_clicked =
        input_state.right_thumbstick.clicked == false && controller.thumbstick(Hand::Right).click;
    input_state.right_thumbstick.clicked = controller.thumbstick(Hand::Right).click;
    input_state.right_thumbstick.just_touched =
        input_state.right_thumbstick.touched == false && controller.thumbstick_touch(Hand::Right);
    input_state.right_thumbstick.touched = controller.thumbstick_touch(Hand::Right);
    input_state.right_thumbstick.position = Vec2::new(
        controller.thumbstick(Hand::Right).x,
        controller.thumbstick(Hand::Right).y,
    );
}
