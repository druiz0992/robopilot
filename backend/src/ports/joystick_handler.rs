use crate::models::JoystickData;

pub trait JoystickHandler: Send + Sync + 'static {
    fn handle_joystick_data(&self, data: JoystickData);
}
