use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
type AnimCallback =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut(f64)>>>>;

#[component]
pub fn BuddyCharacter() -> Element {
    let pos_x = use_signal(|| 60.0_f64);
    let pos_y = use_signal(|| 60.0_f64);
    #[allow(unused_variables)]
    let mouse_x = use_signal(|| -1000.0_f64);
    #[allow(unused_variables)]
    let mouse_y = use_signal(|| -1000.0_f64);
    #[allow(unused_variables)]
    let vel_x = use_signal(|| 0.3_f64);
    #[allow(unused_variables)]
    let vel_y = use_signal(|| 0.2_f64);

    // Track mouse globally
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            start_buddy_loop(pos_x, pos_y, mouse_x, mouse_y, vel_x, vel_y);
        });
    }

    let x = *pos_x.read();
    let y = *pos_y.read();

    rsx! {
        div {
            class: "buddy",
            style: "left: {x}px; top: {y}px;",
            onmousemove: move |_| {},
            "(o_o)"
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn start_buddy_loop(
    mut pos_x: Signal<f64>,
    mut pos_y: Signal<f64>,
    mut mouse_x: Signal<f64>,
    mut mouse_y: Signal<f64>,
    mut vel_x: Signal<f64>,
    mut vel_y: Signal<f64>,
) {
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    // Mouse tracking
    let onmousemove =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
            mouse_x.set(e.client_x() as f64);
            mouse_y.set(e.client_y() as f64);
        });
    window
        .add_event_listener_with_callback("mousemove", onmousemove.as_ref().unchecked_ref())
        .ok();
    onmousemove.forget();

    // Animation loop
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

        let mut x = *pos_x.read();
        let mut y = *pos_y.read();
        let mx = *mouse_x.read();
        let my = *mouse_y.read();
        let mut vx = *vel_x.read();
        let mut vy = *vel_y.read();

        // Flee from cursor if too close
        let dx = x - mx;
        let dy = y - my;
        let dist = (dx * dx + dy * dy).sqrt();
        let flee_radius = 150.0;
        if dist < flee_radius && dist > 0.0 {
            let force = (flee_radius - dist) / flee_radius * 2.0;
            vx += (dx / dist) * force;
            vy += (dy / dist) * force;
        }

        // Wander gently
        let mut seed = (x * 1000.0 + y * 7.0) as u64;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = (seed as f64 / u64::MAX as f64) - 0.5;
        vx += r * 0.05;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r2 = (seed as f64 / u64::MAX as f64) - 0.5;
        vy += r2 * 0.05;

        // Damping
        vx *= 0.98;
        vy *= 0.98;

        // Clamp speed
        let speed = (vx * vx + vy * vy).sqrt();
        let max_speed = 3.0;
        if speed > max_speed {
            vx = vx / speed * max_speed;
            vy = vy / speed * max_speed;
        }

        x += vx;
        y += vy;

        // Bounce off edges
        let margin = 40.0;
        if x < margin {
            x = margin;
            vx = vx.abs();
        }
        if x > w - margin {
            x = w - margin;
            vx = -vx.abs();
        }
        if y < margin {
            y = margin;
            vy = vy.abs();
        }
        if y > h - margin {
            y = h - margin;
            vy = -vy.abs();
        }

        pos_x.set(x);
        pos_y.set(y);
        vel_x.set(vx);
        vel_y.set(vy);

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
