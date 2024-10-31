use crate::xcap::Window;

pub fn get_window(title: &str) -> Option<Window> {
    Window::all().ok().and_then(|windows| {
        windows
            .into_iter()
            .find(|window| window.title().contains(title))
    })
}
