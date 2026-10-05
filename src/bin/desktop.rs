fn main() {
    struct Diagnostics;
    impl log::Log for Diagnostics {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Warn
        }
        fn log(&self, record: &log::Record<'_>) {
            if self.enabled(record.metadata()) {
                eprintln!("{}: {}", record.level(), record.args());
            }
        }
        fn flush(&self) {}
    }
    let _ = log::set_logger(&Diagnostics);
    log::set_max_level(log::LevelFilter::Warn);
    rustscribe::gpui::run();
}
