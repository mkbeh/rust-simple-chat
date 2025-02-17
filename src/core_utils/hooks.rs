pub fn setup_panic_hook() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        default_panic(panic_info);
        std::process::exit(1);
    }))
}
