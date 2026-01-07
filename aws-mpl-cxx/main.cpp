#include "lib.rs.h"
#include <iostream>

int main() {
    // Call the Rust function directly
    MyData data = { 1, rust::String("Example") };
    if (process_data(data)) {
        std::cout << "Success!" << std::endl;
    }
    MyConfig config = { 1, rust::String("Example") };
    auto obj = create_client(config);
    std::cout << obj->get_value() << std::endl;
    delete_client(std::move(obj));
    return 0;
}
