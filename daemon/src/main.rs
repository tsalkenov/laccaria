use zbus::dbus_interface;

struct ProcessManager;

#[dbus_interface(name = "org.laccaria.ProcessManager")]
impl ProcessManager {
    
}
fn main() {
    println!("Hello, world!");
}
