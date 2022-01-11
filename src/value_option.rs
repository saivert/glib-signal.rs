use glib::{Value, value::ToValue};

pub trait ToValueOption: Sized {
	type Type: Into<Self>;

	fn to_value_option(self) -> Option<Value>;
}

pub struct PrimitiveValue<T>(T);
impl<T> From<T> for PrimitiveValue<T> {
	fn from(v: T) -> Self { Self(v) }
}
impl ToValueOption for PrimitiveValue<()> {
	type Type = ();

	fn to_value_option(self) -> Option<Value> { None }
}
impl ToValueOption for PrimitiveValue<usize> {
	type Type = usize;

	#[cfg(target_pointer_width = "16")]
	fn to_value_option(self) -> Option<Value> { Some((self.0 as u16).to_value()) }
	#[cfg(target_pointer_width = "32")]
	fn to_value_option(self) -> Option<Value> { Some((self.0 as u32).to_value()) }
	#[cfg(target_pointer_width = "64")]
	fn to_value_option(self) -> Option<Value> { Some((self.0 as u64).to_value()) }
}

impl<T: ToValue> ToValueOption for T {
	type Type = T;

	fn to_value_option(self) -> Option<Value> {
		Some(ToValue::to_value(&self))
	}
}
