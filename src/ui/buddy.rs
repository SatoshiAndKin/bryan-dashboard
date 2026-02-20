use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
type AnimCallback = std::rc::Rc<std::cell::RefCell<Option<Closure<dyn FnMut(f64)>>>>;

#[component]
pub fn BuddyCharacter(messages: Vec<(String, f64)>) -> Element {
    let pos = use_signal(|| (9999.0_f64, 9999.0_f64));
    #[allow(unused_variables)]
    let mouse = use_signal(|| (-1000.0_f64, -1000.0_f64));
    #[allow(unused_variables)]
    let vel = use_signal(|| (0.3_f64, 0.2_f64));

    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            start_buddy_loop(pos, mouse, vel);
        });
    }

    let (x, y) = *pos.read();

    rsx! {
        div {
            class: "buddy",
            style: "left: {x}px; top: {y}px;",
            onmousemove: move |_| {},
            for (i, (msg, _ts)) in messages.iter().enumerate() {
                div {
                    class: "buddy-bubble",
                    style: "bottom: {20 + i as i32 * 24}px;",
                    "{msg}"
                }
            }
            "(o_o)"
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn start_buddy_loop(
    mut pos: Signal<(f64, f64)>,
    mut mouse: Signal<(f64, f64)>,
    mut vel: Signal<(f64, f64)>,
) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    let onmousemove =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
            mouse.set((e.client_x() as f64, e.client_y() as f64));
        });
    window
        .add_event_listener_with_callback("mousemove", onmousemove.as_ref().unchecked_ref())
        .ok();
    onmousemove.forget();

    let cb: AnimCallback = std::rc::Rc::new(std::cell::RefCell::new(None));
    let cb_clone = cb.clone();

    *cb.borrow_mut() = Some(Closure::new(move |_t: f64| {
        let w = web_sys::window()
            .and_then(|w| w.inner_width().ok())
            .and_then(|v| v.as_f64())
            .unwrap_or(800.0);
        let h = web_sys::window()
            .and_then(|w| w.inner_height().ok())
            .and_then(|v| v.as_f64())
            .unwrap_or(600.0);

        let (mut x, mut y) = *pos.read();
        let (mx, my) = *mouse.read();
        let (mut vx, mut vy) = *vel.read();

        // Home position: bottom-right corner
        let home_x = w - 80.0;
        let home_y = h - 60.0;
        let hx = home_x - x;
        let hy = home_y - y;
        let home_dist = (hx * hx + hy * hy).sqrt();
        if home_dist > 1.0 {
            let pull = 0.02;
            vx += (hx / home_dist) * pull * home_dist.min(200.0) / 200.0;
            vy += (hy / home_dist) * pull * home_dist.min(200.0) / 200.0;
        }

        // Flee from cursor
        let dx = x - mx;
        let dy = y - my;
        let dist = (dx * dx + dy * dy).sqrt();
        let flee_radius = 150.0;
        if dist < flee_radius && dist > 0.0 {
            let force = (flee_radius - dist) / flee_radius * 2.0;
            vx += (dx / dist) * force;
            vy += (dy / dist) * force;
        }

        // Wander
        let mut seed = (x * 1000.0 + y * 7.0) as u64;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = (seed as f64 / u64::MAX as f64) - 0.5;
        vx += r * 0.03;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r2 = (seed as f64 / u64::MAX as f64) - 0.5;
        vy += r2 * 0.03;

        vx *= 0.96;
        vy *= 0.96;

        let speed = (vx * vx + vy * vy).sqrt();
        let max_speed = 3.0;
        if speed > max_speed {
            vx = vx / speed * max_speed;
            vy = vy / speed * max_speed;
        }

        x += vx;
        y += vy;

        let margin = 40.0;
        let top_margin = 80.0;
        if x < margin {
            x = margin;
            vx = vx.abs();
        }
        if x > w - margin {
            x = w - margin;
            vx = -vx.abs();
        }
        if y < top_margin {
            y = top_margin;
            vy = vy.abs();
        }
        if y > h - margin {
            y = h - margin;
            vy = -vy.abs();
        }

        pos.set((x, y));
        vel.set((vx, vy));

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
