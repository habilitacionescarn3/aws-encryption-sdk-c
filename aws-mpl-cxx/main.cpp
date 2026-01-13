#include "aws-mpl-cxx/src/lib.rs.h"
#include "rust/cxx.h"
#include <iostream>

int main() {
    auto config = default_client_config();
    auto obj = create_kms_client(config);
    delete_kms_client(std::move(obj));
    std::cout << "Success!\n";
    return 0;
}
