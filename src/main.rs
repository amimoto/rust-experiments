use smol;

use smol::Executor;

pub mod client;

fn main() {
    smol::block_on(async {
        let mut client = client::Wamp::connect(
                                            "host:port",
                                            "realm",
                                            "username",
                                            "password"
                                        ).await;
        let ex = Executor::new();
            ex.run(async {
            client.call("auth.whoami").await;
            client.run().await;
        }).await;
    });
}
