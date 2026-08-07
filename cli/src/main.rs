use work_context_manager::App;

fn main() {
    let app = App::new();
    println!("{} v{}", app.name(), app.version());
}
