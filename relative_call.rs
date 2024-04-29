#[inline(never)]
fn function_stack_ref(stack: &mut [u64]) -> u64 {
    stack[0] = 2;
    return stack[0];
}

#[no_mangle]
pub fn entrypoint(x: &mut u8) -> u64 {
    let stack = core::mem::MaybeUninit::<[u64; 32]>::uninit();
    let mut stack = unsafe { stack.assume_init() };
    *x = 2;
    stack[0] = *x as u64;
    let y = function_stack_ref(&mut stack);
    let z = *x as u64;
    return function_sum(y, z);
}

#[inline(never)]
fn function_sum(x: u64, y: u64) -> u64 {
    return x + y;
}