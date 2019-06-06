
sleef_files="sleefsimddp_AVX.bc"

cd sleef/build/lib
for f in `ls sleef*.ll`
do 
    `llvm-config --bindir`/llvm-as $f
done
