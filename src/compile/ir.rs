use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;

use crate::{
    ast::{BinaryOp, Comparison, CoordinateAccess, Ident, UnaryOp},
    permute,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IRType {
    Number,
    Vec2,
    Vec3,
    Never,
    Bool,
    NumberList,
    Vec2List,
    Vec3List,
}
impl IRType {
    // Is this type a valid input or output for an IRChunk
    pub fn is_value_type(&self) -> bool {
        matches!(
            self,
            Self::Number
                | Self::Vec2
                | Self::Vec3
                | Self::NumberList
                | Self::Vec2List
                | Self::Vec3List
        )
    }
    pub fn downcast_list(&self) -> Option<Self> {
        match self {
            Self::NumberList => Some(Self::Number),
            Self::Vec2 => Some(Self::Vec2),
            Self::Vec3 => Some(Self::Vec3),
            _ => None,
        }
    }
    pub fn list_of(&self, len: Id) -> Result<IROp> {
        Ok(match self {
            IRType::Number => IROp::NumberList(len),
            IRType::Vec2 => IROp::Vec2List(len),
            IRType::Vec3 => IROp::Vec3List(len),
            t => bail!("cannot create a list of {:?}", t),
        })
    }
}

/// Identifies a numeric argument to the relevant IRChunk by index in the argument list
#[derive(Debug, Clone, Copy)]
pub struct Id {
    idx: u32,
    pub t: IRType,
}

impl Id {
    pub fn new(idx: u32, t: IRType) -> Self {
        Self { idx, t }
    }
    pub fn with_idx(&self, idx: u32) -> Self {
        Self { t: self.t, idx }
    }
}
impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}
impl Eq for Id {}
impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.idx.cmp(&other.idx))
    }
}
impl Ord for Id {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        //Always Some
        self.idx.cmp(&other.idx)
    }
}

/// Identifies an argument to the current broadcast scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BroadcastArg {
    pub t: IRType,
    pub id: u8,
}
// typed indentifier that identifies an item of type and index in args
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArgId(Id);

