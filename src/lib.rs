extern crate nalgebra_glm as glm;

pub mod camera;
use camera::*;

pub mod renderer;
use renderer::*;

pub struct App {
	scene: Scene,

}

pub struct Entity {

}

pub struct Scene {
	entities: Vec<Entity>,
}