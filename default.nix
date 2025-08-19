{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    llvmPackages_17.llvm
    llvmPackages_17.clang
    llvmPackages_17.libllvm
    cargo
    rustc
    libxml2 # for llvm-ir
    libffi # for inkwell
    gtest # for integration with unit test
    z3 # for symbolic fuzzing
  ];
  shellHook = ''
    echo "LLVM 17 and Clang 17 are now available in the shell."
  '';
}
