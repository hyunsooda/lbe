#include <iostream>
#include <map>

using namespace std;

int main() {
    map<string, float> m;
    m["123"] = 999.99;

    try {
        int value = m.at("12");
        cout << value << endl;
    } catch (const out_of_range& e) {
        cout << "out of range error" << endl;
    }

    try {
        int value = m.at("123");
        cout << value << endl;
    } catch (const out_of_range& e) {
        cout << "out of range error" << endl;
    }
}


// Reaseon: 22(15:F)
//   - If line 22 has not been covered, either a line 18 or 19 should have been covered, thus line 15(the same as with line 17(`try`)) should be opposite branch.

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/try_map.cpp.ll)
//_:_// [+] IR file instrumented (covout/instrumented_try_map.cpp.ll)
//_:_// [+] Binary created (try_map.cpp)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/try_map.cpp 
//_:_// out of range error
//_:_// 999
//_:_// +-----------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                              | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +-----------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/try_map.cpp | 100.00  |                 | 80.00    | 22(15:F)           | 80.00   | 12,21,22        |
//_:_// +-----------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

