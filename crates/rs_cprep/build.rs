fn main() {
    let mut cfg = lalrpop::Configuration::new();
    cfg.set_in_dir("grammar");
    cfg.process().unwrap();
}
