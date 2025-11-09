use crate::sdf::Sdf;
use bevy::prelude::*;

/// SDF for an inscribed polygon feature
/// Creates a bump or depression with a plateau (inner polygon) and slopes (between inner and outer polygons)
///
/// For a trapezoid example:
/// - Outer polygon A: the base of the trapezoid (larger)
/// - Inner polygon B: the plateau (smaller, inscribed in A)
/// - Height: how much to raise/lower the plateau
///
/// The SDF creates linear slopes from the outer edge (height 0) to the inner edge (height = `height`)
pub struct InscribedPolygonSdf {
	/// Outer polygon vertices (base) in XZ plane, should form a closed polygon
	outer_polygon: Vec<Vec2>,
	/// Inner polygon vertices (plateau) in XZ plane, should form a closed polygon
	inner_polygon: Vec<Vec2>,
	/// Height of the plateau (positive for bumps, negative for depressions)
	plateau_height: f32,
	/// Base Y coordinate (where the feature sits)
	base_y: f32,
}

impl InscribedPolygonSdf {
	/// Create a new inscribed polygon SDF
	///
	/// # Arguments
	/// * `outer_polygon` - Vertices of the outer polygon (base) in XZ plane
	/// * `inner_polygon` - Vertices of the inner polygon (plateau) in XZ plane
	/// * `plateau_height` - Height of the plateau (positive = bump, negative = depression)
	/// * `base_y` - Base Y coordinate where the feature sits
	pub fn new(
		outer_polygon: Vec<Vec2>,
		inner_polygon: Vec<Vec2>,
		plateau_height: f32,
		base_y: f32,
	) -> Self {
		Self { outer_polygon, inner_polygon, plateau_height, base_y }
	}

	/// Calculate signed distance from a point to a polygon in the XZ plane
	/// Returns negative if inside, positive if outside
	fn polygon_distance_2d(p: Vec2, polygon: &[Vec2]) -> f32 {
		if polygon.len() < 3 {
			return f32::MAX; // Invalid polygon
		}

		let mut min_dist = f32::MAX;
		let mut inside = true;

		// Check distance to each edge
		for i in 0..polygon.len() {
			let v0 = polygon[i];
			let v1 = polygon[(i + 1) % polygon.len()];

			// Edge vector
			let edge = v1 - v0;
			let edge_len = edge.length();

			if edge_len < 1e-6 {
				continue; // Degenerate edge
			}

			// Vector from v0 to point
			let to_point = p - v0;

			// Project point onto edge
			let t = (to_point.dot(edge) / (edge_len * edge_len)).clamp(0.0, 1.0);
			let closest_on_edge = v0 + edge * t;

			// Distance from point to edge
			let dist_to_edge = (p - closest_on_edge).length();

			// Determine if point is inside or outside using cross product
			let cross = edge.x * to_point.y - edge.y * to_point.x;
			if cross < 0.0 {
				inside = false;
			}

			min_dist = min_dist.min(dist_to_edge);
		}

		// Return negative if inside, positive if outside
		if inside {
			-min_dist
		} else {
			min_dist
		}
	}

	/// Get the interpolated height at a point in the XZ plane
	/// Returns the height based on position between inner and outer polygons
	fn get_height_at(&self, p: Vec2) -> f32 {
		// Distance to inner polygon (negative if inside)
		let dist_inner = Self::polygon_distance_2d(p, &self.inner_polygon);

		// Distance to outer polygon (negative if inside)
		let dist_outer = Self::polygon_distance_2d(p, &self.outer_polygon);

		// If inside inner polygon, we're on the plateau
		if dist_inner <= 0.0 {
			return self.plateau_height;
		}

		// If outside outer polygon, we're not in the feature
		if dist_outer > 0.0 {
			return 0.0;
		}

		// We're between inner and outer polygons - interpolate
		// dist_inner is positive (distance from inner edge)
		// dist_outer is negative (we're inside outer polygon)
		// The total distance from inner to outer edge
		let total_dist = dist_inner - dist_outer; // dist_inner is positive, dist_outer is negative

		if total_dist < 1e-6 {
			return self.plateau_height; // Degenerate case, use plateau height
		}

		// Interpolate from plateau_height at inner edge to 0 at outer edge
		let t = dist_inner / total_dist; // 0 at inner edge, 1 at outer edge
		self.plateau_height * (1.0 - t)
	}
}

impl Sdf for InscribedPolygonSdf {
	fn distance(&self, p: Vec3) -> f32 {
		let p_2d = Vec2::new(p.x, p.z);
		let feature_height = self.get_height_at(p_2d);
		let surface_y = self.base_y + feature_height;

		// Return signed distance: negative if below surface, positive if above
		p.y - surface_y
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_trapezoid() {
		// Create a simple trapezoid:
		// Outer: large square from (-10, -10) to (10, 10)
		// Inner: small square from (-5, -5) to (5, 5)
		let outer = vec![
			Vec2::new(-10.0, -10.0),
			Vec2::new(10.0, -10.0),
			Vec2::new(10.0, 10.0),
			Vec2::new(-10.0, 10.0),
		];
		let inner = vec![
			Vec2::new(-5.0, -5.0),
			Vec2::new(5.0, -5.0),
			Vec2::new(5.0, 5.0),
			Vec2::new(-5.0, 5.0),
		];

		let sdf = InscribedPolygonSdf::new(outer, inner, 10.0, 0.0);

		// Test at center (should be on plateau)
		let center_dist = sdf.distance(Vec3::new(0.0, 10.0, 0.0));
		assert!((center_dist.abs() < 0.1), "Center should be on plateau at y=10");

		// Test at outer edge (should be at base)
		let outer_dist = sdf.distance(Vec3::new(10.0, 0.0, 0.0));
		assert!((outer_dist.abs() < 0.1), "Outer edge should be at base y=0");

		// Test halfway between (should be at interpolated height)
		let mid_dist = sdf.distance(Vec3::new(7.5, 5.0, 0.0));
		assert!((mid_dist.abs() < 0.1), "Midpoint should be at interpolated height");
	}
}
