mod fragment;
mod mesh;
mod pipeline;
mod shaders;

use glam::Vec3;
use noise::OpenSimplex;
use raylib::prelude::*;
use rayon::prelude::*;
use std::{error::Error, fs::File, path::Path};

pub struct Uniforms {
    pub time: f32,
    pub angle: f32,
    pub light: Vec3,
    pub body: usize,
    pub grayscale: bool,
    pub noise: OpenSimplex,
}
const NAMES: [&str; 3] = ["01 / HELIOS", "02 / NEREIDA", "03 / ZEFIRO"];
const DESCRIPTIONS: [&str; 3] = [
    "ESTRELLA  /  Granulacion, manchas y plasma vivo",
    "MUNDO ROCOSO  /  Oceanos, continentes y nubes",
    "GIGANTE GASEOSO  /  Bandas fluidas y un gran vortice",
];
const FILES: [&str; 3] = ["helios", "nereida", "zefiro"];

fn shade(fragments: &[fragment::Fragment], u: &Uniforms) -> Vec<u8> {
    let mut rgba = vec![0; fragments.len() * 4];
    rgba.par_chunks_mut(4)
        .zip(fragments.par_iter())
        .for_each(|(out, f)| {
            let color = if f.covered {
                shaders::fragment_shader(f, u).clamp(Vec3::ZERO, Vec3::ONE)
            } else {
                Vec3::new(0.020, 0.031, 0.055)
            };
            out.copy_from_slice(&[
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
                255,
            ]);
        });
    rgba
}

