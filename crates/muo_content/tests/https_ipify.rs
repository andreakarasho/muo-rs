//! Real HTTPS request to the service ServerList.FindPublicAddress uses; `cargo test -p muo_content --test https_ipify -- --ignored`.
#[test]
#[ignore = "needs network"]
fn https_get_ipify() {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(muo_content::http_tls::tls_config())
        .build()
        .into();
    let ip = agent
        .get("https://api.ipify.org")
        .call()
        .unwrap()
        .body_mut()
        .read_to_string()
        .unwrap();
    assert!(ip.trim().parse::<std::net::IpAddr>().is_ok(), "{ip}");
    eprintln!("ipify: {ip}");
}
