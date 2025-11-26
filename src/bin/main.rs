fn main() {
    let networks = wifi_scan::scan().expect("Cannot scan network");
    for network in networks {
        println!(
            "{}",
            network
        );
    }
}
