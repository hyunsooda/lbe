### [TextBook] Language-Based Engineering: A Comprehensive Approach to Software Analysis and Hardening

This book explores dynamic analysis and hardening in the context of software security and engineering.

- Ch1. Coverage
    - Reference: [Hardhat Coverage](https://hardhat.org/hardhat2/redirect?r=%2Fhardhat-runner%2Fdocs%2Fguides%2Ftest-contracts)
- Ch2. Buffer Overrun: Address Sanitizer
    - Reference: [AddressSanitizer: A Fast Address Sanity Checker](https://www.usenix.org/system/files/conference/atc12/atc12-final39.pdf)
- Ch3. Fuzzing
    - Reference: [AFL](https://github.com/google/AFL/blob/master/docs/technical_details.txt)
- Ch4. Symbolic Execution
    - Reference: [KLEE](https://klee-se.org/)
- Ch5. Delta Debugging
    - References:
        - [Simplifying and Isolating Failure-Inducing Input](https://www.cs.purdue.edu/homes/xyzhang/fall07/Papers/delta-debugging.pdf)
        - [Yesterday, my program worked. Today, it does not. Why?](https://dl.acm.org/doi/10.1145/318774.318946)
- Ch6. LLM-based Synthesis
    - Reference: [Fuzz4All: Universal Fuzzing with Large Language Models](https://fuzz4all.github.io/)
- Ch7. Data Race Detector: Thread Sanitizer
    - References:
        - [Time, Clocks, and the Ordering of Events in a Distributed System](https://lamport.azurewebsites.net/pubs/time-clocks.pdf)
        - [Eraser: A Dynamic Data Race Detector for Multithreaded Programs](https://dl.acm.org/doi/pdf/10.1145/265924.265927)

### Environment
- Nix: The build environment (LLVM, gtest) is available through the Nix shell, while the Rust environment (Cargo, rustc) utilizes the host environment.
- How to build: Type `just b`
- How to test: Type `just t`

### Auther

Hyunsoo Shin (신현수)
