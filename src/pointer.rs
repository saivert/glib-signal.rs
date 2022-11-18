use {
	glib::{translate::ToGlibPtr, value::FromValue, StaticType, Type, Value},
	std::ops::Deref,
};

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Pointer<T>(pub *mut T);

impl<T> Pointer<T> {
	pub fn as_ptr_mut(&self) -> *mut T {
		self.0
	}

	pub fn as_ptr(&self) -> *const T {
		self.0 as *const _
	}

	pub fn into_inner(self) -> *mut T {
		self.0
	}
}

impl<T> From<*mut T> for Pointer<T> {
	fn from(ptr: *mut T) -> Self {
		Self(ptr)
	}
}

impl<T> Into<*mut T> for Pointer<T> {
	fn into(self) -> *mut T {
		self.into_inner()
	}
}

impl<T> Into<*const T> for Pointer<T> {
	fn into(self) -> *const T {
		self.into_inner() as *const _
	}
}

impl<T> Deref for Pointer<T> {
	type Target = *mut T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

unsafe impl<'a, T> FromValue<'a> for Pointer<T> {
	type Checker = glib::value::GenericValueTypeChecker<Pointer<T>>;

	unsafe fn from_value(value: &'a Value) -> Self {
		let value = value.to_glib_none();
		Self(glib::gobject_ffi::g_value_get_pointer(value.0) as *mut T)
	}
}

impl<T> StaticType for Pointer<T> {
	fn static_type() -> Type {
		Type::POINTER
	}
}
