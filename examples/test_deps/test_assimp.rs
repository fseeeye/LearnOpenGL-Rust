use russimp::{
    self,
    scene::{PostProcess, Scene},
};

#[allow(unused_variables)]
fn main() {
    let scence = Scene::from_file(
        "assets/models/nanosuit/nanosuit.obj",
        vec![
            PostProcess::CalculateTangentSpace,
            PostProcess::Triangulate,
            PostProcess::JoinIdenticalVertices,
            PostProcess::SortByPrimitiveType,
        ],
    )
    .unwrap();
}
