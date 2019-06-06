
cd sleef/build/lib
for f in `ls *.ll`
do 
    `llvm-config --bindir`/llvm-as $f
done
