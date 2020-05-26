/// Unique ID of a controller.
pub type ControllerId = usize;

/// The instantaneous state of a GameCube controller.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct ControllerState {
    plugged_in: bool,

    buttons: u32,
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    l: f32,
    r: f32,
}

impl ControllerState {
    pub fn is_plugged(data: &[u8; 9]) -> bool {
        data[0] != 0
    }

    pub fn read_slice(&mut self, data: &[u8; 9]) {
        self.plugged_in = ControllerState::is_plugged(data);
        self.buttons = (data[1] as u32) | ((data[2] as u32) << 0x08);
        self.x = data[3] as f32;
        self.y = data[4] as f32;
        self.cx = data[5] as f32;
        self.cy = data[6] as f32;
        self.l = data[7] as f32;
        self.r = data[8] as f32;
    }
}
