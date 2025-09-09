pub(crate) mod listtable;

pub(crate) use listtable::ListTable;

#[macro_export]
macro_rules! impl_for_all_tuples {
    ($name: ident) => {
        $name!((0: T0));
        $name!((0: T0), (1:T1));
        $name!((0: T0), (1:T1), (2:T2));
    }
}
