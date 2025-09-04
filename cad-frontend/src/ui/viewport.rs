use rust_ui::geometry::Vector;

#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportData {
    /// Angle from the horizon up to the camera in radians. At 0.0 degrees the camera is parallel
    /// to the ground. At 90.0 degrees the camera is looking straight at the ground.
    pub horizontal_angle: f32,
    /// Angle "around" the pole. At 0.0 degrees the camera is looking towards the negative x-axis,
    /// at 90.0 degrees it is looking towards the negative y-axis. (I think)
    pub polar_angle: f32,
    /// The point around which the camera is orbiting. Panning moves this point
    pub looking_at: glm::Vec3,
    /// The distance the camera is from [ViewportData::looking_at], similar to a zoom, but not
    /// quite
    pub distance: f32,
    /// The size on the screen of the viewport
    pub size: Vector<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Viewport {}
