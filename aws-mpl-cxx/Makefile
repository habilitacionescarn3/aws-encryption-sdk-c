# Simple Makefile - all C++ bridge code is now compiled into the Rust static library

all:
	g++ main.cpp ./target/cxxbridge/aws-mpl-cxx/src/lib.rs.cc \
		$$(find target/debug/build -path "*/cxx-*/out/libcxxbridge1.a" | head -n1) \
		-L./target/debug -laws_mpl_cxx \
		-I./target/cxxbridge/aws-mpl-cxx/src/ \
		-I./target/cxxbridge \
		-lc++ -lpthread -ldl \
		-std=c++11 \
		-o main

clean:
	rm -f main

.PHONY: all clean