fn capture(mesh: &[mesh::Vertex], u: &mut Uniforms) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("docs")?;
    let size = 480;
    let fragments = pipeline::rasterize(mesh, size, 0.0);
    for (body, name) in FILES.iter().enumerate() {
        u.body = body;
        u.time = 0.0;
        u.angle = 0.0;
        u.light = Vec3::new(-0.55, 0.38, 0.85).normalize();
        let rgba = shade(&fragments, u);
        image::save_buffer(
            format!("docs/{name}.png"),
            &rgba,
            size as u32,
            size as u32,
            image::ColorType::Rgba8,
        )?;
        let file = File::create(format!("docs/{name}.gif"))?;
        let mut encoder = gif::Encoder::new(file, size as u16, size as u16, &[])?;
        encoder.set_repeat(gif::Repeat::Infinite)?;
        // 6 segundos, 10 fps: evidencia determinista del mismo shader de la ventana.
        for i in 0..60 {
            u.time = i as f32 * 0.1;
            let mut rgba = shade(&fragments, u);
            let mut frame = gif::Frame::from_rgba_speed(size as u16, size as u16, &mut rgba, 20);
            frame.delay = 10;
            encoder.write_frame(&frame)?;
        }
        println!("Capturas: docs/{name}.png y docs/{name}.gif");
    }
    // Evidencia de iluminacion: el terreno queda fijo y cambia solo la luz.
    u.body = 1;
    u.time = 0.0;
    let file = File::create("docs/luz.gif")?;
    let mut encoder = gif::Encoder::new(file, size as u16, size as u16, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;
    for i in 0..60 {
        let a = i as f32 / 60.0 * std::f32::consts::TAU;
        u.light = Vec3::new(a.sin(), 0.35, a.cos()).normalize();
        let mut rgba = shade(&fragments, u);
        let mut frame = gif::Frame::from_rgba_speed(size as u16, size as u16, &mut rgba, 20);
        frame.delay = 10;
        encoder.write_frame(&frame)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let path = if let Some(i) = args.iter().position(|a| a == "--mesh") {
        Path::new(args.get(i + 1).ok_or("Falta la ruta despues de --mesh")?)
    } else {
        Path::new("assets/sphere.obj")
    };
    let vertices = mesh::load(path)?;
    let mut u = Uniforms {
        time: 0.0,
        angle: 0.0,
        light: Vec3::new(-0.55, 0.38, 0.85).normalize(),
        body: 0,
        grayscale: false,
        noise: OpenSimplex::new(42),
    };
    if args.iter().any(|a| a == "--capture") {
        return capture(&vertices, &mut u);
    }
    let size = 480;
    let (mut rl, thread) = raylib::init()
        .size(1100, 760)
        .title("Atlas celeste | Laboratorio 6 - CPU shaders")
        .build();
    rl.set_target_fps(30);
    let image = Image::gen_image_color(size as i32, size as i32, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image)?;
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let mut fragments = pipeline::rasterize(&vertices, size, u.angle);
    let mut paused = false;
    let mut moving_light = false;
    let mut light_time = 0.0_f32;
    let mut frames = 0;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.1);
        for (i, key) in [
            KeyboardKey::KEY_ONE,
            KeyboardKey::KEY_TWO,
            KeyboardKey::KEY_THREE,
        ]
        .into_iter()
        .enumerate()
        {
            if rl.is_key_pressed(key) {
                u.body = i;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_G) {
            u.grayscale = !u.grayscale;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_L) {
            moving_light = !moving_light;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            u.time = 0.0;
            u.angle = 0.0;
            u.light = Vec3::new(-0.55, 0.38, 0.85).normalize();
            light_time = 0.0;
            moving_light = false;
            fragments = pipeline::rasterize(&vertices, size, u.angle);
        }
        let direction = rl.is_key_down(KeyboardKey::KEY_RIGHT) as i32
            - rl.is_key_down(KeyboardKey::KEY_LEFT) as i32;
        if direction != 0 {
            u.angle += direction as f32 * dt * 0.6;
            fragments = pipeline::rasterize(&vertices, size, u.angle);
        }
        if !paused {
            u.time += dt;
        }
        if moving_light {
            light_time += dt;
            u.light = Vec3::new(light_time.sin(), 0.35, light_time.cos()).normalize();
        }
        texture.update_texture(&shade(&fragments, &u))?;
        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        let muted = Color::new(137, 153, 172, 255);
        let white = Color::new(231, 237, 240, 255);
        let accent = Color::new(234, 183, 103, 255);
        d.clear_background(Color::new(5, 8, 14, 255));
        d.draw_text("ATLAS CELESTE", 42, 32, 28, white);
        d.draw_text("ESTUDIO DE MATERIALES PROCEDURALES", 43, 70, 14, muted);
        d.draw_line(42, 103, 1058, 103, Color::new(40, 51, 65, 255));
        d.draw_texture_ex(&texture, Vector2::new(22.0, 118.0), 0.0, 1.17, Color::WHITE);
        d.draw_text("EXPLORADOR / 3 CUERPOS", 635, 152, 15, accent);
        d.draw_text(NAMES[u.body], 635, 192, 31, white);
        for (i, line) in DESCRIPTIONS[u.body].split("  /  ").enumerate() {
            d.draw_text(line, 635, 242 + i as i32 * 25, 18, muted);
        }
        for (i, name) in ["1   Helios", "2   Nereida", "3   Zefiro"]
            .iter()
            .enumerate()
        {
            let y = 340 + i as i32 * 54;
            if i == u.body {
                d.draw_rectangle(625, y - 10, 410, 43, Color::new(30, 39, 50, 255));
            }
            d.draw_text(name, 642, y, 20, if i == u.body { accent } else { muted });
        }
        d.draw_text("ESPACIO  Pausar material\nFLECHAS  Girar esfera\nG  Escala de grises\nL  Mover luz    R  Restablecer",635,540,18,muted);
        d.draw_line(42, 692, 1058, 692, Color::new(40, 51, 65, 255));
        d.draw_text(
            &format!("CPU / {} triangulos / {} FPS", vertices.len() / 3, fps),
            43,
            716,
            16,
            muted,
        );
        d.draw_text(
            if paused {
                "MATERIAL EN PAUSA"
            } else {
                "MATERIAL EN EVOLUCION"
            },
            810,
            716,
            15,
            accent,
        );
        drop(d);
        frames += 1;
        if args.iter().any(|a| a == "--smoke") && frames >= 3 {
            break;
        }
    }
    Ok(())
}
