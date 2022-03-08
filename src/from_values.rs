use glib::{Type, StaticType, Value};
use glib::value::{FromValue, ValueTypeChecker, ValueTypeMismatchOrNoneError, ValueTypeMismatchError};
use std::error::Error;

pub trait FromValues<'a> {
	type Error: Error;
	type Types: IntoIterator<Item=Type>;

	fn from_values(args: &'a [Value]) -> Result<Self, Self::Error> where Self: Sized;
	fn static_types() -> Self::Types;
}

macro_rules! impl_signal_arguments {
	($count:literal; ($($tx:ident),*)) => {
		#[allow(non_snake_case)]
		impl<'a, $($tx,)*> FromValues<'a> for ($($tx, )*) where
			$($tx: FromValue<'a> + StaticType,)*
			$(ValueTypeMismatchOrNoneError<ValueTypeMismatchError>: From<<$tx::Checker as ValueTypeChecker>::Error>,)*
		{
			type Error = ValueTypeMismatchOrNoneError<ValueTypeMismatchError>;
			type Types = [Type; $count];

			fn from_values(args: &'a [Value]) -> Result<Self, Self::Error> {
				match args {
					[$($tx,)*] => Ok(($($tx.get()?,)*)),
					_ => Err(ValueTypeMismatchOrNoneError::UnexpectedNone),
				}
			}

			fn static_types() -> Self::Types {
				[$($tx::static_type(),)*]
			}
		}
	};
}

impl_signal_arguments! { 0; () }
impl_signal_arguments! { 1; (T0) }
impl_signal_arguments! { 2; (T0, T1) }
impl_signal_arguments! { 3; (T0, T1, T2) }
impl_signal_arguments! { 4; (T0, T1, T2, T3) }
impl_signal_arguments! { 5; (T0, T1, T2, T3, T4) }
impl_signal_arguments! { 6; (T0, T1, T2, T3, T4, T5) }
impl_signal_arguments! { 7; (T0, T1, T2, T3, T4, T5, T6) }
impl_signal_arguments! { 8; (T0, T1, T2, T3, T4, T5, T6, T7) }
impl_signal_arguments! { 9; (T0, T1, T2, T3, T4, T5, T6, T7, T8) }
impl_signal_arguments! { 10; (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) }
impl_signal_arguments! { 11; (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) }
impl_signal_arguments! { 12; (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) }
