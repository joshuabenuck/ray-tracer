use anyhow::Result;
use ray_tracer::YamlScene;

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("no yaml file provided");
    let mut scene = YamlScene::new(&path)?;
    scene.parse()?;

    let image = scene.render()?;
    scene.save(&path, image)?;

    Ok(())
}
