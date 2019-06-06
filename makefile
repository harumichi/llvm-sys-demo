LLVM_BIN = `llvm-config --bindir`
CC = $(LLVM_BIN)/clang

.PHONY: cargo libcaller.so main
all: main

cargo:
	cargo build
	./target/debug/llvm_demo 2>| caller.ll

libcaller.so:
	$(LLVM_BIN)/llvm-as caller.ll
	$(LLVM_BIN)/opt -O2 caller.bc > caller-opt.bc
	$(LLVM_BIN)/llvm-dis caller-opt.bc
	$(CC) -S caller-opt.bc
	$(CC) -fpic -shared caller-opt.bc -o $@


main: main.c libcaller.so
	$(CC) -o $@ $< -L`pwd` -Wl,-rpath `pwd` -lcaller 

clean:
	rm -f main libcaller.so
