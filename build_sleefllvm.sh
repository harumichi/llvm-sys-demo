git clone https://github.com/shibatch/sleef.git
cd sleef
mkdir build && cd build
cmake -DSLEEF_ENABLE_LLVM_BITCODE=1 ..
make
