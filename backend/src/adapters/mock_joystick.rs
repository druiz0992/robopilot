use crate::models::JoystickData;
use crate::ports::JoystickHandler;

pub struct MockJoystickProcessor;

impl JoystickHandler for MockJoystickProcessor {
    fn handle_joystick_data(&self, data: JoystickData) {
        println!("Processing joystick data: {:?}", data);
    }
}
