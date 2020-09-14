use anyhow::Result;
use image::RgbImage;
use minifb::{Key, Window, WindowOptions};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use ray_tracer::{Color, YamlScene};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

fn async_render(path: &str) -> Result<(usize, usize, Receiver<(usize, usize, Color)>)> {
    let (itx, irx) = channel::<(usize, usize, Color)>();
    let path = path.to_string();
    std::thread::spawn(move || -> Result<()> {
        let mut scene = YamlScene::new(&path)?;
        scene.parse()?;
        let camera = scene.camera.as_mut().unwrap();
        camera.tx = Some(itx);
        scene.render()?;
        Ok(())
    });
    Ok((200, 200, irx))
}

fn render(path: &str) -> Result<RgbImage> {
    let mut scene = YamlScene::new(&path)?;
    scene.parse()?;
    let canvas = scene.render()?;
    let img = canvas.to_image();
    Ok(img)
}

fn draw(img_buffer: &RgbImage, buffer: &mut Vec<u32>) {
    for (i, p) in img_buffer.pixels().enumerate() {
        let p = p.0;
        let r: u32 = p[0] as u32;
        let g: u32 = p[1] as u32;
        let b: u32 = p[2] as u32;
        buffer[i] = (r << 8 * 2) + (g << 8) + b;
    }
}

fn main() -> Result<()> {
    // let image_dir = std::path::PathBuf::from("./images");
    // let images: Vec<String> = std::fs::read_dir(&image_dir)?
    //     .filter_map(|e| e.ok())
    //     .map(|e| e.file_name().to_str().unwrap().to_string())
    //     .filter(|n| n.contains(".png"))
    //     .collect();

    // let img = image::open(image_dir.join(&images.last().unwrap()))?;
    let path = std::env::args().nth(1).expect("no yaml file provided");

    let (tx, rx) = channel();
    // let (itx, irx) = channel::<(usize, usize, Color)>();
    let async_mode = false;
    let (width, height, mut irx, mut img_buffer) = if async_mode {
        let (width, height, irx) = async_render(&path)?;
        (width, height, Some(irx), None)
    } else {
        let img_buffer = render(&path)?;
        (
            img_buffer.width() as usize,
            img_buffer.height() as usize,
            None,
            Some(img_buffer),
        )
    };

    let mut window = Window::new(
        "Ray Tracer Visualizer",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut buffer: Vec<u32> = vec![0; width as usize * height as usize];
    if !async_mode {
        draw(&img_buffer.unwrap(), &mut buffer);
    }
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut watcher = watcher(tx, Duration::from_millis(100))?;
    watcher.watch("./scenes", RecursiveMode::NonRecursive)?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        match rx.try_recv() {
            Ok(DebouncedEvent::Write(event)) => {
                println!("{:?}", event);
                if async_mode {
                    irx = Some(async_render(&path)?.2);
                } else {
                    img_buffer = Some(render(&path)?);
                    draw(&img_buffer.unwrap(), &mut buffer);
                }
            }
            _ => {}
        }
        if let Some(irx) = &irx {
            loop {
                match irx.try_recv() {
                    Ok((x, y, color)) => {
                        let (r, g, b) = color.into();
                        let i = x + y * width as usize;
                        buffer[i] = ((r as u32) << 8 * 2) + ((g as u32) << 8) + b as u32;
                    }
                    _ => break,
                }
            }
        }
        if window.is_key_down(Key::R) {
            if async_mode {
                irx = Some(async_render(&path)?.2);
            } else {
                img_buffer = Some(render(&path)?);
                draw(&img_buffer.unwrap(), &mut buffer);
            }
        }
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .unwrap();
    }
    Ok(())
}
