use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
type AnimCallback =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut(f64)>>>>;

#[component]
pub fn Starfield() -> Element {
    rsx! {
        canvas {
            class: "starfield",
            id: "starfield-canvas",
            onmounted: move |_| {
                #[cfg(target_arch = "wasm32")]
                init_starfield();
            },
        }
    }
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

    // Generate stars
    let num_stars = 200;
    let mut stars: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(num_stars);
    // Simple pseudo-random using a seed
    let mut seed: u64 = 42;
    for _ in 0..num_stars {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = (seed as f64 / u64::MAX as f64) * w;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let y = (seed as f64 / u64::MAX as f64) * h;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let size = 0.5 + (seed as f64 / u64::MAX as f64) * 1.5;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let speed = 0.002 + (seed as f64 / u64::MAX as f64) * 0.008;
        stars.push((x, y, size, speed));
    }

    // Animation loop
    let stars = std::rc::Rc::new(std::cell::RefCell::new(stars));
    let cb: AnimCallback = std::rc::Rc::new(std::cell::RefCell::new(None));
    let cb_clone = cb.clone();
    let stars_clone = stars.clone();

    *cb.borrow_mut() = Some(Closure::new(move |_t: f64| {
        ctx.set_fill_style_str("#0a0a1a");
        ctx.fill_rect(0.0, 0.0, w, h);

        let mut stars = stars_clone.borrow_mut();
        for (x, y, size, speed) in stars.iter_mut() {
            // Twinkle: vary alpha with time
            let alpha = 0.4 + (*speed * 60.0).min(0.6);
            ctx.set_fill_style_str(&format!("rgba(200, 210, 255, {:.2})", alpha));
            ctx.begin_path();
            ctx.arc(*x, *y, *size, 0.0, std::f64::consts::TAU).ok();
            ctx.fill();

            // Slowly drift downward
            *y += *speed * 0.5;
            if *y > h + 2.0 {
                *y = -2.0;
            }
        }

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
