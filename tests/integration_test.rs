use game_engine::math::{Mat4, Vec3, Vec4};
use game_engine::scene::{Camera, Mesh, Scene};

#[test]
fn test_scene_and_camera() {
    // Test creating a scene - it starts empty
    let scene = Scene::new();

    // Test that we can create and configure a camera
    let camera = Camera::new(
        Vec3::new(0.0, 2.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        16.0 / 9.0,
    );

    // Test that camera view projection is calculated correctly
    let view_proj = camera.view_projection_matrix();
    assert_ne!(view_proj, Mat4::identity());

    // Test that view and projection matrices are different
    assert_ne!(camera.view_matrix(), camera.projection_matrix());
}

#[test]
fn test_mesh_creation() {
    // Test creating predefined meshes
    let cube = Mesh::cube();
    assert!(!cube.vertices.is_empty());
    assert!(!cube.indices.is_empty());

    let plane = Mesh::plane(10.0, 10.0);
    assert!(!plane.vertices.is_empty());
    assert!(!plane.indices.is_empty());

    // Test vertex structure
    let vertex = &cube.vertices[0];
    assert!(vertex.position.x >= -1.0 && vertex.position.x <= 1.0);
    assert!(vertex.tex_coord.x >= 0.0 && vertex.tex_coord.x <= 1.0);
}

#[test]
fn test_transform_composition() {
    use game_engine::math::Transform;

    // Test that transform composition works correctly
    let transform = Transform::new(
        Vec3::new(1.0, 2.0, 3.0), // position
        Vec3::new(0.0, 0.0, 0.0), // rotation
        Vec3::new(2.0, 2.0, 2.0), // scale
    );

    let matrix = transform.to_matrix();

    // Test that a point is transformed correctly
    let point = Vec4::new(1.0, 0.0, 0.0, 1.0);
    let result = matrix.multiply_vec4(&point);

    // Point should be scaled by 2 and translated by (1, 2, 3)
    assert_eq!(result.x, 3.0); // 1 * 2 + 1 = 3
    assert_eq!(result.y, 2.0); // 0 * 2 + 2 = 2
    assert_eq!(result.z, 3.0); // 0 * 2 + 3 = 3
    assert_eq!(result.w, 1.0);
}

#[test]
fn test_camera_matrices() {
    let camera = Camera::new(
        Vec3::new(5.0, 5.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        1.0, // square aspect ratio
    );

    // Test that matrices are not identity
    assert_ne!(camera.view_matrix(), Mat4::identity());
    assert_ne!(camera.projection_matrix(), Mat4::identity());

    // Test that view projection is calculated
    let view_proj = camera.view_projection_matrix();
    assert_ne!(view_proj, Mat4::identity());

    // Test different aspect ratio
    let camera_wide = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        16.0 / 9.0,
    );
    assert_ne!(camera_wide.projection_matrix(), Mat4::identity());
}
