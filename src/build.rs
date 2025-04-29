use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        web_platform: { all(target_family = "wasm", target_os = "unknown") },
    }
}
