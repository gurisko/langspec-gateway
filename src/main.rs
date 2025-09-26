use langspec::GatewayProxy;
use log::info;
use pingora::prelude::*;

fn main() {
    // Set up logging
    env_logger::init();

    // Create the server with configuration
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    // Define upstream servers
    let upstreams = vec![
        "127.0.0.1:8001".to_string(),
        "127.0.0.1:8002".to_string(),
        "127.0.0.1:8003".to_string(),
    ];

    // Create proxy instance
    let mut proxy = http_proxy_service(&server.configuration, GatewayProxy::new(upstreams));

    // Add listening address
    proxy.add_tcp("127.0.0.1:8080");

    // Add the service to the server
    server.add_service(proxy);

    // Run the server
    info!("Starting proxy server on 127.0.0.1:8080");
    info!("Configured upstreams: 127.0.0.1:8001, 127.0.0.1:8002, 127.0.0.1:8003");
    server.run_forever();
}
