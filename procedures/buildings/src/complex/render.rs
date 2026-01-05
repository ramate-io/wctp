use crate::complex::{Complex, Floor, Partition};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::RenderItem;

#[derive(Debug, Clone)]
pub struct ComplexRenderer<P: Partition, F: Floor> {
	complex: Complex<P, F>,
	floor_thickness: f32,
	partition_thickness: f32,
}

impl<P: Partition, F: Floor> ComplexRenderer<P, F> {
	pub fn new(complex: Complex<P, F>) -> Self {
		Self { complex, floor_thickness: 0.1, partition_thickness: 0.1 }
	}
}

impl<P: Partition, F: Floor> RenderItem for ComplexRenderer<P, F> {
	fn spawn_render_items(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) -> Vec<Entity> {
		let mut entities = Vec::new();

		for (floor_coordinates, floor) in self.complex.floors.floors.iter() {
			let transform = transform.with_translation(floor_coordinates.position).with_scale(
				Vec3::new(self.complex.step_size.x, self.floor_thickness, self.complex.step_size.z),
			);

			entities.extend(floor.spawn_render_items(commands, cascade_chunk, transform));
		}

		for (partition_coordinates, partition) in self.complex.partitions.partitions.iter() {
			let y_scale = self.complex.step_size.y;

			let z_scale = if partition_coordinates.start.z == partition_coordinates.end.z {
				self.partition_thickness
			} else {
				partition_coordinates.end.z - partition_coordinates.start.z
			};

			let x_scale = if partition_coordinates.start.x == partition_coordinates.end.x {
				self.partition_thickness
			} else {
				partition_coordinates.end.x - partition_coordinates.start.x
			};

			let transform = transform
				.with_translation(partition_coordinates.start)
				.with_scale(Vec3::new(x_scale, y_scale, z_scale));

			entities.extend(partition.spawn_render_items(commands, cascade_chunk, transform));
		}

		entities
	}
}
