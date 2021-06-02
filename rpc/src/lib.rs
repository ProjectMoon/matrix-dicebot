pub mod protos {
    pub mod web_api {
        tonic::include_proto!("web_api");
    }

    pub mod dicebot {
        tonic::include_proto!("dicebot");
    }
}
