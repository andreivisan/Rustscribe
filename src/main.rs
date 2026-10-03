mod cli;

fn main() -> rustscribe::api::ApiResult<()> {
    cli::run()
}
