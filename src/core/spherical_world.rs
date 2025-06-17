use crate::math::{Vec2, Vec3};
use crate::scene::{Mesh, Vertex};
use std::collections::HashMap;

pub struct SphericalWorld {
    pub radius: f32,
    pub subdivision_level: u32,
    pub center: Vec3,
}

impl SphericalWorld {
    pub fn new(radius: f32, subdivision_level: u32) -> Self {
        Self {
            radius,
            subdivision_level,
            center: Vec3::zero(),
        }
    }

    pub fn generate_mesh(&self) -> Mesh {
        let mut vertices = Vec::new();

        // Generate icosahedron vertices
        let t = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio

        // Create 12 vertices of icosahedron
        let mut base_vertices = vec![
            Vec3::new(-1.0, t, 0.0),
            Vec3::new(1.0, t, 0.0),
            Vec3::new(-1.0, -t, 0.0),
            Vec3::new(1.0, -t, 0.0),
            Vec3::new(0.0, -1.0, t),
            Vec3::new(0.0, 1.0, t),
            Vec3::new(0.0, -1.0, -t),
            Vec3::new(0.0, 1.0, -t),
            Vec3::new(t, 0.0, -1.0),
            Vec3::new(t, 0.0, 1.0),
            Vec3::new(-t, 0.0, -1.0),
            Vec3::new(-t, 0.0, 1.0),
        ];

        // Normalize vertices to unit sphere
        for v in &mut base_vertices {
            *v = v.normalize();
        }

        // Create 20 triangular faces of icosahedron
        let base_indices = vec![
            // 5 faces around point 0
            0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, // 5 adjacent faces
            1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7, 1, 8, // 5 faces around point 3
            3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, // 5 adjacent faces
            4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9, 8, 1,
        ];

        // Subdivide the icosahedron
        let (subdivided_vertices, subdivided_indices) =
            self.subdivide_mesh(base_vertices, base_indices, self.subdivision_level);

        // Convert to mesh format with positions, normals, and UVs
        for vertex in subdivided_vertices {
            let position = vertex.scale(self.radius).add(&self.center);
            let normal = vertex; // Already normalized
            let uv = self.sphere_to_uv(vertex);

            vertices.push(Vertex {
                position,
                tex_coord: uv,
                normal,
            });
        }

        Mesh {
            vertices,
            indices: subdivided_indices.into_iter().map(|i| i as u16).collect(),
        }
    }

    fn subdivide_mesh(
        &self,
        vertices: Vec<Vec3>,
        indices: Vec<u32>,
        level: u32,
    ) -> (Vec<Vec3>, Vec<u32>) {
        if level == 0 {
            return (vertices, indices);
        }

        let mut new_vertices = vertices.clone();
        let mut new_indices = Vec::new();
        let mut midpoint_cache = HashMap::new();

        for i in (0..indices.len()).step_by(3) {
            let v1 = indices[i];
            let v2 = indices[i + 1];
            let v3 = indices[i + 2];

            // Get or create midpoints
            let a = self.get_midpoint(v1, v2, &vertices, &mut new_vertices, &mut midpoint_cache);
            let b = self.get_midpoint(v2, v3, &vertices, &mut new_vertices, &mut midpoint_cache);
            let c = self.get_midpoint(v3, v1, &vertices, &mut new_vertices, &mut midpoint_cache);

            // Create 4 new triangles
            new_indices.extend_from_slice(&[v1, a, c]);
            new_indices.extend_from_slice(&[v2, b, a]);
            new_indices.extend_from_slice(&[v3, c, b]);
            new_indices.extend_from_slice(&[a, b, c]);
        }

        self.subdivide_mesh(new_vertices, new_indices, level - 1)
    }

    fn get_midpoint(
        &self,
        v1: u32,
        v2: u32,
        vertices: &[Vec3],
        all_vertices: &mut Vec<Vec3>,
        cache: &mut HashMap<(u32, u32), u32>,
    ) -> u32 {
        let key = if v1 < v2 { (v1, v2) } else { (v2, v1) };

        if let Some(&index) = cache.get(&key) {
            return index;
        }

        let p1 = vertices[v1 as usize];
        let p2 = vertices[v2 as usize];
        let midpoint = p1.add(&p2).scale(0.5).normalize();

        let index = all_vertices.len() as u32;
        all_vertices.push(midpoint);
        cache.insert(key, index);

        index
    }

    fn sphere_to_uv(&self, point: Vec3) -> Vec2 {
        let theta = point.z.atan2(point.x);
        let phi = point.y.asin();

        let u = (theta + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
        let v = (phi + std::f32::consts::PI / 2.0) / std::f32::consts::PI;

        Vec2::new(u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_world_creation() {
        let world = SphericalWorld::new(50.0, 2);
        assert_eq!(world.radius, 50.0);
        assert_eq!(world.subdivision_level, 2);
        assert_eq!(world.center, Vec3::zero());
    }

    #[test]
    fn test_icosphere_generation() {
        let world = SphericalWorld::new(1.0, 0);
        let mesh = world.generate_mesh();

        // Icosahedron has 12 vertices
        assert_eq!(mesh.vertices.len(), 12);
        // Icosahedron has 20 faces * 3 indices per face
        assert_eq!(mesh.indices.len(), 60);
    }

    #[test]
    fn test_subdivision() {
        let world = SphericalWorld::new(1.0, 1);
        let mesh = world.generate_mesh();

        // After 1 subdivision, each triangle becomes 4
        // So 20 * 4 = 80 triangles
        assert_eq!(mesh.indices.len(), 80 * 3);
    }

    #[test]
    fn test_sphere_to_uv() {
        let world = SphericalWorld::new(1.0, 0);

        // Test equator point (positive X)
        let equator_front = Vec3::new(1.0, 0.0, 0.0);
        let uv = world.sphere_to_uv(equator_front);
        // atan2(0, 1) = 0, so (0 + PI) / (2*PI) = 0.5
        assert!((uv.x - 0.5).abs() < 0.01);
        assert!((uv.y - 0.5).abs() < 0.01); // At equator

        // Test north pole
        let north = Vec3::new(0.0, 1.0, 0.0);
        let uv_north = world.sphere_to_uv(north);
        assert!((uv_north.y - 1.0).abs() < 0.01); // At top
    }
}
