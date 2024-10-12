use nutt_web::http::response::responder::Responder;
use nutt_web::http::response::Response;
use nutt_web_macro::get;
use nutt_web::modules::router::route::Route;

fn main() {

}

#[get("/")]
pub async fn test() -> Response {
    "test".into_response()
}