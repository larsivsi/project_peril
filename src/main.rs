extern crate regex;

mod config;
mod draw_object;
mod scene;

use config::Config;
use scene::Scene;
use std::time::{Duration, SystemTime};
//for debug/simulation
use std::thread::sleep;

fn main() {
    // init stuff
    let cfg = Config::read_config("options.cfg");
    println!(
        "window dims: {}x{}\nrender dims: {}x{}",
        cfg.window_dimensions.0,
        cfg.window_dimensions.1,
        cfg.render_dimensions.0,
        cfg.render_dimensions.1,
    );

    let scene = Scene::new();

    // main loop
    let mut running = true;
    let mut framecount: u64 = 0;
    // aim for 60fps = 16.66666... ms
    let delta_time = Duration::from_millis(17);
    let mut elapsed_time = Duration::new(0, 0);
    let mut accumulator = Duration::new(0, 0);
    let mut current_time = SystemTime::now();

    while running {
        let new_time = SystemTime::now();
        let frame_time = new_time.duration_since(current_time).expect(
            "duration_since failed :(",
        );
        current_time = new_time;
        accumulator += frame_time;

        while accumulator >= delta_time {
            //animation, physics engine, scene progression etc. goes here
            accumulator -= delta_time;
            elapsed_time += delta_time;
        }

        //call to render function goes here
        // (now simulated with a sleep)
        scene.draw();
        sleep(Duration::from_millis(10));
        //then swapbuffers etc.
        framecount += 1;

        if framecount % 100 == 0 {
            let frame_time_ms = frame_time.subsec_nanos() as f64 / 1_000_000.0;
            println!(
                "frametime: {}ms => {} FPS",
                frame_time_ms,
                1_000.0 / frame_time_ms
            );
        }

        // need some condition to stop the main loop
        if framecount == 1000 {
            running = false;
        }
    }

    //cleanup
}
