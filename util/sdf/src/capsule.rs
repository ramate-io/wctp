use crate::Sdf;
use bevy::prelude::*;

/// A capsule SDF (cylinder with rounded ends)
pub struct CapsuleSdf {
	pub start: Vec3,
	pub end: Vec3,
	pub radius: f32,
}

impl CapsuleSdf {
	pub fn new(start: Vec3, end: Vec3, radius: f32) -> Self {
		Self { start, end, radius }
	}
}

impl Sdf for CapsuleSdf {
	fn distance(&self, p: Vec3) -> f32 {
		let pa = p - self.start;
		let ba = self.end - self.start;
		let h = (pa.dot(ba) / ba.length_squared()).clamp(0.0, 1.0);
		let closest_point = self.start + ba * h;
		(p - closest_point).length() - self.radius
	}
}