/// ### Desmoxide IR format
/// This is mostly equivalent to the TAC-based IR format used by desmos (see https://github.com/DesModder/DesModder/blob/main/parsing/IR.ts).
/// #### types
///  * `Number`: floating point value of unspecified precision (the Constant IR operation currently uses f64, but this precision is not guaranteed)
///  * `Vec2/3`: two and three-dimensional vectors
///  * `Never`: instructions that Never yield a value of any kind
///  * `Bool`: comparison instructions that yield boolean type
///  * `List`: opaque list identifer
/// and special broadcasting instructions are used to iterate over complex types component-wise
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IROp {
    Binary(Id, Id, BinaryOp),
    Unary(Id, UnaryOp),
    /// 64-bit floating point constant
    Const(f64),
    /// 64-bit integer constant
    IConst(i64),
    /// load args\[a] to this reg
    LoadArg(ArgId),
    /// load args\[a]\[i] (if args\[a] is a list of number)
    CoordinateOf(Id, CoordinateAccess),
    /// 2d vector.
    Vec2(Id, Id),
    /// 3d vector.
    Vec3(Id, Id, Id),
    /// Instantiate an empty list of number with length a
    NumberList(Id),
    /// Instantiate an empty list of Vec2 with length a
    Vec2List(Id),
    /// Instantiate an empty list of Vec3 with length a
    Vec3List(Id),
    /// length of list at a
    ListLength(Id),
    /// Begins a broadcast loop that executes its body over indices 0->end_index inclusive, and stores its output in b
    BeginBroadcast {
        end_index: Id,
        write_to: Id,
    },
    /// Only allowed directly following SetBroadcast or BeginBroadcast instructions. Sets the broadcast argument slot at b to the item a
    SetBroadcastArg(Id, BroadcastArg),
    LoadBroadcastArg(BroadcastArg),
    EndBroadcast {
        /// ID of the corresponding BeginBroadcast register
        begin: Id,
        /// ID of the value to push to the output list
        ret: Id,
    },
    Comparison {
        lhs: Id,
        comp: Comparison,
        rhs: Id,
    },
    /// Piecewise consist of a BeginPiecewise, any number of InnerPiecewise s and an EndPiecewise
    BeginPiecewise {
        comp: Id,
        res: Id,
    },
    InnerPiecewise {
        comp: Id,
        res: Id,
    },
    EndPiecewise {
        default: Id,
    },
    /// Return the value stored
    Ret(Id),
}
impl IROp {
    pub fn type_of(&self) -> IRType {
        // this match statement should always be exhaustive to prevent new instructions from being made without assigning them a type
        match self {
            //Number type
            IROp::Binary(_, _, _)
            | IROp::Unary(_, _)
            | IROp::Const(_)
            | IROp::IConst(_)
            | IROp::CoordinateOf(_, _)
            | IROp::ListLength(_) => IRType::Number,
            //Passthrough type
            IROp::LoadArg(ArgId(Id { t, .. }))
            | IROp::LoadBroadcastArg(BroadcastArg { t, .. })
            | IROp::BeginPiecewise {
                res: Id { t, .. }, ..
            }
            | IROp::BeginBroadcast {
                write_to: Id { t, .. },
                ..
            } => *t,
            //Opaque declarations
            IROp::Vec2(_, _) => IRType::Vec2,
            IROp::Vec3(_, _, _) => IRType::Vec3,
            IROp::NumberList(_) => IRType::NumberList,
            IROp::Vec2List(_) => IRType::Vec2List,
            IROp::Vec3List(_) => IRType::Vec3List,
            IROp::BeginBroadcast { .. } => IRType::Never,
            IROp::SetBroadcastArg(_, _) => IRType::Never,
            IROp::EndBroadcast { .. } => IRType::Never,
            IROp::Comparison { .. } => IRType::Bool,
            // it is invalid to refer to a non- BeginPiecewise instruction
            IROp::InnerPiecewise { .. } | IROp::EndPiecewise { .. } => IRType::Never,
            IROp::Ret(i) => i.t,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IRInstructionSeq {
    backing: BTreeMap<Id, IROp>,
}
impl IRInstructionSeq {
    pub fn push(&mut self, op: IROp) {
        let _ = self.place(op);
    }
    pub fn place(&mut self, op: IROp) -> Id {
        let mut nid = 0;
        if let Some(v) = self.backing.last_key_value() {
            nid = v.0.idx + 1;
        };
        let id = Id::new(nid, op.type_of());
        self.backing.insert(id, op);
        id
    }
    pub fn coordinates_of2d(&mut self, point: Id) -> (Id, Id) {
        (
            self.place(IROp::CoordinateOf(point, CoordinateAccess::DotAccessX)),
            self.place(IROp::CoordinateOf(point, CoordinateAccess::DotAccessY)),
        )
    }
    pub fn coordinates_of3d(&mut self, point: Id) -> (Id, Id, Id) {
        (
            self.place(IROp::CoordinateOf(point, CoordinateAccess::DotAccessX)),
            self.place(IROp::CoordinateOf(point, CoordinateAccess::DotAccessY)),
            self.place(IROp::CoordinateOf(point, CoordinateAccess::DotAccessZ)),
        )
    }
    pub fn place_block(&mut self, ops: &[IROp]) -> Option<Id> {
        if ops.len() == 0 {
            None
        } else {
            let first = self.place(ops[0]);
            for i in &ops[1..] {
                self.push(*i);
            }
            Some(first)
        }
    }
    pub fn get(&self, id: &Id) -> Result<&IROp> {
        self.backing.get(id).context("Could not get IR opcode")
    }
    pub fn latest(&self) -> Result<&IROp> {
        self.backing
            .last_key_value()
            .map(|a| a.1)
            .context("called latest on empty InstructionSeq")
    }
}
pub struct BroadcastBuilder<'a> {
    seq: &'a mut IRInstructionSeq,
    args: Vec<BroadcastArg>,
}
impl<'a> BroadcastBuilder<'a> {}
