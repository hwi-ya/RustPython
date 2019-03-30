use std::cell::RefCell;
use std::ops::AddAssign;

use num_bigint::BigInt;
use num_traits::Zero;

use crate::function::OptionalArg;
use crate::pyobject::{PyContext, PyObjectRef, PyRef, PyResult, PyValue};
use crate::vm::VirtualMachine;

use super::objint::PyIntRef;
use super::objiter;
use super::objtype::PyClassRef;

#[derive(Debug)]
pub struct PyEnumerate {
    counter: RefCell<BigInt>,
    iterator: PyObjectRef,
}
type PyEnumerateRef = PyRef<PyEnumerate>;

impl PyValue for PyEnumerate {
    fn class(vm: &VirtualMachine) -> PyClassRef {
        vm.ctx.enumerate_type()
    }
}

fn enumerate_new(
    cls: PyClassRef,
    iterable: PyObjectRef,
    start: OptionalArg<PyIntRef>,
    vm: &VirtualMachine,
) -> PyResult<PyEnumerateRef> {
    let counter = match start {
        OptionalArg::Present(start) => start.as_bigint().clone(),
        OptionalArg::Missing => BigInt::zero(),
    };

    let iterator = objiter::get_iter(vm, &iterable)?;
    PyEnumerate {
        counter: RefCell::new(counter.clone()),
        iterator,
    }
    .into_ref_with_type(vm, cls)
}

impl PyEnumerateRef {
    fn next(self, vm: &VirtualMachine) -> PyResult {
        let iterator = &self.iterator;
        let counter = &self.counter;
        let next_obj = objiter::call_next(vm, iterator)?;
        let result = vm
            .ctx
            .new_tuple(vec![vm.ctx.new_int(counter.borrow().clone()), next_obj]);

        AddAssign::add_assign(&mut counter.borrow_mut() as &mut BigInt, 1);

        Ok(result)
    }
}

pub fn init(context: &PyContext) {
    let enumerate_type = &context.enumerate_type;
    objiter::iter_type_init(context, enumerate_type);
    extend_class!(context, enumerate_type, {
        "__new__" => context.new_rustfunc(enumerate_new),
        "__next__" => context.new_rustfunc(PyEnumerateRef::next)
    });
}
