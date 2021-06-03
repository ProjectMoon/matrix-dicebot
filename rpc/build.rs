fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "only-client") {
        tonic_build::configure().build_server(false).compile(
            &["protos/dicebot.proto", "protos/web-api.proto"],
            &["protos/"],
        )?;

        Ok(())
    } else {
        tonic_build::configure().compile(
            &["protos/dicebot.proto", "protos/web-api.proto"],
            &["protos/"],
        )?;
        Ok(())
    }
}
