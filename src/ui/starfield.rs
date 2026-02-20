use dioxus::prelude::*;

#[component]
pub fn Starfield() -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            init_starfield();
        });
    }

    rsx! {
        canvas {
            class: "starfield",
            id: "starfield-canvas",
        }
    }
}

#[cfg(target_arch = "wasm32")]
type AnimCallback =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut(f64)>>>>;

#[cfg(target_arch = "wasm32")]
struct Star {
    x: f64,
    y: f64,
    size: f64,
    brightness: f64,
    twinkle_speed: f64,
    twinkle_offset: f64,
    drift_x: f64,
    drift_y: f64,
    hue_shift: f64,
}

#[cfg(target_arch = "wasm32")]
fn init_starfield() {
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };
    let canvas = match document.get_element_by_id("starfield-canvas") {
        Some(el) => match el.dyn_into::<web_sys::HtmlCanvasElement>() {
            Ok(c) => c,
            Err(_) => return,
        },
        None => return,
    };

    let dpr = window.device_pixel_ratio();
    let w = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(1920.0);
    let h = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(1080.0);
    canvas.set_width((w * dpr) as u32);
    canvas.set_height((h * dpr) as u32);
    let _ = canvas.style().set_property("width", &format!("{}px", w));
    let _ = canvas.style().set_property("height", &format!("{}px", h));

    let ctx = match canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
    {
        Some(c) => c,
        None => return,
    };
    ctx.scale(dpr, dpr).ok();

    let num_stars = 350;
    let mut stars: Vec<Star> = Vec::with_capacity(num_stars);
    let mut seed: u64 = 7919; // prime seed

    let mut rng = |seed: &mut u64| -> f64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*seed >> 33) as f64 / (u32::MAX as f64)
    };

    for _ in 0..num_stars {
        let layer = rng(&mut seed);
        let size = if layer < 0.6 {
            0.3 + rng(&mut seed) * 0.7 // small dim stars (60%)
        } else if layer < 0.9 {
            1.0 + rng(&mut seed) * 1.0 // medium stars (30%)
        } else {
            1.8 + rng(&mut seed) * 1.2 // bright large stars (10%)
        };

        let brightness = if layer < 0.6 {
            0.2 + rng(&mut seed) * 0.3
        } else if layer < 0.9 {
            0.4 + rng(&mut seed) * 0.4
        } else {
            0.7 + rng(&mut seed) * 0.3
        };

        stars.push(Star {
            x: rng(&mut seed) * w,
            y: rng(&mut seed) * h,
            size,
            brightness,
            twinkle_speed: 0.5 + rng(&mut seed) * 2.5,
            twinkle_offset: rng(&mut seed) * std::f64::consts::TAU,
            drift_x: (rng(&mut seed) - 0.5) * 0.03,
            drift_y: 0.005 + rng(&mut seed) * 0.02,
            hue_shift: rng(&mut seed),
        });
    }

    let stars = std::rc::Rc::new(std::cell::RefCell::new(stars));
    let cb: AnimCallback = std::rc::Rc::new(std::cell::RefCell::new(None));
    let cb_clone = cb.clone();
    let stars_clone = stars.clone();

    *cb.borrow_mut() = Some(Closure::new(move |t: f64| {
        let t_sec = t / 1000.0;

        // Dark background with subtle gradient feel
        ctx.set_fill_style_str("#060612");
        ctx.fill_rect(0.0, 0.0, w, h);

        // Subtle nebula glow patches
        let glow_colors = [
            (w * 0.2, h * 0.3, 200.0, "rgba(30, 20, 60, 0.15)"),
            (w * 0.7, h * 0.6, 250.0, "rgba(15, 30, 50, 0.12)"),
            (w * 0.5, h * 0.8, 180.0, "rgba(20, 15, 45, 0.10)"),
        ];
        for (gx, gy, gr, gc) in &glow_colors {
            let gradient = match ctx.create_radial_gradient(*gx, *gy, 0.0, *gx, *gy, *gr) {
                Ok(g) => g,
                Err(_) => continue,
            };
            let _ = gradient.add_color_stop(0.0, gc);
            let _ = gradient.add_color_stop(1.0, "rgba(0,0,0,0)");
            ctx.set_fill_style_canvas_gradient(&gradient);
            ctx.fill_rect(gx - gr, gy - gr, gr * 2.0, gr * 2.0);
        }

        let mut stars = stars_clone.borrow_mut();
        for star in stars.iter_mut() {
            let twinkle = ((t_sec * star.twinkle_speed + star.twinkle_offset).sin() + 1.0) / 2.0;
            let alpha = star.brightness * (0.4 + 0.6 * twinkle);

            // Color variation: most stars white-blue, some warmer
            let (r, g, b) = if star.hue_shift < 0.7 {
                // White-blue
                let blue_mix = 0.8 + star.hue_shift * 0.2;
                (
                    (180.0 + 60.0 * blue_mix) as u8,
                    (190.0 + 50.0 * blue_mix) as u8,
                    (220.0 + 35.0 * blue_mix) as u8,
                )
            } else if star.hue_shift < 0.85 {
                // Warm yellow-white
                (240, 220, 180)
            } else if star.hue_shift < 0.95 {
                // Pale orange
                (240, 200, 160)
            } else {
                // Faint blue-purple
                (180, 170, 240)
            };

            ctx.set_global_alpha(alpha);
            ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));
            ctx.begin_path();
            ctx.arc(star.x, star.y, star.size, 0.0, std::f64::consts::TAU)
                .ok();
            ctx.fill();

            // Glow for brighter stars
            if star.size > 1.2 {
                ctx.set_global_alpha(alpha * 0.15);
                ctx.begin_path();
                ctx.arc(star.x, star.y, star.size * 3.0, 0.0, std::f64::consts::TAU)
                    .ok();
                ctx.fill();
            }

            // Drift
            star.x += star.drift_x;
            star.y += star.drift_y;
            if star.y > h + 4.0 {
                star.y = -4.0;
            }
            if star.x > w + 4.0 {
                star.x = -4.0;
            } else if star.x < -4.0 {
                star.x = w + 4.0;
            }
        }

        ctx.set_global_alpha(1.0);

        if let Some(win) = web_sys::window() {
            if let Some(ref closure) = *cb_clone.borrow() {
                let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
            }
        }
    }));

    if let Some(ref closure) = *cb.borrow() {
        let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
    };
}
