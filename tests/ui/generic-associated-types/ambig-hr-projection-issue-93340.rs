//@ revisions: old next
//@[next] compile-flags: -Znext-solver
pub trait Scalar: 'static {
    type RefType<'a>: ScalarRef<'a>;
}

pub trait ScalarRef<'a>: 'a {}

fn cmp_eq<'a, 'b, A: Scalar, B: Scalar, O: Scalar>(a: A::RefType<'a>, b: B::RefType<'b>) -> O {
    todo!()
}

fn build_expression<A: Scalar, B: Scalar, O: Scalar>(
) -> impl Fn(A::RefType<'_>, B::RefType<'_>) -> O {
    //[next]~^ ERROR overflow evaluating the requirement
    //[next]~| ERROR overflow evaluating the requirement
    cmp_eq
    //~^ ERROR type annotations needed
    //[next]~| ERROR overflow evaluating the requirement
    //[next]~| ERROR overflow evaluating the requirement
}

fn main() {}
