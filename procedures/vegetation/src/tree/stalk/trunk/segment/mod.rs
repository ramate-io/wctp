pub mod join_point;
pub mod radial_branches;
pub mod simple;
pub mod trunk_split;

pub use join_point::{JoinPoint, JoinPointSdf, JoinType};
pub use radial_branches::RadialBranchesSegment;
pub use simple::SimpleTrunkSegment;
pub use trunk_split::TrunkSplitSegment;

/// Base configuration for a trunk segment
/// All segments work in unit space (0-1) and are transformed later
#[derive(Clone)]
pub struct SegmentConfig {
	/// Seed for noise generation
	pub seed: u32,
	/// Base radius at bottom (in unit space, typically 0.5)
	pub base_radius: f32,
	/// Top radius at top (in unit space, typically 0.4)
	pub top_radius: f32,
	/// Noise amplitude for surface variation
	pub noise_amplitude: f32,
	/// Noise frequency for surface variation
	pub noise_frequency: f32,
}

impl Default for SegmentConfig {
	fn default() -> Self {
		Self {
			seed: 0,
			base_radius: 0.5,
			top_radius: 0.4,
			noise_amplitude: 0.05,
			noise_frequency: 5.0,
		}
	}
}
