use crate::llvm_intrinsic::{cstr_to_str, get_instr_filename};
use inkwell::values::InstructionOpcode;
use inkwell::{basic_block::BasicBlock, module::Module};
use petgraph::prelude::{DiGraphMap, Direction};
use std::collections::HashMap;

pub struct CFG<'ctx> {
    graph: DiGraphMap<usize, ()>,
    bbs: HashMap<usize, BasicBlock<'ctx>>,
}

impl<'ctx> CFG<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Self {
        let module_filename = cstr_to_str(module.get_source_file_name());
        let mut bbs = HashMap::new();
        let mut graph: DiGraphMap<usize, ()> = DiGraphMap::new();
        let funcs: Vec<_> = module.get_functions().collect();
        for func in funcs {
            for basic_blk in func.get_basic_blocks() {
                if let Some(first_instr) = basic_blk.get_first_instruction() {
                    // this guard prevents instrumentation of unknown code (e.g., C++)
                    if let Some(instr_filename) = get_instr_filename(&first_instr) {
                        if instr_filename != module_filename {
                            continue;
                        }
                    }
                }
                if let Some(term) = basic_blk.get_terminator() {
                    let bb_addr = basic_blk.as_mut_ptr() as usize;
                    bbs.insert(bb_addr, basic_blk);
                    match term.get_opcode() {
                        InstructionOpcode::Br => {
                            if !term.is_conditional() {
                                let next_bb = term.get_operand(0).unwrap().right().unwrap();
                                let next_bb_addr = next_bb.as_mut_ptr() as usize;
                                graph.add_edge(bb_addr, next_bb_addr, ());
                                bbs.insert(next_bb_addr, next_bb);
                            } else {
                                let (tbr, fbr) = (
                                    term.get_operand(2).unwrap().right().unwrap(),
                                    term.get_operand(1).unwrap().right().unwrap(),
                                );
                                let (tbr_addr, fbr_addr) =
                                    (tbr.as_mut_ptr() as usize, fbr.as_mut_ptr() as usize);
                                graph.add_edge(bb_addr, tbr_addr, ());
                                graph.add_edge(bb_addr, fbr_addr, ());
                                bbs.insert(tbr_addr, tbr);
                                bbs.insert(fbr_addr, fbr);
                            }
                        }
                        InstructionOpcode::Switch => {
                            for op in term.get_operands() {
                                if let Some(case_bb) = op.unwrap().right() {
                                    let case_bb_addr = case_bb.as_mut_ptr() as usize;
                                    graph.add_edge(bb_addr, case_bb_addr, ());
                                    bbs.insert(case_bb_addr, case_bb);
                                }
                            }
                        }
                        // TODO:
                        // InstructionOpcode::Invoke => {}
                        _ => {}
                    }
                }
            }
        }
        Self { graph, bbs }
    }

    fn get_neighbor(&self, bb: &BasicBlock<'ctx>, dir: Direction) -> Vec<BasicBlock<'ctx>> {
        let bb_addr = bb.as_mut_ptr() as usize;
        let edges: Vec<usize> = self.graph.neighbors_directed(bb_addr, dir).collect();
        edges
            .iter()
            .map(|addr| self.bbs.get(&addr).unwrap().clone())
            .collect()
    }

    pub fn preds(&self, bb: &BasicBlock<'ctx>) -> Vec<BasicBlock<'ctx>> {
        self.get_neighbor(bb, Direction::Incoming)
    }

    pub fn succs(&self, bb: &BasicBlock<'ctx>) -> Vec<BasicBlock<'ctx>> {
        self.get_neighbor(bb, Direction::Outgoing)
    }
}
