//! The gallery, in a window drawn by Blitz (`native`) or in a browser
//! (`web`).

fn main() {
    #[cfg(feature = "native")]
    dioxus_native::launch(unit_gallery::Gallery);
    #[cfg(all(feature = "web", not(feature = "native")))]
    dioxus_web::launch::launch_cfg(unit_gallery::Gallery, dioxus_web::Config::new());
}
