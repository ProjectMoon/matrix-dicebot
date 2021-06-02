fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &["protos/dicebot.proto", "protos/web-api.proto"],
        &["protos/"],
    )?;
    //tonic_build::compile_protos("protos/*.proto")?;
    Ok(())
}
