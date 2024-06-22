use env_logger::Builder;
use log::{ LevelFilter, info, trace, warn };

fn main() {
    Builder::new().filter(None, LevelFilter::Info).init();

    info!("This is an info message");
    warn!("This is a warning message");
    trace!("This is a trace message");
}
