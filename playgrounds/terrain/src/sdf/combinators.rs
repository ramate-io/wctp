use crate::sdf::Sdf;
use bevy::prelude::*;

/// Add two SDFs together - adds their heights (for heightfield-like SDFs)
/// This is useful for adding features to terrain (bumps, depressions, etc.)
/// The result is the sum of the two surfaces
pub struct AddY<A, B> {
	a: A,
	b: B,
}

impl<A: Sdf, B: Sdf> AddY<A, B> {
	pub fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

impl<A: Sdf, B: Sdf> Sdf for AddY<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		// For heightfield-like SDFs where distance = p.y - height(x,z):
		// If d1 = p.y - h1 and d2 = p.y - h2
		// And we want h_combined = h1 + h2
		// Then d_combined = p.y - (h1 + h2) = (p.y - h1) + (p.y - h2) - p.y
		// = d1 + d2 - p.y
		let da = self.a.distance(p);
		let db = self.b.distance(p);
		da + db - p.y
	}
}

/// Union of two SDFs - combines them using the minimum distance
/// This creates the union of the two shapes
pub struct Union<A, B> {
	a: A,
	b: B,
}

impl<A: Sdf, B: Sdf> Union<A, B> {
	pub fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

impl<A: Sdf, B: Sdf> Sdf for Union<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		self.a.distance(p).min(self.b.distance(p))
	}
}

/// Smooth union of two SDFs using polynomial smooth minimum
/// The `k` parameter controls the smoothness (larger = smoother)
pub struct SmoothUnion<A, B> {
	a: A,
	b: B,
	k: f32,
}

impl<A: Sdf, B: Sdf> SmoothUnion<A, B> {
	pub fn new(a: A, b: B, k: f32) -> Self {
		Self { a, b, k }
	}

	/// Polynomial smooth minimum function
	/// Returns a smooth approximation of min(a, b)
	fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
		let h = (k - (a - b).abs()).max(0.0) / k;
		a.min(b) - h * h * h * k * (1.0 / 6.0)
	}
}

impl<A: Sdf, B: Sdf> Sdf for SmoothUnion<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		let da = self.a.distance(p);
		let db = self.b.distance(p);
		Self::smooth_min(da, db, self.k)
	}
}

/// Difference of two SDFs - subtracts B from A
/// This creates A - B (A with B removed)
pub struct Difference<A, B> {
	a: A,
	b: B,
}

impl<A: Sdf, B: Sdf> Difference<A, B> {
	pub fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

impl<A: Sdf, B: Sdf> Sdf for Difference<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		// Difference: max(a, -b)
		// This keeps points that are inside A but outside B
		self.a.distance(p).max(-self.b.distance(p))
	}
}

/// Smooth difference of two SDFs
pub struct SmoothDifference<A, B> {
	a: A,
	b: B,
	k: f32,
}

impl<A: Sdf, B: Sdf> SmoothDifference<A, B> {
	pub fn new(a: A, b: B, k: f32) -> Self {
		Self { a, b, k }
	}

	fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
		// Smooth max is negative of smooth min of negatives
		-SmoothUnion::<A, B>::smooth_min(-a, -b, k)
	}
}

impl<A: Sdf, B: Sdf> Sdf for SmoothDifference<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		let da = self.a.distance(p);
		let db = -self.b.distance(p);
		Self::smooth_max(da, db, self.k)
	}
}

/// Intersection of two SDFs - takes the maximum distance
/// This creates the intersection of the two shapes
pub struct Intersection<A, B> {
	a: A,
	b: B,
}

impl<A: Sdf, B: Sdf> Intersection<A, B> {
	pub fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

impl<A: Sdf, B: Sdf> Sdf for Intersection<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		// Intersection: max(a, b)
		// This keeps points that are inside both A and B
		self.a.distance(p).max(self.b.distance(p))
	}
}

/// Smooth intersection of two SDFs
pub struct SmoothIntersection<A, B> {
	a: A,
	b: B,
	k: f32,
}

impl<A: Sdf, B: Sdf> SmoothIntersection<A, B> {
	pub fn new(a: A, b: B, k: f32) -> Self {
		Self { a, b, k }
	}
}

impl<A: Sdf, B: Sdf> Sdf for SmoothIntersection<A, B> {
	fn distance(&self, p: Vec3) -> f32 {
		let da = self.a.distance(p);
		let db = self.b.distance(p);
		SmoothDifference::<A, B>::smooth_max(da, db, self.k)
	}
}

/// Translate an SDF by a vector
pub struct Translate<A> {
	sdf: A,
	offset: Vec3,
}

impl<A: Sdf> Translate<A> {
	pub fn new(sdf: A, offset: Vec3) -> Self {
		Self { sdf, offset }
	}
}

impl<A: Sdf> Sdf for Translate<A> {
	fn distance(&self, p: Vec3) -> f32 {
		self.sdf.distance(p - self.offset)
	}
}

/// Scale an SDF uniformly
pub struct Scale<A> {
	sdf: A,
	scale: f32,
}

impl<A: Sdf> Scale<A> {
	pub fn new(sdf: A, scale: f32) -> Self {
		Self { sdf, scale }
	}
}

impl<A: Sdf> Sdf for Scale<A> {
	fn distance(&self, p: Vec3) -> f32 {
		// Scale the point, then scale the distance back
		self.sdf.distance(p / self.scale) * self.scale
	}
}

/// Rotate an SDF around the Y axis
pub struct RotateY<A> {
	sdf: A,
	angle: f32, // in radians
}

impl<A: Sdf> RotateY<A> {
	pub fn new(sdf: A, angle: f32) -> Self {
		Self { sdf, angle }
	}
}

impl<A: Sdf> Sdf for RotateY<A> {
	fn distance(&self, p: Vec3) -> f32 {
		let cos_a = self.angle.cos();
		let sin_a = self.angle.sin();

		// Rotate point around Y axis
		let x = p.x * cos_a - p.z * sin_a;
		let z = p.x * sin_a + p.z * cos_a;

		self.sdf.distance(Vec3::new(x, p.y, z))
	}
}

/// Round the edges of an SDF (chamfer)
pub struct Round<A> {
	sdf: A,
	radius: f32,
}

impl<A: Sdf> Round<A> {
	pub fn new(sdf: A, radius: f32) -> Self {
		Self { sdf, radius }
	}
}

impl<A: Sdf> Sdf for Round<A> {
	fn distance(&self, p: Vec3) -> f32 {
		self.sdf.distance(p) - self.radius
	}
}

/// Elongate an SDF along an axis
pub struct Elongate<A> {
	sdf: A,
	elongation: Vec3,
}

impl<A: Sdf> Elongate<A> {
	pub fn new(sdf: A, elongation: Vec3) -> Self {
		Self { sdf, elongation }
	}
}

impl<A: Sdf> Sdf for Elongate<A> {
	fn distance(&self, p: Vec3) -> f32 {
		// Elongate by clamping point coordinates to the elongation bounds
		// This extends the SDF along the specified axes
		let q = Vec3::new(
			p.x - p.x.clamp(-self.elongation.x, self.elongation.x),
			p.y - p.y.clamp(-self.elongation.y, self.elongation.y),
			p.z - p.z.clamp(-self.elongation.z, self.elongation.z),
		);
		self.sdf.distance(q)
	}
}
