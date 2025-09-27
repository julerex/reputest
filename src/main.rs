use azure_functions::bindings::{HttpRequest, HttpResponse};
use azure_functions::func;
use log::info;

#[func]
#[binding(name = "req", auth_level = "anonymous")]
pub fn reputest(req: HttpRequest) -> HttpResponse {
    info!("Reputesting!");
    
    HttpResponse::ok()
        .header("Content-Type", "text/plain")
        .body("Reputesting!")
}

fn main() {
    env_logger::init();
    azure_functions::worker_main(std::env::args());
}
