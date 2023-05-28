#[tokio::main]
async fn main() {
    let args = axact::argparser::get_arg_parser();
    axact::router::start_server(args).await;
}
