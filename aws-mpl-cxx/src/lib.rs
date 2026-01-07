#[cxx::bridge]
mod ffi {
    // Shared struct: visible to both languages
    struct MyData {
        id: u32,
        name: String,
    }
    struct MyConfig {
        id: u32,
        name: String,
    }

    extern "Rust" {
        // Function implemented in Rust, callable from C++
        fn process_data(data: MyData) -> bool;
        type MyClient;  // Opaque type declaration
        fn create_client(value: &MyConfig) -> Box<MyClient>;
        fn delete_client(client: Box<MyClient>);
        fn get_value(self: &MyClient) -> i32;
    }
}

struct MyClient {}
fn create_client(_value: &ffi::MyConfig) -> Box<MyClient> {
    Box::new(MyClient {})
}
// Standard Rust implementation (no no_mangle or extern "C" needed)
fn process_data(data: ffi::MyData) -> bool {
    println!("Processing: {}", data.name);
    !data.name.is_empty()
}

impl MyClient {
    fn get_value(&self) -> i32 {
        42
    }
}

fn delete_client(_client: Box<MyClient>) {
    // The Box will be automatically dropped when this function ends
    // You can add cleanup logic here if needed
    println!("Client deleted");
}
