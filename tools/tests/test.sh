tool=../bin/debug/tools
common_libpath=../bin/debug
coverage_libname=coverage_runtime
asan_libname=asan_runtime
fuzzer_libname=fuzzer_runtime
symbolic_libname=symbolic_runtime
race_libname=race_runtime

outdir=covout
covout='cov.out'
input=$1
outbin=$(echo "$input" | sed 's:.*/::' | sed 's/\.c$//') # remove filepath and extension (e.g., "a/b/c.c" => "c")

if [[ "$input" == *.c ]]; then
    compiler="clang"
elif [[ "$input" == *.cpp ]]; then
    compiler="clang++"
else
    echo "Error: No valid file extension found (.c or .cpp)"
    exit 1
fi

# 1. compile
$tool -c $compiler -o $outdir -b $outbin -q $common_libpath -w $coverage_libname -a $common_libpath -s $asan_libname -f $common_libpath -m $fuzzer_libname -v $common_libpath -g $symbolic_libname -k $common_libpath -j $race_libname -i $input
# 2. run
COVERAGE_OUTPUT=$covout COLOR=1 LD_LIBRARY_PATH=$libpath ./$outdir/$outbin
