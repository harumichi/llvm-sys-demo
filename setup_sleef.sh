git clone https://github.com/shibatch/sleef.git
cd sleef
mkdir build && cd build
cmake -DSLEEF_ENABLE_LLVM_BITCODE=1 ..
make

cd lib
for f in `ls sleef*.ll`
do 
    `llvm-config --bindir`/llc $f -o ${f/ll/bc}
done
