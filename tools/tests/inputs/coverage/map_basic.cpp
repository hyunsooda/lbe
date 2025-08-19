#include <iostream>
#include <map>

using namespace std;

int main() {
    map<string, int> thisismymap;
    thisismymap["123"] = 999;
    int v = thisismymap["123"];

    if (v == 9999) {
        printf("%d\n", thisismymap["888"]);
        v = 456;
    } else {
        v = 123;
    }
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/map_basic.cpp.ll)
//_:_// [+] IR file instrumented (covout/instrumented_map_basic.cpp.ll)
//_:_// [+] Binary created (map_basic.cpp)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/map_basic.cpp 
//_:_// +-------------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                                | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +-------------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/map_basic.cpp | 100.00  |                 | 50.00    | 12(15:F)           | 72.73   | 12,13,14        |
//_:_// +-------------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

