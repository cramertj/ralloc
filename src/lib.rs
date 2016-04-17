#![cfg_attr(not(test), feature(oom))]
#![cfg_attr(test, feature(const_fn))]

#![feature(alloc)]
#![feature(stmt_expr_attributes)]
#![feature(unique)]

#[cfg(target_os = "redox")]
extern crate system;
#[cfg(not(target_os = "redox"))]
#[macro_use]
extern crate syscall;

#[macro_use]
extern crate extra;
extern crate alloc;

mod sys;
mod block;
mod block_list;