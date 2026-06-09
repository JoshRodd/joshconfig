#[path = "../shellenv.rs"]
mod shellenv;

fn main() {
    std::process::exit(shellenv::emit_shell_env());
}
