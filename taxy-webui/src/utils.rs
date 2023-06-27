use taxy_api::port::Port;

pub fn format_addr(port: &Port) -> String {
    let bind = port
        .bind
        .iter()
        .map(|addr| addr.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{} [{}]",
        port.protocol.to_string().to_ascii_uppercase(),
        bind
    )
}
