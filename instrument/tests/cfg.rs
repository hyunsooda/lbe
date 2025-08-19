use inkwell::{basic_block::BasicBlock, context::Context};
use instrument::{cfg::CFG, llvm_intrinsic::cstr_to_str};
mod util;

#[test]
fn test_asd() {
    let src = r#"
    #include <stdio.h>

    int main() {
        int i = 0;
        if (i == 0) {
            return 1;
        } else {
            return 2;
        }
    }
"#;

    let mem_buf = util::load_ir(&src);
    let context = Context::create();
    let module = context.create_module_from_ir(mem_buf).unwrap();
    let cfg = CFG::new(&module);
    for func in module.get_functions() {
        if cstr_to_str(func.get_name()) == "main" {
            let bbs: Vec<BasicBlock> = func.get_basic_blocks().clone();
            // successor
            let nexts = cfg.succs(&bbs[0]);
            assert_eq!(nexts.len(), 2);
            let next_next_t = cfg.succs(&nexts[0]);
            let next_next_f = cfg.succs(&nexts[1]);
            assert_eq!(next_next_t.len(), 1);
            assert_eq!(next_next_f.len(), 1);
            assert_eq!(next_next_t[0], bbs[bbs.len() - 1]);
            assert_eq!(next_next_f[0], bbs[bbs.len() - 1]);

            // predecessor
            let pred1 = cfg.preds(&next_next_f[0]);
            let pred2 = cfg.preds(&next_next_t[0]);
            assert_eq!(pred1.len(), pred2.len());
            assert_eq!(pred1.len(), 2);
            assert_eq!(pred1[0], pred2[0]);
            assert_eq!(pred1[1], pred2[1]);
            assert_eq!(pred1[0], bbs[1]);
            assert_eq!(pred1[1], bbs[2]);

            let pred3 = cfg.preds(&pred1[0]);
            let pred4 = cfg.preds(&pred1[1]);
            assert_eq!(pred3.len(), pred4.len());
            assert_eq!(pred3.len(), 1);
            assert_eq!(pred4.len(), 1);
            assert_eq!(pred3[0], bbs[0]);
            assert_eq!(pred4[0], bbs[0]);
        }
    }
}
